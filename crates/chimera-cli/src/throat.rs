//! `chimera throat …`: Tuvan overtone singing (khöömei / sygyt) by tract tuning.
//!
//! The melody of khöömei is a single drone harmonic made to stand ~20–40 dB above its neighbours
//! by a very sharp vocal-tract resonance (F2 and F3 brought together by a small, tightly tuned
//! front cavity). `solve` searches the articulators, with the frequency-domain airway model, for
//! a posture whose transfer function peaks on harmonic n of the drone and falls off on n±1;
//! `render` sings a drone through the time-domain airway while stepping between solved postures.

use crate::js::fixed;
use crate::{Ctx, FS, arg, body, peak, wav_name};
use chimera::airway::frequency::transfer_function;
use chimera::anatomy::human::{ArticKey, ArticPatch, HumanArtic, NEUTRAL};
use chimera::math::{js_max, js_round};
use chimera::util::optimize::{NelderMeadOpts, nelder_mead};
use chimera::voice::human_airway::HumanAirway;
use chimera::voice::human_voice::{HumanVoice, SourcePatch};

/// tbX tbY jaw lipAp lipProt lipWidth root larynx ttX ttY
const LO: [f64; 10] = [1.2, 5.0, 0.05, 0.15, 0.0, 0.0, -0.9, -1.3, 3.5, 5.0];
const HI: [f64; 10] = [4.6, 8.8, 1.6, 1.6, 1.4, 1.0, 0.9, 1.3, 8.0, 10.8];

pub fn to_artic(p: &[f64], epilarynx: f64) -> HumanArtic {
    HumanArtic {
        tb_x: p[0],
        tb_y: p[1],
        jaw: p[2],
        lip_ap: p[3],
        lip_prot: p[4],
        lip_width: p[5],
        root: p[6],
        larynx: p[7],
        tt_x: p[8],
        tt_y: p[9],
        epilarynx,
        ..NEUTRAL
    }
}

fn to_patch(a: &HumanArtic) -> ArticPatch {
    let mut p = ArticPatch::default();
    for k in ArticKey::ALL {
        p.set(k, a.get(k));
    }
    p
}

/// dB response at harmonics 1..=nh of f0, and the narrowest tract area.
fn harmonic_response(air: &mut HumanAirway, a: &HumanArtic, freqs: &[f64]) -> (Vec<f64>, f64) {
    air.apply(a, 0.0);
    let el = air.net.linear_elements();
    let h = transfer_function(&el, air.pharynx.first_node(), freqs, &[air.glottis_port]);
    let mut min_area = f64::INFINITY;
    for &x in air.ph_areas.iter().chain(air.or_areas.iter()) {
        min_area = min_area.min(x);
    }
    (h.mag, min_area)
}

/// The cost scores every harmonic up to this frequency: a competing resonance anywhere in the
/// band the ear hears as the whistle's surroundings counts (with a fixed count of 26 harmonics, a
/// 65 Hz drone was scored only up to 1.7 kHz, and stray resonances near 2 kHz went unseen).
const BAND_TOP: f64 = 4000.0;
const WIDE: f64 = 0.2;
/// source roll-off assumed when scoring (dB per octave): a pressed voice still loses about this
const TILT: f64 = -10.0;

/// Harmonics scored for drone f0 and target n.
fn n_harmonics(f0: f64, n: usize) -> usize {
    ((BAND_TOP / f0).ceil() as usize).max(n + 3)
}

/// Frequencies the cost samples: harmonics 1..=nh of f0, then the target ±WIDE·f0.
fn cost_freqs(f0: f64, n: usize) -> Vec<f64> {
    let mut v: Vec<f64> = (1..=n_harmonics(f0, n)).map(|k| k as f64 * f0).collect();
    v.push((n as f64 - WIDE) * f0);
    v.push((n as f64 + WIDE) * f0);
    v
}

/// Predicted output level per harmonic: tract response plus the source roll-off.
/// `h` holds the harmonics first, as `cost_freqs` lays them out, then the two target-width probes.
fn out_db(h: &[f64]) -> Vec<f64> {
    (0..h.len() - 2)
        .map(|i| h[i] + TILT * ((i + 1) as f64).log2())
        .collect()
}

