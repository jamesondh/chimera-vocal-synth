//! `chimera throat song <file> <out> [--tele voice.tele]`: throat singing shaped by the breath.
//!
//! A Tuvan singer holds a drone and its melody for as long as the lungs hold out; the breath, not
//! a bar line, ends the phrase. Each line of a song file is one breath:
//!
//! ```text
//! # style   drone  melody (harmonic:secs, "*" holds until the air runs out)   options
//! khoomei   65     16:2 18:1 20:1 24:*                                         lungs=3.5
//! sygyt     82     20:1.5 24:1 27:*                                            pressure=11
//! ```
//!
//! Every breath: an inhale (negative lung pressure through the open glottis, so the air rushing
//! in makes its own noise), the drone scooping up from below into its pitch, the melody settling
//! in after it, then singing while the lungs empty. The air spent is the glottal airflow the
//! simulation computes, integrated over time, so a louder or more pressed drone runs out sooner.
//! Over the last part of the breath the pressure sags; the phrase ends when the score does or
//! the air is gone, whichever is first.
//!
//! Not physical: the refill. The inhale's sound comes from the airway, but the lungs are refilled
//! to `lungs` over the inhale by decree (the model's open glottis, ~0.3 cm², passes ~0.8 L/s at a
//! gasp's pressure, where a singer gasping through a wide glottis and mouth takes ~3 L/s).

use crate::js::fixed;
use crate::render::{TELE_HOP, TeleRecorder};
use crate::throat::{Style, solve_set, to_artic, to_patch};
use crate::{Ctx, FS, body, body_mix, peak, wav_name};
use chimera::anatomy::human::{ArticPatch, HumanArtic, NEUTRAL};
use chimera::voice::human_airway::HumanAirway;
use chimera::voice::human_voice::{HumanVoice, SourcePatch};

/// samples per rendered block: telemetry hops (768) and the airflow integration land on these
const BLOCK: usize = 32;
/// the drone's onset starts this fraction of its pitch below and glides up
const SCOOP_FROM: f64 = 0.6;
/// time constant of the scoop (s)
const SCOOP_GLIDE: f64 = 0.09;
/// the mouth leaves the inhale posture for the first note's as the drone starts, and the note loop
/// begins this long after (the melody resonance settles in behind the drone)
const MELODY_AFTER: f64 = 0.25;
/// below this fraction of the breath, lung pressure sags with the volume left
const SAG_FROM: f64 = 0.12;
/// the phrase ends with this fraction of the breath left (the singer does not empty to residual)
const AIR_FLOOR: f64 = 0.02;
/// refill rate of a catch-breath (L/s)
const INHALE_RATE: f64 = 3.0;
/// the inhale's gate swells in and out over this long (s)
const RAMP_IN: f64 = 0.08;
/// a phrase's release: breath and pitch fall away over this long (s), the pitch toward this
/// fraction (the real singers' harmonics curve down to nothing over ~0.3 s)
const RELEASE: f64 = 0.3;
const RELEASE_TO: f64 = 0.55;

#[derive(Clone, Debug)]
struct Phrase {
    line: usize,
    style: String,
    f0: f64,
    /// (harmonic, seconds); None holds until the air runs out
    notes: Vec<(usize, Option<f64>)>,
    lungs: f64,
    pressure: f64,
}

