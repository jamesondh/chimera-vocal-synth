//! Pharyngealization (IPA ˤ, the Arabic emphatics): the consonants exist, the stops are
//! unaspirated, and the pulled-back tongue lowers F2 of the vowel as much as it does in Arabic.

use chimera::airway::frequency::{find_peaks, transfer_function};
use chimera::anatomy::human::{BODY_MALE, HumanArtic};
use chimera::phonetics::inventory::phoneme;
use chimera::phonetics::ipa::tokenize_ipa;
use chimera::phonetics::planner::{PlanOptions, plan_utterance};
use chimera::voice::human_airway::HumanAirway;
use chimera::voice::human_voice::HumanVoice;

#[test]
fn the_emphatics_exist_and_are_unaspirated_stops() {
    for ipa in ["tˤ", "dˤ", "sˤ", "zˤ", "ðˤ", "lˤ", "ɾˤ", "rˤ"] {
        assert!(phoneme(ipa, "male".into()).is_some(), "{ipa}");
    }
    assert!(phoneme("tˤ", "male".into()).unwrap().unaspirated);
    assert!(phoneme("q", "male".into()).unwrap().unaspirated);
    assert!(!phoneme("t", "male".into()).unwrap().unaspirated);
    assert!(!phoneme("sˤ", "male".into()).unwrap().unaspirated);
    let segs: Vec<String> = tokenize_ipa("sˤaˤbaːħ tˠa").into_iter().map(|s| s.ipa).collect();
    assert_eq!(segs, ["sˤ", "aˤ", "b", "a", "ħ", "tˤ", "a"]);
}

const FS: f64 = 48000.0;

/// The articulation at each vowel's midpoint as the voice says `ipa`.
fn vowel_postures(ipa: &str) -> Vec<HumanArtic> {
    let mut v = HumanVoice::with_defaults(FS);
    let plan = plan_utterance(&tokenize_ipa(ipa), &PlanOptions { body: "male".into(), t0: 0.05, f0: Some(110.0), ..Default::default() });
    v.schedule(&plan.gestures);
    let mids: Vec<f64> = plan.segments.iter().filter(|s| s.kind == "vowel").map(|s| 0.5 * (s.t0 + s.t1)).collect();
    let n = (FS * (plan.end + 0.05)).round() as usize;
    let mut out = vec![0.0f32; 128];
    let (mut i, mut k, mut got) = (0, 0, vec![]);
    while i < n {
        v.render(&mut out);
        i += 128;
        while k < mids.len() && i as f64 / FS >= mids[k] {
            got.push(v.artic.x);
            k += 1;
        }
    }
    got
}

fn f1_f2(a: &HumanArtic) -> (f64, f64) {
    let mut air = HumanAirway::new(96000.0, BODY_MALE);
    let freqs: Vec<f64> = (0..600).map(|i| 60.0 * (5000.0f64 / 60.0).powf(i as f64 / 599.0)).collect();
    air.apply(a, 0.0);
    let el = air.net.linear_elements();
    let h = transfer_function(&el, air.pharynx.first_node(), &freqs, &[air.glottis_port]);
    let pk = find_peaks(&freqs, &h.mag, 150.0);
    (pk[0].freq, pk[1].freq)
}

#[test]
fn emphasis_lowers_f2_of_the_vowel_as_in_arabic() {
    // Emphatic consonants lower F2 of the following vowel by a few hundred hertz in front and
    // open vowels and by less in /u/ (Jongman, Herd & Al-Masri 2011; Card 1983).
    for (plain, emphatic, min_drop) in [("ˈsaːb", "ˈsˤaˤːb", 250.0), ("ˈtiːn", "ˈtˤiˤːn", 250.0), ("ˈdaːl", "ˈdˤaˤːl", 250.0), ("ˈðal", "ˈðˤaˤl", 250.0), ("ˈsuːq", "ˈsˤuˤːq", 40.0)] {
        let (_, f2p) = f1_f2(&vowel_postures(plain)[0]);
        let (_, f2e) = f1_f2(&vowel_postures(emphatic)[0]);
        assert!(f2p - f2e >= min_drop, "{plain} F2 {f2p:.0} → {emphatic} {f2e:.0}: dropped {:.0} Hz, want ≥ {min_drop}", f2p - f2e);
    }
}