/// Cost of posture p for target harmonic n (lower is better): the target's lead over its
/// neighbours and over everything above the drone's low band, with the target held over ±WIDE of
/// a harmonic spacing (the time-domain airway is lossier than this model, so a needle-thin
/// resonance blunts into its neighbours), and a guard against near-closures (frication).
fn cost(
    air: &mut HumanAirway,
    p: &[f64],
    n: usize,
    freqs: &[f64],
    epi: f64,
    home: Option<&[f64]>,
) -> f64 {
    let a = to_artic(p, epi);
    let (h, min_area) = harmonic_response(air, &a, freqs);
    let tilt_n = TILT * (n as f64).log2();
    let o = out_db(&h);
    let nh = o.len();
    let t = (h[n - 1].min(h[nh]).min(h[nh + 1])) + tilt_n;
    let nb = |d: isize| -> f64 {
        let i = n as isize - 1 + d;
        if i >= 0 && (i as usize) < nh {
            o[i as usize]
        } else {
            -200.0
        }
    };
    let near = nb(-1).max(nb(1));
    let next = nb(-2).max(nb(2));
    // everything else above the drone's low band (excluding n±2): the strongest competitor. The
    // low harmonics (< ~450 Hz, F1 kept low by the small rounded mouth) carry the drone itself.
    let mut other = -200.0f64;
    for (i, &x) in o.iter().enumerate() {
        if (i as isize - (n as isize - 1)).abs() > 2 && freqs[i] > 450.0 {
            other = other.max(x);
        }
    }
    let lead = (t - near) + 0.5 * (t - next) + 1.5 * (t - other).min(15.0);
    let mut pen = 400.0 * js_max(0.0, 0.3 - min_area).powi(2);
    if let Some(h0) = home {
        for i in 0..p.len() {
            pen += HOME_W[i] * 100.0 * ((p[i] - h0[i]) / (HI[i] - LO[i])).powi(2);
        }
    }
    -lead + pen
}

/// How strongly each articulator is held near the shared home posture while the melody moves
/// (tbX tbY jaw lipAp lipProt lipWidth root larynx ttX ttY). A khöömei singer's lips, jaw and
/// larynx barely move between notes; the tongue does the work behind the lips.
const HOME_W: [f64; 10] = [0.3, 0.3, 3.0, 3.0, 3.0, 3.0, 1.0, 3.0, 0.3, 0.3];

/// Best posture for harmonic n of f0, multi-start Nelder–Mead.
pub fn solve_one(
    air: &mut HumanAirway,
    f0: f64,
    n: usize,
    epi: f64,
    starts: &[Vec<f64>],
    home: Option<&[f64]>,
) -> (Vec<f64>, f64) {
    let freqs = cost_freqs(f0, n);
    let mut best: (Vec<f64>, f64) = (starts[0].clone(), f64::INFINITY);
    for x0 in starts {
        let r = nelder_mead(
            |p| cost(air, p, n, &freqs, epi, home),
            x0,
            NelderMeadOpts {
                lo: Some(LO.to_vec()),
                hi: Some(HI.to_vec()),
                step: Some(vec![0.4, 0.4, 0.2, 0.15, 0.3, 0.2, 0.3, 0.3, 0.4, 0.4]),
                max_iter: Some(1500),
                ..Default::default()
            },
        );
        if r.fx < best.1 {
            best = (r.x, r.fx);
        }
    }
    best
}

/// Postures for a whole melody, all near one shared home posture so that moving between notes is
/// mostly the tongue. Each note solved freely is a candidate home (one alone can land in a basin,
/// e.g. a wider mouth, from which the other notes are unreachable); the candidate whose melody
/// costs least overall wins.
pub fn solve_set(
    air: &mut HumanAirway,
    f0: f64,
    ns: &[usize],
    epi: f64,
) -> Vec<(usize, Vec<f64>, f64)> {
    let mut sorted: Vec<usize> = ns.to_vec();
    sorted.sort();
    sorted.dedup();
    let st = starts();
    let free: Vec<Vec<f64>> = sorted
        .iter()
        .map(|&n| solve_one(air, f0, n, epi, &st, None).0)
        .collect();
    let mut best: (f64, Vec<(usize, Vec<f64>, f64)>) = (f64::INFINITY, Vec::new());
    for home in &free {
        let mut starts_h = vec![home.clone()];
        starts_h.extend(free.iter().cloned());
        let set: Vec<(usize, Vec<f64>, f64)> = sorted
            .iter()
            .map(|&n| {
                let (x, fx) = solve_one(air, f0, n, epi, &starts_h, Some(home));
                (n, x, fx)
            })
            .collect();
        // the worst note matters most: a melody with one dead note is not a melody
        let total: f64 = set.iter().map(|s| s.2).sum::<f64>()
            + 2.0 * set.iter().map(|s| s.2).fold(f64::NEG_INFINITY, f64::max);
        if total < best.0 {
            best = (total, set);
        }
    }
    best.1
}