fn parse_song(text: &str) -> Result<Vec<Phrase>, String> {
    let mut out = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let err = |m: String| format!("line {}: {m}", i + 1);
        let mut it = line.split_whitespace();
        let style = it.next().unwrap().to_string();
        Style::by_name(&style).map_err(err)?;
        let f0: f64 = it
            .next()
            .ok_or_else(|| err("needs a drone pitch".into()))?
            .parse()
            .map_err(|_| err("bad drone pitch".into()))?;
        let mut p = Phrase {
            line: i + 1,
            style,
            f0,
            notes: Vec::new(),
            lungs: 3.5,
            pressure: 12.0,
        };
        for tok in it {
            if let Some((k, v)) = tok.split_once('=') {
                let v: f64 = v
                    .parse()
                    .map_err(|_| err(format!("bad value in {tok:?}")))?;
                match k {
                    "lungs" => p.lungs = v,
                    "pressure" => p.pressure = v,
                    _ => return Err(err(format!("unknown option {k:?} (lungs, pressure)"))),
                }
            } else {
                let (n, d) = tok.split_once(':').ok_or_else(|| {
                    err(format!("note {tok:?} must be harmonic:secs or harmonic:*"))
                })?;
                let n: usize = n
                    .parse()
                    .map_err(|_| err(format!("bad harmonic in {tok:?}")))?;
                let d = if d == "*" {
                    None
                } else {
                    Some(
                        d.parse::<f64>()
                            .map_err(|_| err(format!("bad seconds in {tok:?}")))?,
                    )
                };
                p.notes.push((n, d));
            }
        }
        if p.notes.is_empty() {
            return Err(err("a breath needs at least one note".into()));
        }
        if p.notes.iter().any(|n| n.1.is_none()) && p.notes.last().unwrap().1.is_some() {
            return Err(err(
                "only the last note can hold (*): notes after it would never sound".into(),
            ));
        }
        out.push(p);
    }
    if out.is_empty() {
        return Err("the song has no breaths".into());
    }
    Ok(out)
}

/// Postures for every (style, drone) in the song: all the harmonics one voice sings on one drone
/// share a home posture, as one singer's mouth would.
fn solve_song(song: &[Phrase], epi: f64) -> Result<Vec<(String, f64, usize, HumanArtic)>, String> {
    let mut air = HumanAirway::new(96000.0, body("male")?);
    let mut out = Vec::new();
    let mut seen: Vec<(String, u64)> = Vec::new();
    for p in song {
        let key = (p.style.clone(), p.f0.to_bits());
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        let ns: Vec<usize> = song
            .iter()
            .filter(|q| q.style == p.style && q.f0 == p.f0)
            .flat_map(|q| q.notes.iter().map(|n| n.0))
            .collect();
        let style = Style::by_name(&p.style)?;
        for (n, x, fx) in solve_set(&mut air, p.f0, &ns, epi, &style) {
            println!(
                "{} {} Hz: n={n} ({} Hz) cost {}",
                p.style,
                fixed(p.f0, 1),
                fixed(n as f64 * p.f0, 0),
                fixed(fx, 1)
            );
            out.push((p.style.clone(), p.f0, n, to_artic(&x, epi)));
        }
    }
    Ok(out)
}

/// The mouth while breathing in: jaw a little open, lips `lips` apart, tongue at rest.
fn inhale_posture(lips: f64) -> ArticPatch {
    to_patch(&HumanArtic {
        jaw: 0.5,
        lip_ap: lips,
        lip_prot: 0.3,
        ..NEUTRAL
    })
}

/// Renders the voice in BLOCK-sample steps, integrating the glottal airflow and recording
/// telemetry on its hop.
struct Run {
    voice: HumanVoice,
    out: Vec<f32>,
    tele: Option<TeleRecorder>,
    /// litres of air sent out through the glottis since `reset_air`
    spent: f64,
}

