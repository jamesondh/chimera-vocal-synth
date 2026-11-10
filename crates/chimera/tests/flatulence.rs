//! The flatulence creature (docs/flatulence.md): a golden render that must not depend on the block
//! size, and the physics it is meant to show — continence, the event ending when the gas is gone,
//! gas conservation, pitch rising with the external sphincter's contraction and falling with body
//! size, a relaxed sphincter playing low, a clench cutting the flow off no faster than muscle
//! activates, the rectoanal reflex opening the internal sphincter's zone with distension, more gas
//! lasting longer, pressed cheeks flapping as a second valve and sputtering near threshold, and
//! mucus roughening the sound or holding the gas.
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
    let (out, t) = hold(&[("eas", 80.0), ("push", 30.0), ("gas", 100.0)], 1.0);
    assert!(rms(&out) < 1e-5, "leaked: rms {}", rms(&out));
    assert!(source(&t, "expelled").abs() < 0.1);
    assert!(source(&t, "gas") > 95.0);
}

#[test]
fn the_event_ends_when_the_gas_is_gone_and_conserves_it() {
    let (out, t) = hold(&[("eas", 20.0), ("push", 30.0), ("gas", 100.0)], 2.0);
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
fn a_relaxed_sphincter_holds_a_clean_low_tone_and_a_contracted_one_plays_higher() {
    // the lip's stiffness is the relaxed canal's (measured compliance) plus the contracting
    // muscle's short-range stiffness. Relaxed, just opened by the gas, it holds a clean tone below
    // the old model's 160 Hz floor (the recordings' clean tones are at 93–142 Hz); contracted, it
    // plays higher. Internal sphincter relaxed and cheeks spread, so the ring is the closure.
    let (relaxed, _) = hold(&[("eas", 5.0), ("push", 4.0), ("ias", 0.0), ("spread", 1.0)], 2.5);
    let lax = f0(&relaxed);
    let periodic = periodic_fraction(&relaxed);
    assert!(lax < 140.0 && periodic > 0.8, "relaxed: {lax} Hz, {periodic} periodic");
    let tight = f0(&hold(&[("eas", 40.0), ("push", 31.0), ("ias", 0.0), ("spread", 1.0)], 2.0).0);
    assert!(tight > lax * 1.25, "relaxed (5 cmH₂O): {lax} Hz, contracted (40): {tight} Hz");
}

#[test]
fn pitch_falls_with_size() {
    let kv = |s: f64| [("eas", 40.0), ("push", 31.0), ("ias", 0.0), ("spread", 1.0), ("size", s)];
    let small = f0(&hold(&kv(0.7), 2.0).0);
    let large = f0(&hold(&kv(1.2), 2.0).0);
    assert!(small > large * 1.3, "size 0.7: {small} Hz, size 1.2: {large} Hz");
}

#[test]
fn more_gas_lasts_longer() {
    // (internal sphincter relaxed: at 30 mL the reflex leaves a third of its tone, which holds)
    let little = duration(&hold(&[("gas", 30.0), ("ias", 0.0)], 2.0).0);
    let lots = duration(&hold(&[("gas", 250.0), ("ias", 0.0)], 3.0).0);
    assert!(lots > little * 1.5, "30 mL: {little} s, 250 mL: {lots} s");
}

/// Fraction of the loud 40 ms frames that YIN finds periodic (aperiodicity < 0.3).
fn periodic_fraction(x: &[f32]) -> f64 {
    let w: Vec<f64> = x.chunks(480).map(rms).collect();
    let top = w.iter().cloned().fold(0.0, f64::max);
    let (mut n, mut p) = (0.0, 0.0);
    for (i, e) in w.iter().enumerate() {
        if *e > top * 0.1 && (i + 4) * 480 <= x.len() {
            let fr: Vec<f64> = x[i * 480..(i + 4) * 480].iter().map(|&v| v as f64).collect();
            let y = yin_pitch(&fr, FS, YinOpts { min_f0: Some(40.0), max_f0: Some(1500.0), ..Default::default() });
            n += 1.0;
            if y.f0 > 0.0 && y.aperiodicity < 0.3 {
                p += 1.0;
            }
        }
    }
    p / n
}

/// The cheeks' gap (cm) over a held event, sampled every block: smallest, largest, and how many
/// times it fell shut after being open.
fn cheek_gaps(kv: &[(&str, f64)], sec: f64) -> (f64, f64, usize) {
    let mut v = CreatureVoice::new(def(), FS, &params(kv));
    let mut t = v.new_telemetry();
    v.set_gate(1.0);
    let mut buf = vec![0.0f32; 128];
    let (mut lo, mut hi, mut shut, mut was_open) = (f64::INFINITY, 0.0f64, 0, false);
    for _ in 0..(sec * FS / 128.0) as usize {
        v.render(&mut buf);
        v.telemetry(&mut t);
        let g = source(&t, "cheeks");
        lo = lo.min(g);
        hi = hi.max(g);
        if g > 0.0 {
            was_open = true;
        } else if was_open {
            shut += 1;
            was_open = false;
        }
    }
    (lo, hi, shut)
}

#[test]
fn pressed_cheeks_flap_and_spread_cheeks_stay_open() {
    // the cheeks are a second valve: pressed, they touch; the gas parts them and they fall shut
    // behind each puff. Spread wide they stay open whatever the flow.
    let (_, hi, shut) = cheek_gaps(&[("spread", 0.0)], 1.0);
    assert!(hi > 0.05 && shut >= 3, "pressed cheeks: opened to {hi} cm, fell shut {shut} times");
    let (lo, _, _) = cheek_gaps(&[("spread", 1.0)], 1.0);
    assert!(lo > 0.5, "spread cheeks closed to {lo} cm");
}

#[test]
fn just_above_threshold_pressed_cheeks_sputter() {
    // a push only just able to part the sphincter and the cheeks lets the gas out in puffs, each
    // ended by the cheeks falling shut: seconds long, dozens of puffs a second
    let kv = [("spread", 0.0), ("eas", 10.0), ("push", 8.0), ("gas", 100.0), ("ias", 0.0)];
    let (out, _) = hold(&kv, 8.0);
    let d = duration(&out);
    let (_, _, shut) = cheek_gaps(&kv, d);
    assert!(d > 3.0 && shut as f64 > 15.0 * d, "{d} s, {shut} puffs");
}

#[test]
fn a_wet_canal_is_rougher_and_still_passes_its_gas() {
    // mucus bridges seal the gap after every contact and let go all at once past 0.27 mm: the
    // openings come irregularly, and the gas still leaves (most of it: once the emptying rectum's
    // recoil has fallen, the bridge holds the rest; what left is what the pocket lost)
    // (cheeks spread, so that the puffs are the mucus's)
    let kv = [("eas", 5.0), ("push", 8.0), ("gas", 100.0), ("spread", 1.0), ("ias", 0.0)];
    let dry = periodic_fraction(&hold(&kv, 3.0).0);
    let (out, t) = hold(&[kv[0], kv[1], kv[2], kv[3], kv[4], ("mucus", 1.0)], 4.0);
    let wet = periodic_fraction(&out);
    assert!(wet < dry - 0.1, "periodic: dry {dry}, wet {wet}");
    let gas = source(&t, "gas");
    assert!(gas < 50.0 && (source(&t, "expelled") - (100.0 - gas)).abs() < 8.0, "gas left {gas}, expelled {}", source(&t, "expelled"));
}

#[test]
fn a_wet_seal_holds_what_a_dry_canal_lets_go() {
    // push 30 against an external sphincter at 20 passes the gas dry; the strongest mucus bridge (3 kPa ≈ 31 cmH₂O)
    // holds the 22 cmH₂O left over
    let (out, t) = hold(&[("eas", 20.0), ("push", 30.0), ("gas", 100.0), ("mucus", 3.0)], 1.0);
    assert!(rms(&out) < 1e-5, "leaked: rms {}", rms(&out));
    assert!(source(&t, "gas") > 95.0);
}

/// Held event with the given controls for `before` s, then `change` applied, then `after` s more;
/// returns per-block (time, telemetry source values) after the change.
fn change_mid_event(kv: &[(&str, f64)], before: f64, change: &[(&str, f64)], after: f64) -> Vec<(f64, Vec<(&'static str, f64)>)> {
    let mut v = CreatureVoice::new(def(), FS, &params(kv));
    let mut t = v.new_telemetry();
    v.set_gate(1.0);
    let mut buf = vec![0.0f32; 128];
    for _ in 0..(before * FS / 128.0) as usize {
        v.render(&mut buf);
    }
    v.set_params(&params(change));
    let mut out = Vec::new();
    for i in 0..(after * FS / 128.0) as usize {
        v.render(&mut buf);
        v.telemetry(&mut t);
        out.push(((i + 1) as f64 * 128.0 / FS, t.source.clone()));
    }
    out
}

fn get(s: &[(&'static str, f64)], k: &str) -> f64 {
    s.iter().find(|(n, _)| *n == k).unwrap().1
}

#[test]
fn a_clench_cuts_the_flow_off_no_faster_than_muscle() {
    // plenty of gas, flowing at 60 ms; then the command jumps to a hard squeeze. The sphincter's
    // pressure follows through the activation dynamics (τ 10 ms), so it is not there at once, but
    // within a few time constants it shuts the ring and the flow stops with gas left.
    let kv = [("eas", 20.0), ("push", 30.0), ("gas", 300.0), ("spread", 1.0)];
    let trace = change_mid_event(&kv, 0.06, &[("eas", 200.0)], 0.3);
    let first = &trace[0].1;
    assert!(get(first, "flow") > 50.0, "not flowing before the clench: {} mL/s", get(first, "flow"));
    // after one block (2.7 ms) the pressure has risen by well under its whole jump
    assert!(get(first, "eas") < 5.0 + 0.4 * 195.0, "the sphincter jumped: {} cmH₂O after 2.7 ms", get(first, "eas"));
    let shut = trace.iter().find(|(_, s)| get(s, "gap") <= 0.0 && get(s, "flow").abs() < 5.0).map(|(t, _)| *t);
    let shut = shut.expect("the clench never stopped the flow");
    assert!(shut > 0.004 && shut < 0.06, "flow stopped {shut} s after the clench");
    // and stays stopped
    assert!(trace.iter().filter(|(t, _)| *t > shut + 0.03).all(|(_, s)| get(s, "gap") <= 0.0), "reopened after the clench");
    let left = get(&trace[trace.len() - 1].1, "gas");
    assert!(left > 50.0, "only {left} mL held back");
}

#[test]
fn rectal_distension_opens_the_internal_sphincter_zone() {
    // the rectoanal reflex: the same rectal pressure (push + the wall's recoil = 22 cmH₂O, the
    // external sphincter relaxed) passes gas from a distended rectum (150 mL: the internal
    // sphincter inhibited) and not from a nearly empty one (20 mL: half its tone left)
    let recoil = |v: f64| v * 1333.22 / 11.0 / 980.665;
    let full = |sec| hold(&[("eas", 0.0), ("gas", 150.0), ("push", 22.0 - recoil(150.0))], sec).1;
    let low = hold(&[("eas", 0.0), ("gas", 20.0), ("push", 22.0 - recoil(20.0))], 1.0).1;
    assert!(source(&full(1.0), "expelled") > 30.0, "150 mL: expelled {}", source(&full(1.0), "expelled"));
    assert!(source(&low, "expelled") < 0.5, "20 mL: expelled {}", source(&low, "expelled"));
    // the internal sphincter's pressure at the start: nearly all inhibited at 150 mL, half at 20
    let early = full(0.02);
    assert!(source(&early, "ias") < 1.0 && source(&low, "ias") > 15.0, "internal sphincter: {} at 150 mL, {} at 20 mL (cmH₂O)", source(&early, "ias"), source(&low, "ias"));
}
