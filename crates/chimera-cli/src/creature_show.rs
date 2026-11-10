//! `chimera render creature-show <script.json> <out> [--tele show.ctele]`: a creature performing a
//! scripted sequence of calls and held notes, as one WAV, plus (with `--tele`) its telemetry over
//! time for the app's show page (`?show=<id>` with `"creature"`), which replays it in the
//! Bestiary's drawing of the creature.
//!
//! The script (times in seconds, rounded to the millisecond):
//!
//! ```json
//! { "creature": "flatulence", "params": {"size": 1.0}, "tail": 0.5,
//!   "events": [
//!     {"at": 0.5, "call": "squeak"},
//!     {"at": 3.0, "hold": 1.2, "params": {"gas": 150, "eas": 10, "push": "0:20,0.6:45"},
//!      "caption": "A held push", "hint": "…"} ] }
//! ```
//!
//! `params` sets the performer's controls from the start. A `call` plays one of the creature's
//! calls (optional `pitch`, `amp`), which drives its own controls and hands them back when it
//! ends. A `hold` turns the controls (constants, or courses `t:v,t:v,…` over the held note, timed
//! from the breath opening, set every millisecond as `render creature` does) at `at`, opens the
//! breath `settle` later (0.1 s: the controls settle first, as every call lets them) and keeps it
//! open `hold` seconds; the controls stay where it left them. Events may not overlap. Each event
//! is a cue — its time span, the call's name (or the hold's `caption`, else "Sustain") and hint —
//! for the show's captions; the render lasts until the last cue ends, plus `tail`.
//!
//! The creature runs as in the app (`CreatureVoice`, the worklet's engine path), sample by sample,
//! so the sound is the same whatever the block size and whether telemetry is taken; it is
//! deterministic.

use crate::js::fixed;
use crate::render::{TELE_HOP, course_at, measure, parse_course};
use crate::{Ctx, FS, arg, peak, wav_name};
use chimera::creatures::types::CreatureDef;
use chimera::creatures::voice::{CreatureTelemetry, CreatureVoice};
use chimera::creatures::{CREATURES, creature_by_id};
use chimera::math::js_round;
use serde_json::{Value, json};

/// `--tele` file version (see `CreatureTeleRecorder`).
pub const CTELE_VERSION: u32 = 1;
/// The script's clock: events and courses act on 1 ms ticks (48 samples; `TELE_HOP` is 16 ticks).
const TICK: usize = 48;
/// Bytes before the metadata: magic and six u32s.
const CTELE_HEADER: usize = 28;

enum Kind {
    Call { call: usize, pitch: Option<f64>, amp: f64 },
    Hold { settle: f64, hold: f64, consts: Vec<(usize, f64)>, courses: Vec<(usize, Vec<(f64, f64)>)> },
}

struct Event {
    at: f64,
    kind: Kind,
    caption: Option<String>,
    hint: Option<String>,
}

/// One event's span and what the show writes under it.
#[derive(Clone, Debug, PartialEq)]
pub struct Cue {
    pub at: f64,
    pub end: f64,
    pub name: String,
    pub hint: String,
}

pub struct Script {
    def: &'static CreatureDef,
    initial: Vec<(usize, f64)>,
    events: Vec<Event>,
    tail: f64,
}

fn num(v: &Value, what: &str) -> Result<f64, String> {
    v.as_f64().filter(|x| x.is_finite()).ok_or_else(|| format!("{what}: a number, not {v}"))
}