impl Run {
    fn step(&mut self) -> Result<(), String> {
        let pos = self.out.len();
        self.out.resize(pos + BLOCK, 0.0);
        self.voice.render(&mut self.out[pos..]);
        // glottal volume velocity (cm³/s) now; sampled every BLOCK samples (1.5 kHz) its mean is
        // the breath's mean flow (the cycle's phase drifts against the block)
        let u = self.voice.airway.net.u[self.voice.airway.glottis_port];
        self.spent += u.max(0.0) * (BLOCK as f64 / FS) / 1000.0;
        if let Some(t) = self.tele.as_mut()
            && self.out.len().is_multiple_of(TELE_HOP)
        {
            t.snap(&mut self.voice)?;
        }
        Ok(())
    }
    fn now(&self) -> f64 {
        self.out.len() as f64 / FS
    }
    /// Move the breath gate from `a` to `b` linearly over `secs`.
    fn ramp_gate(&mut self, a: f64, b: f64, secs: f64) -> Result<(), String> {
        let start = self.now();
        while self.now() < start + secs {
            let u = (self.now() - start) / secs;
            self.voice.set_gate(a + (b - a) * u);
            self.step()?;
        }
        self.voice.set_gate(b);
        Ok(())
    }
    fn run_for(&mut self, secs: f64) -> Result<(), String> {
        let end = self.now() + secs;
        while self.now() < end {
            self.step()?;
        }
        Ok(())
    }
}