/// Starting postures: back-ish tongue with rounded lips (khöömei) and tip-up front-cavity
/// shapes (sygyt), at a few tongue heights.
fn starts() -> Vec<Vec<f64>> {
    let mut v = Vec::new();
    for &(tbx, tby) in &[(2.6, 7.3), (3.2, 7.6), (3.8, 7.9), (2.2, 6.8)] {
        // tip resting low (khöömei-like): lips rounded, small aperture
        v.push(vec![tbx, tby, 0.5, 0.35, 0.8, 0.3, 0.0, 0.2, 6.2, 7.0]);
        // tip raised toward the alveolar ridge (sygyt-like)
        v.push(vec![tbx, tby, 0.45, 0.45, 0.6, 0.4, 0.2, 0.3, 6.8, 9.6]);
    }
    v
}

fn parse_f(s: &str, what: &str) -> Result<f64, String> {
    s.parse().map_err(|_| format!("bad {what} {s:?}"))
}

/// `throat solve [f0=140] [n1 n2 …] [--epi 0.9]`: print the best posture per harmonic, with the
/// contrast it reaches, as rows for `throat render`.
pub fn solve(a: &[String]) -> Result<(), String> {
    let (rest, epi) = take_opt(a, "--epi", 0.9)?;
    let f0 = parse_f(arg(&rest, 0, "140"), "f0")?;
    let ns: Vec<usize> = if rest.len() > 1 {
        rest[1..]
            .iter()
            .map(|s| s.parse().map_err(|_| format!("bad harmonic {s:?}")))
            .collect::<Result<_, _>>()?
    } else {
        vec![6, 7, 8, 9, 10, 12]
    };
    let mut air = HumanAirway::new(96000.0, body("male")?);
    let set = solve_set(&mut air, f0, &ns, epi);
    for (n, x, fx) in &set {
        let (n, fx) = (*n, *fx);
        let freqs = cost_freqs(f0, n);
        let (h, min_area) = harmonic_response(&mut air, &to_artic(x, epi), &freqs);
        let h = out_db(&h);
        let t = h[n - 1];
        let near = h[n - 2].max(h[n]);
        let xs: Vec<String> = x.iter().map(|v| fixed(*v, 3)).collect();
        println!(
            "n={n:2} {} Hz  cost {}  lead over n±1 {} dB  minA {}  [{}]",
            fixed(n as f64 * f0, 0),
            fixed(fx, 1),
            fixed(t - near, 1),
            fixed(min_area, 2),
            xs.join(", ")
        );
        println!(
            "      palate clearance (cm, <0 = through the palate): tip {}  body {}",
            fixed(air.tract.contact_tip, 2),
            fixed(air.tract.contact_body, 2)
        );
        let hs: Vec<String> = h.iter().map(|v| fixed(v - t, 0)).collect();
        println!("      FD out dB rel. target, from h1: {}", hs.join(" "));
    }
    // how far each articulator travels across the melody
    let names = [
        "tbX", "tbY", "jaw", "lipAp", "lipProt", "lipWidth", "root", "larynx", "ttX", "ttY",
    ];
    let spans: Vec<String> = (0..10)
        .map(|i| {
            let (lo, hi) = set
                .iter()
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), s| {
                    (l.min(s.1[i]), h.max(s.1[i]))
                });
            format!("{} {}", names[i], fixed(hi - lo, 2))
        })
        .collect();
    println!("travel across the melody: {}", spans.join(", "));
    Ok(())
}

