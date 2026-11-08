//! Voiceless fricatives: the glottis closes again by the release, so the vowels around them are
//! voiced; and a constriction at the back of the tract makes frication.
//!
//! The bugs these keep fixed:
//! - The glottal opening for a voiceless fricative lasted ~23 ms past it (an abduction gesture
//!   saturates at a third of its activation), so a short vowel after one was voiced late: /ɪ/ in
//!   "fishes" 14 %, /a/ between f and ħ in كَيْفَ حَالُكَ 37 %, /ɪ/ in "city" 0 %. The glottis
//!   closes by the release (Löfqvist & Yoshioka 1980).
//! - A constriction one or two sections into the mouth (the uvular fricatives) put its upstream
//!   injection point on the pharynx–mouth junction, which has no branch, and lost its whole
//!   frication source: χ was 29 dB under the vowel, quieter than [h].

use chimera::phonetics::ipa::tokenize_ipa;
use chimera::phonetics::planner::{PlanOptions, plan_utterance};
use chimera::voice::human_voice::HumanVoice;

const FS: f64 = 48000.0;

/// Per planned vowel: the share of its span the folds are voicing.
fn voiced_shares(ipa: &str) -> Vec<(String, f64)> {
    let mut v = HumanVoice::with_defaults(FS);
    let plan = plan_utterance(&tokenize_ipa(ipa), &PlanOptions { body: "male".into(), t0: 0.05, f0: Some(110.0), ..Default::default() });
    v.schedule(&plan.gestures);
    let n = (FS * (plan.end + 0.05)).round() as usize;
    let (mut out, mut i) = (vec![0.0f32; 48], 0usize);
    let (mut on, mut all) = (vec![0usize; plan.segments.len()], vec![0usize; plan.segments.len()]);
    while i < n {
        v.render(&mut out);
        i += 48;
        let t = i as f64 / FS;
        if let Some(k) = plan.segments.iter().position(|s| t >= s.t0 && t < s.t1) {
            all[k] += 1;
            on[k] += (v.glottis.voicing > 0.5) as usize;
        }
    }
    plan.segments.iter().enumerate().filter(|(_, s)| s.kind == "vowel").map(|(k, s)| (s.ipa.clone(), on[k] as f64 / all[k].max(1) as f64)).collect()
}

#[test]
fn a_vowel_between_voiceless_fricatives_is_voiced() {
    // before the fix: 14, 14, 37, 48 and 0 %
    for (ipa, k, min) in [("ˈfɪʃɪz", 0, 0.4), ("ˈθɪsəl", 0, 0.4), ("ˈkajfa ˈħaːluk", 1, 0.55), ("ˈsafiʃ", 1, 0.55), ("ˈsɪti", 0, 0.45)] {
        let v = voiced_shares(ipa);
        assert!(v[k].1 >= min, "{ipa}: /{}/ voiced {:.0} % of its length, want ≥ {:.0} %", v[k].0, 100.0 * v[k].1, 100.0 * min);
    }
}

/// Noise sources active at the middle of the first consonant.
fn sources_mid_consonant(ipa: &str) -> usize {
    let mut v = HumanVoice::with_defaults(FS);
    let plan = plan_utterance(&tokenize_ipa(ipa), &PlanOptions { body: "male".into(), t0: 0.05, f0: Some(110.0), ..Default::default() });
    v.schedule(&plan.gestures);
    let c = plan.segments.iter().find(|s| s.kind != "vowel").unwrap();
    let mid = 0.5 * (c.t0 + c.t1);
    let (mut out, mut i) = (vec![0.0f32; 128], 0usize);
    while (i as f64) / FS < mid {
        v.render(&mut out);
        i += 128;
    }
    v.airway.net.noise_sources().iter().filter(|s| s.level > 0.0).count()
}

#[test]
fn back_fricatives_have_their_frication_sources() {
    // glottis, the constriction's jet, and the back obstacle
    for ipa in ["ˈaχa", "ˈaħa"] {
        let n = sources_mid_consonant(ipa);
        assert!(n >= 3, "{ipa}: {n} noise sources sounding, want the glottis, the jet and the back obstacle");
    }
}
