//! The domestic cat (`creatures/cat.rs`): it holds at any device rate, its purr sounds on the
//! in-breath as well as the out-breath at the purr rate, and its meow opens and closes.
use chimera::analysis::fft::{Fft, hann, power_spectrum_db};
use chimera::creatures::{CreatureVoice, creature_by_id};

fn cat(fs: f64) -> CreatureVoice {
    chimera::creatures::init();
    CreatureVoice::new(creature_by_id("cat").expect("the cat is registered"), fs, &[])
}

fn render(v: &mut CreatureVoice, seconds: f64) -> Vec<f32> {
    let mut out = vec![0.0f32; (seconds * v.fs_out) as usize];
    for chunk in out.chunks_mut(128) {
        v.render(chunk);
    }
    out
}

fn rms(x: &[f32]) -> f64 {
    (x.iter().map(|&s| (s as f64).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
}

fn call_rms(fs: f64, call: &str) -> f64 {
    let mut v = cat(fs);
    v.play_call_id(call, None, 1.0);
    let out = render(&mut v, 1.2);
    assert!(out.iter().all(|x| x.is_finite()), "{call} at {fs} Hz went NaN");
    rms(&out)
}

#[test]
fn the_cat_holds_at_any_device_rate() {
    // The cat's pharynx is 2 cm: at 2x a 16 kHz headset (32 kHz) its sections would be shorter than
    // the stable minimum and the airway would blow up. min_rate raises the oversampling instead.
    for fs in [8000.0, 16000.0, 22050.0, 32000.0, 44100.0, 96000.0] {
        assert!(cat(fs).fs >= 64000.0, "internal rate {} at {fs} Hz", cat(fs).fs);
    }
    for call in ["meow", "purr", "hiss"] {
        let reference = call_rms(48000.0, call);
        assert!(reference > 1e-3, "{call} is silent at 48 kHz");
        for fs in [8000.0, 16000.0, 32000.0, 44100.0, 96000.0] {
            let r = call_rms(fs, call);
            assert!(r > 0.5 * reference && r < 2.0 * reference, "{call} at {fs} Hz: RMS {r:.4}, 48 kHz {reference:.4}");
        }
    }
}

/// The strongest envelope periodicity between 12 and 60 Hz (the shortest lag near the top).
fn pulse_rate(x: &[f32], fs: f64) -> f64 {
    let d = 48;
    let env: Vec<f64> = x.chunks(d).map(|c| c.iter().map(|s| s.abs() as f64).sum()).collect();
    let mean = env.iter().sum::<f64>() / env.len() as f64;
    let env: Vec<f64> = env.iter().map(|e| e - mean).collect();
    let efs = fs / d as f64;
    let ac = |l: usize| (0..env.len() - l).map(|i| env[i] * env[i + l]).sum::<f64>() / (env.len() - l) as f64;
    let lags: Vec<usize> = ((efs / 60.0) as usize..(efs / 12.0) as usize).collect();
    let top = lags.iter().map(|&l| ac(l)).fold(f64::MIN, f64::max);
    let best = lags.iter().copied().find(|&l| ac(l) >= 0.85 * top && ac(l) >= ac(l - 1) && ac(l) >= ac(l + 1)).expect("no envelope periodicity between 12 and 60 Hz");
    efs / best as f64
}

#[test]
fn the_purr_sounds_breathing_in_and_out() {
    let fs = 48000.0;
    let mut v = cat(fs);
    v.play_call_id("purr", None, 1.0);
    let out = render(&mut v, 5.0);
    let seg = |a: f64, b: f64| &out[(a * fs) as usize..(b * fs) as usize];
    // four breaths: out for 585 ms, then in for 652 ms (the means of four cats, Schötz & Eklund
    // 2011); away from the turns
    let cycle = 0.585 + 0.652;
    let (mut re, mut ri) = (0.0, 0.0);
    for c in 0..4 {
        let t = c as f64 * cycle;
        let (exhale, inhale) = (seg(t + 0.1, t + 0.5), seg(t + 0.69, t + 1.2));
        re += rms(exhale) / 4.0;
        ri += rms(inhale) / 4.0;
        for (name, s) in [("out", exhale), ("in", inhale)] {
            let rate = pulse_rate(s, fs);
            assert!((rate - 25.0).abs() < 1.5, "breath {c} {name}: purr pulses at {rate:.1} Hz, set 25");
        }
    }
    // the four cats purred 1.9 dB quieter breathing in, on average (0 to 5.8 dB)
    let db = 20.0 * (ri / re).log10();
    assert!((-4.0..=0.0).contains(&db), "in-breath {db:.1} dB against the out-breath, cats -1.9");
}

#[test]
fn breathing_in_without_the_purr_is_nearly_silent() {
    // The ordinary vocal folds only vibrate on positive pressure: the in-breath sound is the
    // purr valve's, not the folds'.
    let fs = 48000.0;
    let def = creature_by_id("cat").unwrap();
    let held = |purr: f64| {
        let mut v = cat(fs);
        let set = [("breath", -0.8), ("purr", purr), ("pressure", 9.0), ("mouth", 0.0)];
        v.set_params(&set.map(|(k, x)| (def.param_index(k).unwrap(), x)));
        v.set_gate(1.0);
        let out = render(&mut v, 1.0);
        rms(&out[(0.3 * fs) as usize..])
    };
    let (folds, purr) = (held(0.0), held(1.0));
    assert!(folds < 0.1 * purr, "in-breath RMS {folds:.5} with the folds, {purr:.5} purring");
}

/// Spectral centroid (Hz) of a 2048-sample frame starting at `t`.
fn centroid(x: &[f32], fs: f64, t: f64) -> f64 {
    let n = 2048;
    let fft = Fft::new(n);
    let s = (t * fs) as usize;
    let frame: Vec<f64> = x[s..s + n].iter().map(|&v| v as f64).collect();
    let db = power_spectrum_db(&frame, &fft, &hann(n));
    let (mut num, mut den) = (0.0, 0.0);
    for (k, d) in db.iter().enumerate() {
        let p = 10f64.powf(d / 10.0);
        num += p * k as f64 * fs / n as f64;
        den += p;
    }
    num / den
}

#[test]
fn the_meow_opens_and_closes() {
    // i-a-u: the jaw opens (brighter) and closes again (darker)
    let fs = 48000.0;
    let mut v = cat(fs);
    v.play_call_id("meow", None, 1.0);
    let out = render(&mut v, 0.9);
    let (start, open, end) = (centroid(&out, fs, 0.02), centroid(&out, fs, 0.18), centroid(&out, fs, 0.68));
    assert!(open > 1.3 * start && open > 1.3 * end, "meow centroid {start:.0} → {open:.0} → {end:.0} Hz");
}

#[test]
fn the_hiss_is_centred_like_real_hisses() {
    // Four cats' hisses: centre of gravity 820-1186 Hz, mean 1105 Hz (Schötz 2015)
    let fs = 48000.0;
    let mut v = cat(fs);
    v.play_call_id("hiss", None, 1.0);
    let out = render(&mut v, 0.65);
    let c = centroid_of(&out[(0.05 * fs) as usize..(0.6 * fs) as usize], fs);
    assert!((800.0..1400.0).contains(&c), "hiss centre of gravity {c:.0} Hz, cats 820-1186");
}

/// Spectral centroid (Hz) of a whole segment.
fn centroid_of(x: &[f32], fs: f64) -> f64 {
    let n = x.len().next_power_of_two() / 2;
    let fft = Fft::new(n);
    let frame: Vec<f64> = x[..n].iter().map(|&v| v as f64).collect();
    let db = power_spectrum_db(&frame, &fft, &hann(n));
    let (mut num, mut den) = (0.0, 0.0);
    for (k, d) in db.iter().enumerate() {
        let p = 10f64.powf(d / 10.0);
        num += p * k as f64 * fs / n as f64;
        den += p;
    }
    num / den
}

#[test]
fn the_cats_telemetry_keeps_the_ui_key_order() {
    // The TS-era fixture checks this for the creatures before the cat (golden_creatures.rs); the
    // UI reads params in the def's order.
    let mut v = cat(48000.0);
    let mut t = v.new_telemetry();
    v.telemetry(&mut t);
    let text = serde_json::to_string(&t).unwrap();
    let def = creature_by_id("cat").unwrap();
    let mut keys: Vec<&str> = vec!["kind", "id", "params", "gate", "outRms", "clock", "tubes", "source", "call"];
    let pos = |k: &str| text.find(&format!("\"{k}\":")).unwrap_or_else(|| panic!("no key {k}"));
    assert!(keys.windows(2).all(|w| pos(w[0]) < pos(w[1])), "telemetry key order {keys:?}");
    keys = def.params.iter().map(|p| p.key).collect();
    assert!(keys.windows(2).all(|w| pos(w[0]) < pos(w[1])), "param key order {keys:?}");
}