impl Script {
    pub fn parse(text: &str) -> Result<Script, String> {
        let j: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let id = j["creature"].as_str().ok_or("\"creature\": the creature's id")?;
        let def = creature_by_id(id).ok_or_else(|| format!("unknown creature {id}; have {}", CREATURES.iter().map(|c| c.id).collect::<Vec<_>>().join(", ")))?;
        let param = |k: &str| def.param_index(k).ok_or_else(|| format!("{id} has no parameter {k}"));
        let mut initial = Vec::new();
        if let Some(m) = j.get("params") {
            for (k, v) in m.as_object().ok_or("\"params\": an object")? {
                initial.push((param(k)?, num(v, k)?));
            }
        }
        let tail = j.get("tail").map(|v| num(v, "tail")).transpose()?.unwrap_or(0.5);
        let mut events = Vec::new();
        for (n, e) in j["events"].as_array().ok_or("\"events\": an array")?.iter().enumerate() {
            let what = format!("event {n}");
            let at = num(&e["at"], &format!("{what}: at"))?;
            let text = |k: &str| e.get(k).and_then(Value::as_str).map(str::to_string);
            let kind = if let Some(c) = e.get("call") {
                let c = c.as_str().ok_or_else(|| format!("{what}: call is a call id"))?;
                let call = def.call_index(c).ok_or_else(|| format!("{what}: {id} has no call {c}; have {}", def.calls.iter().map(|c| c.id).collect::<Vec<_>>().join(", ")))?;
                let pitch = e.get("pitch").map(|v| num(v, &format!("{what}: pitch"))).transpose()?;
                let amp = e.get("amp").map(|v| num(v, &format!("{what}: amp"))).transpose()?.unwrap_or(1.0);
                Kind::Call { call, pitch, amp }
            } else if let Some(h) = e.get("hold") {
                let hold = num(h, &format!("{what}: hold"))?;
                let settle = e.get("settle").map(|v| num(v, &format!("{what}: settle"))).transpose()?.unwrap_or(0.1);
                let (mut consts, mut courses) = (Vec::new(), Vec::new());
                if let Some(m) = e.get("params") {
                    for (k, v) in m.as_object().ok_or_else(|| format!("{what}: params is an object"))? {
                        match v.as_str() {
                            Some(s) => courses.push((param(k)?, parse_course(k, s)?)),
                            None => consts.push((param(k)?, num(v, k)?)),
                        }
                    }
                }
                if hold <= 0.0 || settle < 0.0 {
                    return Err(format!("{what}: hold > 0 and settle >= 0"));
                }
                Kind::Hold { settle, hold, consts, courses }
            } else {
                return Err(format!("{what}: either \"call\" or \"hold\""));
            };
            if at < 0.0 {
                return Err(format!("{what}: at >= 0"));
            }
            events.push(Event { at, kind, caption: text("caption"), hint: text("hint") });
        }
        events.sort_by(|a, b| a.at.total_cmp(&b.at));
        let s = Script { def, initial, events, tail };
        // one thing at a time: an event ends before the next starts
        for w in s.events.windows(2) {
            if w[1].at < s.busy_until(&w[0]) - 1e-9 {
                return Err(format!("the event at {} s starts before the one at {} s ends ({} s)", w[1].at, w[0].at, fixed(s.busy_until(&w[0]), 3)));
            }
        }
        Ok(s)
    }

    /// When an event lets go: a call's last key, a hold's breath closing.
    fn busy_until(&self, e: &Event) -> f64 {
        match &e.kind {
            Kind::Call { call, .. } => e.at + self.def.calls[*call].keys.last().map_or(0.0, |k| k.t),
            Kind::Hold { settle, hold, .. } => e.at + settle + hold,
        }
    }

    pub fn cues(&self) -> Vec<Cue> {
        self.events
            .iter()
            .map(|e| {
                let (end, name, hint) = match &e.kind {
                    Kind::Call { call, .. } => {
                        let c = &self.def.calls[*call];
                        (self.busy_until(e), c.name, c.hint)
                    }
                    // the pocket's pressure falls back over half a second, as a call allows
                    Kind::Hold { .. } => (self.busy_until(e) + 0.5, "Sustain", ""),
                };
                Cue { at: e.at, end, name: e.caption.clone().unwrap_or(name.to_string()), hint: e.hint.clone().unwrap_or(hint.to_string()) }
            })
            .collect()
    }
}

/// A time (s) in ticks.
fn ms(t: f64) -> usize {
    js_round(t * 1000.0) as usize
}

/// Render the script: the samples, and the telemetry if asked for.
pub fn render_script(s: &Script, tele: bool) -> Result<(Vec<f32>, Option<CreatureTeleRecorder>), String> {
    let mut v = CreatureVoice::new(s.def, FS, &s.initial);
    let cues = s.cues();
    let end = cues.iter().map(|c| c.end).fold(0.0, f64::max) + s.tail;
    let ticks = ms(end);
    let mut out = vec![0.0f32; ticks * TICK];
    let mut rec = if tele { Some(CreatureTeleRecorder::new(&mut v, cues)) } else { None };
    // the holds' breath on and off, in ticks
    let gates: Vec<Option<(usize, usize)>> = s
        .events
        .iter()
        .map(|e| match &e.kind {
            Kind::Hold { settle, hold, .. } => {
                let on = ms(e.at + settle);
                Some((on, on + ms(*hold)))
            }
            _ => None,
        })
        .collect();
    for (j, block) in out.chunks_mut(TICK).enumerate() {
        for (e, g) in s.events.iter().zip(&gates) {
            match &e.kind {
                Kind::Call { call, pitch, amp } if ms(e.at) == j => v.play_call(*call, *pitch, *amp),
                Kind::Hold { consts, courses, .. } => {
                    let (on, off) = g.unwrap();
                    if ms(e.at) == j {
                        v.set_params(consts);
                        for (k, keys) in courses {
                            v.set_params(&[(*k, keys[0].1)]);
                        }
                    }
                    if (on..off).contains(&j) {
                        let t = ((j - on) * TICK) as f64 / FS;
                        for (k, keys) in courses {
                            v.set_params(&[(*k, course_at(keys, t))]);
                        }
                    }
                    if j == on {
                        v.set_gate(1.0);
                    }
                    if j == off {
                        v.set_gate(0.0);
                    }
                }
                _ => {}
            }
        }
        v.render(block);
        if let Some(r) = rec.as_mut()
            && ((j + 1) * TICK) % TELE_HOP == 0
        {
            r.snap(&mut v)?;
        }
    }
    Ok((out, rec))
}

