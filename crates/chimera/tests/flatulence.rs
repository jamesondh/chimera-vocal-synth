//! The flatulence creature (docs/flatulence.md): a golden render that must not depend on the block
//! size, and the physics it is meant to show — continence, the event ending when the gas is gone,
//! gas conservation, pitch rising with sphincter tone, falling with body size, more gas lasting
//! longer, and pressed cheeks swinging as a second valve.
mod common;
use chimera::analysis::pitch::{YinOpts, yin_pitch};
use chimera::creatures::{CreatureDef, CreatureTelemetry, CreatureVoice, creature_by_id};
use common::*;

const FS: f64 = 48000.0;

fn def() -> &'static CreatureDef {
    creature_by_id("flatulence").expect("flatulence creature")
}

fn params(kv: &[(&str, f64)]) -> Vec<(usize, f64)> {
    kv.iter().map(|(k, v)| (def().param_index(k).unwrap(), *v)).collect()
}

/// Held event with the given controls, rendered for `sec`; returns the samples and the last telemetry.
fn hold(kv: &[(&str, f64)], sec: f64) -> (Vec<f32>, CreatureTelemetry) {
    let mut v = CreatureVoice::new(def(), FS, &params(kv));
    let mut t = v.new_telemetry();
    v.set_gate(1.0);
    let mut out = vec![0.0f32; (sec * FS) as usize];
    for b in out.chunks_mut(128) {
        v.render(b);
    }
    v.telemetry(&mut t);
    (out, t)
}

fn source(t: &CreatureTelemetry, k: &str) -> f64 {
    t.source.iter().find(|(n, _)| *n == k).unwrap().1
}

fn rms(x: &[f32]) -> f64 {
    (x.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / x.len().max(1) as f64).sqrt()
}

/// Seconds between the first and last 10 ms window within 30 dB of the loudest.
fn duration(x: &[f32]) -> f64 {
    let w: Vec<f64> = x.chunks(480).map(rms).collect();
    let top = w.iter().cloned().fold(0.0, f64::max);
    let on: Vec<usize> = (0..w.len()).filter(|&i| w[i] > top * 0.0316).collect();
    (on[on.len() - 1] + 1 - on[0]) as f64 * 0.01
}

/// Median YIN f0 over the loud part.
fn f0(x: &[f32]) -> f64 {
    let w: Vec<f64> = x.chunks(480).map(rms).collect();
    let top = w.iter().cloned().fold(0.0, f64::max);
    let mut f = Vec::new();
    for (i, e) in w.iter().enumerate() {
        if *e > top * 0.3 && (i + 4) * 480 <= x.len() {
            let fr: Vec<f64> = x[i * 480..(i + 4) * 480].iter().map(|&v| v as f64).collect();
            let p = yin_pitch(&fr, FS, YinOpts { min_f0: Some(40.0), max_f0: Some(1500.0), ..Default::default() });
            if p.f0 > 0.0 && p.aperiodicity < 0.3 {
                f.push(p.f0);
            }
        }
    }
    assert!(!f.is_empty(), "no periodic frames");
    f.sort_by(|a, b| a.partial_cmp(b).unwrap());
    f[f.len() / 2]
}

/// The golden session: a held event, a pause, then the squeak call.
fn session(block: usize) -> Vec<f32> {
    let mut v = CreatureVoice::new(def(), FS, &[]);
    let n = (1.9 * FS) as usize;
    let gate_off = (0.7 * FS) as usize;
    let call_at = (0.9 * FS) as usize;
    let mut out = vec![0.0f32; n];
    v.set_gate(1.0);
    let mut i = 0;
    while i < n {
        let next = [gate_off, call_at, n].into_iter().find(|&e| e > i).unwrap();
        let end = next.min(i + block);
        v.render(&mut out[i..end]);
        i = end;
        if i == gate_off {
            v.set_gate(0.0);
        }
        if i == call_at {
            v.play_call_id("squeak", None, 1.0);
        }
    }
    out
}