fn take_opt(a: &[String], name: &str, default: f64) -> Result<(Vec<String>, f64), String> {
    let mut rest = Vec::new();
    let mut val = default;
    let mut it = a.iter();
    while let Some(x) = it.next() {
        if x == name {
            val = parse_f(it.next().ok_or(format!("{name} needs a value"))?, name)?;
        } else {
            rest.push(x.clone());
        }
    }
    Ok((rest, val))
}

/// `throat render <out> [f0=140] [n:secs …] [--epi 0.9] [--tension 1.0] [--pressure 12]`:
/// solve each melody harmonic once, then sing the drone and step through the melody.
pub fn render(ctx: &Ctx, a: &[String]) -> Result<(), String> {
    let (a, epi) = take_opt(a, "--epi", 0.9)?;
    let (a, tension) = take_opt(&a, "--tension", 1.0)?;
    let (a, pressure) = take_opt(&a, "--pressure", 12.0)?;
    let (a, bright) = take_opt(&a, "--bright", 0.6)?;
    let (a, vib) = take_opt(&a, "--vib", 8.0)?;
    let (a, vib_rate) = take_opt(&a, "--vib-rate", 6.5)?;
    let name = arg(&a, 0, "out/throat").to_string();
    let f0 = parse_f(arg(&a, 1, "140"), "f0")?;
    let mut melody: Vec<(usize, f64)> = Vec::new();
    for s in a.iter().skip(2) {
        let (n, d) = s
            .split_once(':')
            .ok_or(format!("melody step {s:?} must be n:secs"))?;
        melody.push((
            n.parse().map_err(|_| format!("bad harmonic {n:?}"))?,
            parse_f(d, "secs")?,
        ));
    }
    if melody.is_empty() {
        melody = vec![
            (8, 1.5),
            (9, 1.5),
            (10, 1.5),
            (12, 1.5),
            (10, 1.0),
            (9, 1.0),
            (8, 2.0),
        ];
    }
    let b = body("male")?;
    let mut air = HumanAirway::new(96000.0, b);
    let ns: Vec<usize> = melody.iter().map(|m| m.0).collect();
    let shapes: Vec<(usize, HumanArtic)> = solve_set(&mut air, f0, &ns, epi)
        .into_iter()
        .map(|(n, x, fx)| {
            println!(
                "solved n={n} ({} Hz) cost {}",
                fixed(n as f64 * f0, 0),
                fixed(fx, 1)
            );
            (n, to_artic(&x, epi))
        })
        .collect();
    let mut voice = HumanVoice::new(FS, b, 2);
    voice.set_source(&SourcePatch {
        f0: Some(f0),
        pressure: Some(pressure),
        tension: Some(tension),
        breathiness: Some(0.0),
        brightness: Some(bright),
        roughness: Some(0.05),
        drift: Some(6.0),
        vibrato_depth: Some(vib),
        vibrato_rate: Some(vib_rate),
        ..Default::default()
    });
    let total: f64 = 0.4 + melody.iter().map(|m| m.1).sum::<f64>() + 0.5;
    let mut out = vec![0f32; js_round(FS * total) as usize];
    let block = 128;
    let first = shapes.iter().find(|s| s.0 == melody[0].0).unwrap().1;
    voice.set_articulation(&to_patch(&first), Some(false));
    let mut bounds = Vec::new();
    let mut t = 0.4;
    for &(n, d) in &melody {
        bounds.push((t, n));
        t += d;
    }
    let end = t;
    let mut pos = 0;
    let mut k = 0;
    while pos < out.len() {
        let now = pos as f64 / FS;
        while k < bounds.len() && now >= bounds[k].0 {
            let s = shapes.iter().find(|s| s.0 == bounds[k].1).unwrap().1;
            voice.set_articulation(&to_patch(&s), Some(false));
            k += 1;
        }
        voice.set_gate(if now < end { 1.0 } else { 0.0 });
        let len = block.min(out.len() - pos);
        voice.render(&mut out[pos..pos + len]);
        pos += len;
    }
    let file = wav_name(&name);
    ctx.write_wav(&file, &out)?;
    println!(
        "wrote {file}  {} s  peak {}",
        fixed(out.len() as f64 / FS, 2),
        fixed(peak(&out), 3)
    );
    Ok(())
}