/// `throat song <file> <out> [--tele voice.tele] [--epi 0.9]`
pub fn song(ctx: &Ctx, a: &[String]) -> Result<(), String> {
    let mut tele_out: Option<String> = None;
    let mut epi = 0.9;
    // a quick gasp: -11 to -27 dB under the singing on the real recordings, ≈ -24 dB here
    let mut inhale_p = -12.0;
    let mut inhale_lips = 0.9;
    let mut rest = Vec::new();
    let mut it = a.iter();
    while let Some(x) = it.next() {
        match x.as_str() {
            "--tele" => tele_out = Some(it.next().ok_or("--tele needs a file")?.clone()),
            "--epi" => {
                epi = it
                    .next()
                    .ok_or("--epi needs a value")?
                    .parse()
                    .map_err(|_| "bad --epi")?
            }
            "--inhale-p" => {
                inhale_p = it
                    .next()
                    .ok_or("--inhale-p needs a value")?
                    .parse()
                    .map_err(|_| "bad --inhale-p")?
            }
            "--inhale-lips" => {
                inhale_lips = it
                    .next()
                    .ok_or("--inhale-lips needs a value")?
                    .parse()
                    .map_err(|_| "bad --inhale-lips")?
            }
            _ => rest.push(x.clone()),
        }
    }
    let file = rest.first().ok_or("throat song needs a song file")?;
    let name = rest
        .get(1)
        .cloned()
        .unwrap_or_else(|| "out/throat-song".into());
    let text = std::fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
    let song = parse_song(&text).map_err(|e| format!("{file}: {e}"))?;
    let shapes = solve_song(&song, epi)?;
    let shape = |p: &Phrase, n: usize| {
        shapes
            .iter()
            .find(|s| s.0 == p.style && s.1 == p.f0 && s.2 == n)
            .unwrap()
            .3
    };

    let b = body("male")?;
    let mut voice = HumanVoice::new(FS, b, 2);
    voice.set_source(&SourcePatch {
        tension: Some(1.0),
        brightness: Some(0.6),
        roughness: Some(0.05),
        drift: Some(6.0),
        vibrato_depth: Some(8.0),
        vibrato_rate: Some(6.5),
        ..Default::default()
    });
    voice.set_articulation(&inhale_posture(inhale_lips), Some(false));
    voice.set_gate(0.0);
    let tele = if tele_out.is_some() {
        Some(TeleRecorder::new(&mut voice))
    } else {
        None
    };
    let mut run = Run {
        voice,
        out: Vec::new(),
        tele,
        spent: 0.0,
    };
    run.run_for(0.3)?;

    let mut air = 0.0; // litres in the lungs above the floor the singer stops at
    for (k, p) in song.iter().enumerate() {
        // inhale: refill to this breath's capacity
        let t_in = ((p.lungs - air) / INHALE_RATE).clamp(0.45, 1.5);
        run.voice.set_source(&SourcePatch {
            voiced: Some(false),
            breathiness: Some(1.0),
            pressure: Some(inhale_p),
            ..Default::default()
        });
        // (the first breath starts from this posture; later ones reached it during the release)
        run.voice
            .set_articulation(&inhale_posture(inhale_lips), Some(false));
        // the gasp swells in and out: a step to the inhale pressure radiated a click
        run.ramp_gate(0.0, 1.0, RAMP_IN)?;
        run.run_for(t_in - 2.0 * RAMP_IN)?;
        run.ramp_gate(1.0, 0.0, RAMP_IN)?;
        air = p.lungs;
        // onset: the drone scoops up into its pitch on the first note's posture
        let first = shape(p, p.notes[0].0);
        run.run_for(0.03)?;
        run.voice.set_source(&SourcePatch {
            voiced: Some(true),
            breathiness: Some(0.0),
            pressure: Some(p.pressure),
            f0: Some(p.f0 * SCOOP_FROM),
            glide: Some(0.01),
            ..Default::default()
        });
        run.voice.set_articulation(&to_patch(&first), Some(false));
        run.voice.set_gate(1.0);
        run.run_for(0.03)?;
        run.voice.set_source(&SourcePatch {
            f0: Some(p.f0),
            glide: Some(SCOOP_GLIDE),
            ..Default::default()
        });
        let start = run.now();
        run.spent = 0.0;
        run.run_for(MELODY_AFTER)?;
        // the melody, until the score or the air runs out
        let mut ended_by_air = false;
        let mut sag_p = p.pressure;
        'notes: for &(n, d) in &p.notes {
            run.voice
                .set_articulation(&to_patch(&shape(p, n)), Some(false));
            let note_end = d.map(|d| run.now() + d);
            loop {
                let left = (air - run.spent) / p.lungs;
                if left <= AIR_FLOOR {
                    ended_by_air = true;
                    break 'notes;
                }
                if note_end.is_some_and(|e| run.now() >= e) {
                    break;
                }
                // the last part of the breath: pressure sags with the air left
                let target = if left < SAG_FROM {
                    p.pressure * (0.55 + 0.45 * (left - AIR_FLOOR) / (SAG_FROM - AIR_FLOOR))
                } else {
                    p.pressure
                };
                if (target - sag_p).abs() > 0.05 {
                    sag_p = target;
                    run.voice.set_source(&SourcePatch {
                        pressure: Some(sag_p),
                        ..Default::default()
                    });
                }
                run.step()?;
            }
        }
        let sang = run.now() - start;
        let used = run.spent;
        air -= used;
        println!(
            "breath {} (line {}, {} {} Hz): inhale {} s, sang {} s, air {} of {} L, mean flow {} mL/s, ended by {}",
            k + 1,
            p.line,
            p.style,
            fixed(p.f0, 1),
            fixed(t_in, 2),
            fixed(sang, 1),
            fixed(used, 2),
            fixed(p.lungs, 1),
            fixed(1000.0 * used / sang.max(1e-9), 0),
            if ended_by_air { "the air" } else { "the score" }
        );
        // release: the breath leaves, the pitch falls away under it, and the mouth relaxes toward
        // the inhale meanwhile (moving it after, in silence, the displaced air ticked audibly)
        run.voice
            .set_articulation(&inhale_posture(inhale_lips), Some(false));
        run.voice.set_source(&SourcePatch {
            f0: Some(p.f0 * RELEASE_TO),
            glide: Some(RELEASE / 2.0),
            ..Default::default()
        });
        run.ramp_gate(1.0, 0.0, RELEASE)?;
        run.run_for(0.08)?;
    }
    run.run_for(0.5)?;

    let file_out = wav_name(&name);
    ctx.write_wav(&file_out, &run.out)?;
    println!(
        "wrote {file_out}  {} s  peak {}",
        fixed(run.now(), 2),
        fixed(peak(&run.out), 3)
    );
    if let (Some(path), Some(r)) = (&tele_out, run.tele) {
        std::fs::write(path, r.bytes(body_mix("male")?)).map_err(|e| format!("{path}: {e}"))?;
        println!(
            "{path}: {} telemetry frames every {TELE_HOP} samples",
            r.frames
        );
    }
    Ok(())
}