#[test]
fn golden_session_in_any_block_size() {
    let got = session(128);
    assert!(got.iter().all(|x| x.is_finite()));
    if blessing() {
        save_f32("flatulence", &got);
    } else {
        let reference = load_f32("flatulence");
        let d = compare_audio(&reference, &got, 1e-6, 480);
        println!("flatulence: peak {:.3} max err {:.2e} first >1e-6 at {:?}", d.peak, d.max_err, d.first_diverge);
        assert!(d.first_diverge.is_none(), "flatulence: renders diverge from sample {:?} (max error {:.2e} of peak)", d.first_diverge, d.max_err);
    }
    for block in [1usize, 77, 4096] {
        assert!(session(block) == got, "block size {block} changed the output");
    }
}

#[test]
fn a_closed_sphincter_holds_the_gas() {
    // continence: a closing pressure above the push and the rectal recoil keeps the canal shut
    let (out, t) = hold(&[("tone", 80.0), ("push", 30.0), ("gas", 100.0)], 1.0);
    assert!(rms(&out) < 1e-5, "leaked: rms {}", rms(&out));
    assert!(source(&t, "expelled").abs() < 0.1);
    assert!(source(&t, "gas") > 95.0);
}

#[test]
fn the_event_ends_when_the_gas_is_gone_and_conserves_it() {
    let (out, t) = hold(&[("tone", 20.0), ("push", 30.0), ("gas", 100.0)], 2.0);
    let n = out.len();
    let early = rms(&out[..n / 4]);
    let late = rms(&out[n * 3 / 4..]);
    assert!(early > 1e-3, "no sound: {early}");
    assert!(late < early * 1e-3, "still sounding at the end: {late} vs {early}");
    // the pocket is at its floor, and what left through the anus is what the pocket lost (the
    // gas is compressed by at most ~5 %)
    let gas = source(&t, "gas");
    let expelled = source(&t, "expelled");
    assert!(gas < 3.0, "gas left {gas}");
    assert!((expelled - (100.0 - gas)).abs() < 8.0, "expelled {expelled} of {}", 100.0 - gas);
}

#[test]
fn pitch_rises_with_tone_and_falls_with_size() {
    let lax = f0(&hold(&[("tone", 10.0), ("push", 25.0)], 1.5).0);
    let tight = f0(&hold(&[("tone", 50.0), ("push", 65.0)], 1.5).0);
    assert!(tight > lax * 1.3, "tone 10: {lax} Hz, tone 50: {tight} Hz");
    let small = f0(&hold(&[("size", 0.7)], 1.5).0);
    let large = f0(&hold(&[("size", 1.2)], 1.5).0);
    assert!(small > large * 1.3, "size 0.7: {small} Hz, size 1.2: {large} Hz");
}

#[test]
fn more_gas_lasts_longer() {
    let little = duration(&hold(&[("gas", 30.0)], 2.0).0);
    let lots = duration(&hold(&[("gas", 250.0)], 3.0).0);
    assert!(lots > little * 1.5, "30 mL: {little} s, 250 mL: {lots} s");
}

/// The cheeks' gap (cm) over a held event, sampled every block: (smallest, largest).
fn cheek_gap_range(kv: &[(&str, f64)], sec: f64) -> (f64, f64) {
    let mut v = CreatureVoice::new(def(), FS, &params(kv));
    let mut t = v.new_telemetry();
    v.set_gate(1.0);
    let mut buf = vec![0.0f32; 128];
    let (mut lo, mut hi) = (f64::INFINITY, 0.0f64);
    for _ in 0..(sec * FS / 128.0) as usize {
        v.render(&mut buf);
        v.telemetry(&mut t);
        let g = source(&t, "cheeks");
        lo = lo.min(g);
        hi = hi.max(g);
    }
    (lo, hi)
}

#[test]
fn pressed_cheeks_are_parted_by_the_gas_and_spread_cheeks_stay_open() {
    // the cheeks are a second valve: pressed (1 mm apart at rest) the gas parts them by half as
    // much again and they swing back past their rest gap (a valve only pushed open would stay
    // above it); spread wide they stay open whatever the flow
    let (lo, hi) = cheek_gap_range(&[("spread", 0.0)], 1.0);
    assert!(lo < 0.098 && hi > 0.14, "pressed cheeks: gap {lo}..{hi} cm");
    let (lo, _) = cheek_gap_range(&[("spread", 1.0)], 1.0);
    assert!(lo > 0.5, "spread cheeks closed to {lo} cm");
}
