//! Real-time factor of the voices (single core): `cargo run --release -p chimera --example bench`.
//! The last line is several voices at once, as a chord needs them: `-- 4` for four (default 6).
use chimera::anatomy::human::BODY_MALE;
use chimera::control::score::Gesture;
use chimera::voice::human_voice::{HumanVoice, SourcePatch};
use std::time::Instant;

/// A male voice that says the golden test's sentence, starting `at` seconds in.
fn speaker(fs: f64, os: usize, gs: &[Gesture], at: f64) -> HumanVoice {
    let mut voice = HumanVoice::new(fs, BODY_MALE, os);
    voice.set_source(&SourcePatch::f0(110.0));
    let mut g2 = gs.to_vec();
    for g in g2.iter_mut() {
        g.t0 += at;
        g.t1 += at;
    }
    voice.schedule(&g2);
    voice.set_gate(1.0);
    voice
}

fn main() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"));
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(format!("{root}/tests/golden/voice.json")).unwrap()).unwrap();
    let s = v.as_array().unwrap().iter().find(|s| s["name"] == "speech-male").unwrap();
    let gs: Vec<Gesture> = serde_json::from_value(s["ops"][0]["gestures"].clone()).unwrap();
    let fs = 48000.0;
    let secs = 4.0;
    let n = (fs * secs) as usize;
    for os in [2usize, 1] {
        let mut voice = speaker(fs, os, &gs, 0.03);
        let mut out = vec![0.0f32; 128];
        let t0 = Instant::now();
        for _ in 0..n / 128 {
            voice.render(&mut out);
        }
        let dt = t0.elapsed().as_secs_f64();
        println!("human male os={os}: {:.1}% of one core in real time ({:.2} ms per 128-sample block)", 100.0 * dt / secs, 1000.0 * dt / (n / 128) as f64);
    }

    // Several voices on one thread, each a whole body, rendered in turn a block at a time and summed.
    // Each says the sentence a little later than the one before, so all of them are moving, at
    // different places in it: the most a chord can cost. The best of five runs, and a hash of the
    // sum (FNV-1a over the samples' bits), which a change that keeps the sound must not move.
    let count: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(6);
    let (mut best, mut hash) = (f64::INFINITY, 0u64);
    for _ in 0..5 {
        let mut voices: Vec<HumanVoice> = (0..count).map(|k| speaker(fs, 2, &gs, 0.03 + 0.21 * k as f64)).collect();
        let (mut out, mut sum) = (vec![0.0f32; 128], vec![0.0f32; 128]);
        hash = 0xcbf29ce484222325;
        let t0 = Instant::now();
        for _ in 0..n / 128 {
            sum.fill(0.0);
            for voice in voices.iter_mut() {
                voice.render(&mut out);
                for (s, x) in sum.iter_mut().zip(&out) {
                    *s += *x;
                }
            }
            for b in sum.iter().flat_map(|x| x.to_bits().to_le_bytes()) {
                hash = (hash ^ b as u64).wrapping_mul(0x100000001b3);
            }
        }
        best = best.min(t0.elapsed().as_secs_f64());
    }
    println!("{count} voices os=2: {:.1}% of one core in real time ({:.1}% a voice), sum 0x{hash:016x}", 100.0 * best / secs, 100.0 * best / secs / count as f64);
}