/// A creature's telemetry over time, for replaying a render in the app's creature view.
///
/// File layout, little-endian: `b"CHCT"`, then u32 version, sampleRate, hop, frames, frameLen and
/// metaLen (28 bytes), then `metaLen` bytes of UTF-8 JSON (space-padded to a multiple of 4):
/// `{"id", "params": [key…], "calls": [id…], "tubes": [{"name", "n"}…], "sources": [key…],
/// "cues": [{"at", "end", "name", "hint"}…]}`; then `frames` frames of `frameLen` f32s. A frame is
/// the wasm creature telemetry block in its order (crates/chimera-wasm `engine_creature_telemetry`,
/// what the live view gets): gate, outRms, clock, the active call's index (−1: none), the params
/// (smoothed, in `params` order), per tube its areas then its pressure RMS, then the source values.
/// Frame i is the state at sample i·hop (frame 0 before any sound); its RMS levels cover the hop
/// before it, as the live worklet's (one snapshot every 768 samples).
pub struct CreatureTeleRecorder {
    t: CreatureTelemetry,
    def: &'static CreatureDef,
    meta: Value,
    shape: (usize, Vec<usize>, usize),
    data: Vec<f32>,
    pub frames: u32,
}

impl CreatureTeleRecorder {
    fn new(v: &mut CreatureVoice, cues: Vec<Cue>) -> Self {
        let mut t = v.new_telemetry();
        v.telemetry(&mut t);
        let def = v.def;
        let meta = json!({
            "id": def.id,
            "params": def.params.iter().map(|p| p.key).collect::<Vec<_>>(),
            "calls": def.calls.iter().map(|c| c.id).collect::<Vec<_>>(),
            "tubes": t.tubes.iter().map(|tb| json!({ "name": tb.name, "n": tb.areas.len() })).collect::<Vec<_>>(),
            "sources": t.source.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
            "cues": cues.iter().map(|c| json!({ "at": c.at, "end": c.end, "name": c.name, "hint": c.hint })).collect::<Vec<_>>(),
        });
        let shape = Self::shape_of(&t);
        let mut r = CreatureTeleRecorder { t, def, meta, shape, data: Vec::new(), frames: 0 };
        r.push_current();
        r
    }

    fn shape_of(t: &CreatureTelemetry) -> (usize, Vec<usize>, usize) {
        (t.params.len(), t.tubes.iter().map(|tb| tb.areas.len()).collect(), t.source.len())
    }

    fn frame_len(&self) -> usize {
        4 + self.shape.0 + 2 * self.shape.1.iter().sum::<usize>() + self.shape.2
    }

    fn snap(&mut self, v: &mut CreatureVoice) -> Result<(), String> {
        v.telemetry(&mut self.t);
        if Self::shape_of(&self.t) != self.shape {
            return Err("the creature's telemetry changed shape during the render".into());
        }
        self.push_current();
        Ok(())
    }

    fn push_current(&mut self) {
        let t = &self.t;
        let call = t.call.and_then(|id| self.def.call_index(id)).map_or(-1.0, |i| i as f64);
        let d = &mut self.data;
        d.extend([t.gate, t.out_rms, t.clock, call].map(|x| x as f32));
        d.extend(t.params.iter().map(|(_, x)| *x as f32));
        for tb in &t.tubes {
            d.extend_from_slice(&tb.areas);
            d.extend_from_slice(&tb.rms);
        }
        d.extend(t.source.iter().map(|(_, x)| *x as f32));
        self.frames += 1;
        debug_assert_eq!(self.data.len(), self.frames as usize * self.frame_len());
    }

