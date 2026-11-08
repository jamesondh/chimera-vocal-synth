//! Alien bodies: the human shape changes nothing, every preset and every corner of the ranges
//! sounds (no blow-up, no silence, not pinned to the limiter), and each organ is really there.

use chimera::anatomy::alien::{ALIEN_PRESETS, AlienShape};
use chimera::anatomy::human::{BODY_FEMALE, BODY_MALE, HumanBody};
use chimera::voice::human_voice::{HumanVoice, SourcePatch, VoiceTelemetry};

const FS: f64 = 48000.0;

/// One second of a held voice at 112 Hz; returns the second half and the voicing.
fn hold(body: HumanBody, shape: AlienShape, src: &SourcePatch) -> (Vec<f32>, f64) {
    let mut v = HumanVoice::with_alien(FS, body, shape, 2);
    v.set_source(&SourcePatch::f0(112.0));
    v.set_source(src);
    v.set_gate(1.0);
    let mut out = vec![0f32; FS as usize];
    for c in out.chunks_mut(128) {
        v.render(c);
    }
    let mut t = VoiceTelemetry::default();
    v.telemetry(&mut t);
    (out.split_off(out.len() / 2), t.voicing)
}

fn rms(x: &[f32]) -> f64 {
    (x.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
}

/// Sounding, finite and not held at the limiter (a blown-up airway rides it at ±0.75).
fn assert_sane(what: &str, out: &[f32], voicing: f64) {
    assert!(out.iter().all(|v| v.is_finite()), "{what}: non-finite output");
    let r = rms(out);
    assert!(voicing > 0.5, "{what}: not voicing ({voicing:.2})");
    assert!(r > 0.02, "{what}: too quiet (rms {r:.3})");
    assert!(r < 0.3, "{what}: too loud or unstable (rms {r:.3})");
}

#[test]
fn the_human_shape_is_the_human_voice() {
    for body in [BODY_MALE, BODY_FEMALE] {
        let mut a = HumanVoice::new(FS, body, 2);
        let mut b = HumanVoice::with_alien(FS, body, AlienShape::HUMAN, 2);
        for v in [&mut a, &mut b] {
            v.set_gate(1.0);
        }
        let (mut x, mut y) = (vec![0f32; 9600], vec![0f32; 9600]);
        a.render(&mut x);
        b.render(&mut y);
        assert_eq!(x, y, "{}", body.name);
        assert_eq!(a.airway.net.n_nodes, b.airway.net.n_nodes);
    }
}

#[test]
fn every_preset_sounds() {
    for p in ALIEN_PRESETS {
        let (out, voicing) = hold(BODY_MALE, p.shape, &p.source);
        assert_sane(p.id, &out, voicing);
    }
}

#[test]
fn the_corners_of_the_ranges_stay_stable() {
    let h = AlienShape::HUMAN;
    for size in [0.6, 2.5] {
        for throat in [0.6, 1.8] {
            for lips in [0.5, 2.0] {
                for tongue in [0.85, 1.2] {
                    let s = AlienShape { size, throat, lips, tongue, sac: 1.0, twin: 1.0, crest: 1.0, ..h };
                    let (out, voicing) = hold(BODY_MALE, s, &SourcePatch::default());
                    assert_sane(&format!("{s:?}"), &out, voicing);
                }
            }
        }
    }
}

#[test]
fn each_organ_is_built_only_when_present() {
    let h = AlienShape::HUMAN;
    let n = |s: AlienShape| HumanVoice::with_alien(FS, BODY_MALE, s, 2).airway;
    let human = n(h);
    assert!(human.sac.is_none() && human.twin_port.is_none() && human.crest.is_none());
    assert!(n(AlienShape { sac: 0.5, ..h }).sac.is_some());
    assert!(n(AlienShape { twin: 0.5, ..h }).twin_port.is_some());
    assert!(n(AlienShape { crest: 0.5, ..h }).crest.is_some());
}

#[test]
fn each_organ_changes_the_sound() {
    let h = AlienShape::HUMAN;
    let (base, _) = hold(BODY_MALE, h, &SourcePatch::default());
    for (name, s) in [("sac", AlienShape { sac: 0.8, ..h }), ("twin", AlienShape { twin: 0.8, ..h }), ("crest", AlienShape { crest: 0.8, ..h })] {
        let (out, _) = hold(BODY_MALE, s, &SourcePatch::default());
        let diff: Vec<f32> = out.iter().zip(&base).map(|(a, b)| a - b).collect();
        assert!(rms(&diff) > 0.1 * rms(&base), "{name} barely changes the output");
    }
}

#[test]
fn pitch_transposes_the_voice() {
    let s = AlienShape { pitch: 0.5, ..AlienShape::HUMAN };
    let mut v = HumanVoice::with_alien(FS, BODY_MALE, s, 2);
    v.set_source(&SourcePatch::f0(200.0));
    v.set_gate(1.0);
    let mut out = vec![0f32; 24000];
    v.render(&mut out);
    let mut t = VoiceTelemetry::default();
    v.telemetry(&mut t);
    assert!((t.f0 - 100.0).abs() < 3.0, "f0 {}", t.f0);
}

#[test]
fn out_of_range_shapes_are_clamped() {
    let s = AlienShape::new(AlienShape { size: 0.1, throat: f64::NAN, pitch: 99.0, ..AlienShape::HUMAN });
    assert_eq!((s.size, s.throat, s.pitch), (0.6, 1.0, 4.0));
}