    pub fn bytes(&self) -> Vec<u8> {
        let mut meta = serde_json::to_vec(&self.meta).unwrap();
        while meta.len() % 4 != 0 {
            meta.push(b' ');
        }
        let mut b = Vec::with_capacity(CTELE_HEADER + meta.len() + 4 * self.data.len());
        b.extend_from_slice(b"CHCT");
        for x in [CTELE_VERSION, FS as u32, TELE_HOP as u32, self.frames, self.frame_len() as u32, meta.len() as u32] {
            b.extend_from_slice(&x.to_le_bytes());
        }
        b.extend_from_slice(&meta);
        for x in &self.data {
            b.extend_from_slice(&x.to_le_bytes());
        }
        b
    }
}

/// `render creature-show <script.json> <out> [--tele show.ctele]`
pub fn creature_show(ctx: &Ctx, a: &[String]) -> Result<(), String> {
    let mut tele_out: Option<String> = None;
    let mut rest = Vec::new();
    let mut it = a.iter();
    while let Some(x) = it.next() {
        if x == "--tele" {
            tele_out = Some(it.next().ok_or("--tele needs a file")?.clone());
        } else {
            rest.push(x.clone());
        }
    }
    let input = rest.first().ok_or("render creature-show <script.json> <out> [--tele show.ctele]")?;
    let out_name = wav_name(arg(&rest, 1, "out/creature-show"));
    let text = std::fs::read_to_string(input).map_err(|e| format!("{input}: {e}"))?;
    let script = Script::parse(&text).map_err(|e| format!("{input}: {e}"))?;
    let (out, rec) = render_script(&script, tele_out.is_some())?;
    let bad = out.iter().filter(|x| !x.is_finite()).count();
    ctx.write_wav(&out_name, &out)?;
    println!("wrote {out_name}  {} s  peak {}  non-finite {bad}", fixed(out.len() as f64 / FS, 2), fixed(peak(&out), 3));
    for c in script.cues() {
        let seg = &out[(js_round(c.at * FS) as usize).min(out.len())..(js_round(c.end * FS) as usize).min(out.len())];
        println!("  {:>6}–{:<6} {:<16} {}", fixed(c.at, 2), fixed(c.end, 2), c.name, measure(seg, FS));
    }
    if let (Some(path), Some(r)) = (tele_out, rec) {
        std::fs::write(&path, r.bytes()).map_err(|e| format!("{path}: {e}"))?;
        println!("wrote {path}  {} frames", r.frames);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `--tele` file read back: the metadata and each frame's values.
    fn read(b: &[u8]) -> (Value, Vec<Vec<f32>>) {
        assert_eq!(&b[..4], b"CHCT");
        let u = |i: usize| u32::from_le_bytes(b[4 + 4 * i..8 + 4 * i].try_into().unwrap()) as usize;
        let (version, rate, hop, frames, len, meta_len) = (u(0), u(1), u(2), u(3), u(4), u(5));
        assert_eq!((version, rate, hop), (CTELE_VERSION as usize, 48000, TELE_HOP));
        assert_eq!(meta_len % 4, 0);
        let meta: Value = serde_json::from_slice(&b[CTELE_HEADER..CTELE_HEADER + meta_len]).unwrap();
        let data = &b[CTELE_HEADER + meta_len..];
        assert_eq!(data.len(), 4 * frames * len);
        let f: Vec<f32> = data.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect();
        (meta, f.chunks(len).map(<[f32]>::to_vec).collect())
    }

    /// The file holds exactly what the creature's telemetry said, snapshot by snapshot, when the
    /// voice is driven by hand the way the script says (a call at 0.1 s), and the sound is that
    /// voice's sound.
    #[test]
    fn creature_tele_round_trips_the_engine_telemetry() {
        let s = Script::parse(r#"{"creature": "flatulence", "params": {"spread": 0.4}, "tail": 0.2, "events": [{"at": 0.1, "call": "squeak"}]}"#).unwrap();
        let (out, rec) = render_script(&s, true).unwrap();
        let (meta, frames) = read(&rec.unwrap().bytes());

        let def = creature_by_id("flatulence").unwrap();
        let mut v = CreatureVoice::new(def, FS, &[(def.param_index("spread").unwrap(), 0.4)]);
        let mut t = v.new_telemetry();
        let mut snaps = Vec::new();
        v.telemetry(&mut t);
        snaps.push(t.clone());
        let mut want = vec![0.0f32; out.len()];
        // in blocks of another size than the script's ticks (the sound doesn't depend on it)
        for (j, block) in want.chunks_mut(96).enumerate() {
            if j * 96 == 4800 {
                v.play_call_id("squeak", None, 1.0);
            }
            v.render(block);
            if ((j + 1) * 96) % TELE_HOP == 0 {
                v.telemetry(&mut t);
                snaps.push(t.clone());
            }
        }
        assert!(out == want, "the script's sound differs from the voice's");
        // the squeak (1.9 s) and the tail
        assert_eq!(out.len(), 48 * 2200);
        assert_eq!(frames.len(), 1 + out.len() / TELE_HOP);
        assert_eq!(frames.len(), snaps.len());

        let names = |k: &str| meta[k].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect::<Vec<_>>();
        assert_eq!(meta["id"], "flatulence");
        assert_eq!(names("params"), def.params.iter().map(|p| p.key).collect::<Vec<_>>());
        assert_eq!(names("calls"), def.calls.iter().map(|c| c.id).collect::<Vec<_>>());
        assert_eq!(names("sources"), snaps[0].source.iter().map(|(k, _)| *k).collect::<Vec<_>>());
        let tubes: Vec<(String, usize)> = meta["tubes"].as_array().unwrap().iter().map(|t| (t["name"].as_str().unwrap().into(), t["n"].as_u64().unwrap() as usize)).collect();
        assert_eq!(tubes, snaps[0].tubes.iter().map(|t| (t.name.to_string(), t.areas.len())).collect::<Vec<_>>());
        assert_eq!(meta["cues"], json!([{ "at": 0.1, "end": 2.0, "name": "Squeak", "hint": def.calls[0].hint }]));

        let mut calling = 0;
        for (i, (f, t)) in frames.iter().zip(&snaps).enumerate() {
            let call = t.call.map_or(-1.0, |id| def.call_index(id).unwrap() as f32);
            calling += (call >= 0.0) as usize;
            let mut want = vec![t.gate as f32, t.out_rms as f32, t.clock as f32, call];
            want.extend(t.params.iter().map(|(_, x)| *x as f32));
            for tb in &t.tubes {
                want.extend(&tb.areas);
                want.extend(&tb.rms);
            }
            want.extend(t.source.iter().map(|(_, x)| *x as f32));
            assert_eq!(f, &want, "frame {i}");
        }
        // the call from 0.1 s until 50 ms after its last key (1.9 s): 1.95 s, 122 frames
        assert!((120..=124).contains(&calling), "{calling} frames calling");
        assert!(frames.iter().any(|f| f[1] > 0.01), "silent");
    }

    #[test]
    fn creature_show_render_is_deterministic_and_telemetry_leaves_the_sound_alone() {
        let text = r#"{"creature": "flatulence", "tail": 0.1, "events": [
            {"at": 0.05, "hold": 0.4, "params": {"gas": 120, "eas": 10, "push": "0:15,0.3:40"}, "caption": "Held"},
            {"at": 0.6, "call": "staccato"}]}"#;
        let s = Script::parse(text).unwrap();
        let (a, ra) = render_script(&s, true).unwrap();
        let (b, rb) = render_script(&Script::parse(text).unwrap(), true).unwrap();
        let (c, _) = render_script(&s, false).unwrap();
        assert!(a == b && a == c);
        assert!(ra.unwrap().bytes() == rb.unwrap().bytes());
        let cues = s.cues();
        assert_eq!(cues[0].name, "Held");
        assert!((cues[0].end - 1.05).abs() < 1e-9, "{}", cues[0].end);
        assert_eq!(cues[1].name, "Staccato");
        assert!(a.iter().all(|x| x.is_finite()) && peak(&a) > 0.05);
    }

    #[test]
    fn creature_show_scripts_refuse_overlaps_and_unknown_names() {
        let p = |t: &str| Script::parse(t).err().unwrap_or_default();
        assert!(p(r#"{"creature": "flatulence", "events": [{"at": 0, "call": "squeak"}, {"at": 1, "call": "rumble"}]}"#).contains("starts before"));
        assert!(p(r#"{"creature": "flatulence", "events": [{"at": 0, "call": "bark"}]}"#).contains("no call bark"));
        assert!(p(r#"{"creature": "flatulence", "events": [{"at": 0, "hold": 1, "params": {"volume": 3}}]}"#).contains("no parameter volume"));
        assert!(p(r#"{"creature": "dragon", "events": []}"#).contains("unknown creature"));
    }
}
