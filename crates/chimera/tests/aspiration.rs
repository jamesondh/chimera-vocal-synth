//! Aspiration is the front end's call: a segment may say whether a voiceless stop or affricate is
//! aspirated (IPA ʰ / ˭); unsaid, the planner keeps English's rule. The sing planner keeps the flag.
use chimera::anatomy::human::BodyMix;
use chimera::control::score::{ControlKey, Gesture};
use chimera::phonetics::ipa::tokenize_ipa;
use chimera::phonetics::planner::{PlanOptions, plan_utterance};
use chimera::phonetics::sing::{SongOptions, SungNote, plan_song};
use chimera::phonetics::{Segment, Syllable};

/// The abduction of the first "aspiration" gesture: 0.36 aspirated, 0.22 not.
fn aspiration(gs: &[Gesture]) -> f64 {
    let g = gs.iter().find(|g| g.label.as_ref().is_some_and(|l| l.as_str() == "aspiration")).expect("an aspiration gesture");
    g.targets.get(ControlKey::ABDUCTION).unwrap()
}

fn say(segs: Vec<Segment>) -> f64 {
    aspiration(&plan_utterance(&segs, &PlanOptions { body: BodyMix::MALE, ..Default::default() }).gestures)
}

#[test]
fn ipa_marks_set_aspiration() {
    let a: Vec<Option<bool>> = tokenize_ipa("pʰa p˭a pa tɕʰi").into_iter().filter(|s| s.ipa != "a" && s.ipa != "i").map(|s| s.aspirated).collect();
    assert_eq!(a, [Some(true), Some(false), None, Some(true)]);
}

#[test]
fn the_front_end_overrides_the_english_rule() {
    // English's rule: "pa" aspirated, "spa" not
    assert_eq!(say(tokenize_ipa("pa")), 0.36);
    assert_eq!(say(tokenize_ipa("spa")), 0.22);
    // said explicitly, either way
    assert_eq!(say(tokenize_ipa("p˭a")), 0.22);
    assert_eq!(say(tokenize_ipa("spʰa")), 0.36);
}

#[test]
fn sung_onsets_keep_the_flag() {
    let syl = |aspirated| Syllable { text: "pa".into(), onset: vec![Segment { aspirated, ..Segment::new("p") }], nucleus: Segment::new("a"), coda: vec![], melisma: None, word_end: true, phrase_end: false, ..Default::default() };
    let sing = |aspirated| aspiration(&plan_song(&[SungNote { midi: 57.0, beats: 1.0, syl: syl(aspirated) }], &SongOptions { body: "male".into(), bpm: 90.0, t0: 0.5, vibrato: 0.0, ..Default::default() }).gestures);
    assert_eq!(sing(None), 0.36);
    assert_eq!(sing(Some(false)), 0.22);
}

fn aspir_abd_on(ipa: &str, stop_index: usize) -> f64 {
    let plan = plan_utterance(&tokenize_ipa(ipa), &PlanOptions { body: BodyMix::MALE, ..Default::default() });
    let stop = plan.gestures.iter().filter(|g| g.label_str() == "aspiration").nth(stop_index).unwrap();
    stop.targets.get(ControlKey::ABDUCTION).unwrap()
}

#[test]
fn english_skips_aspiration_before_unstressed_medial_vowel() {
    // word-initial onset still aspirates (even before an unmarked/unstressed vowel)
    assert_eq!(aspir_abd_on("pa", 0), 0.36);
    assert_eq!(aspir_abd_on("aɪ pa", 0), 0.36);
    // coda / medial stop before unstressed ʌ: unaspirated (abd 0.22)
    assert_eq!(aspir_abd_on("ˈæt.ʌˌtʰɔn", 0), 0.22);
    assert_eq!(aspir_abd_on("ˈætʌˌtʰɔn", 0), 0.22);
    // explicit ʰ on the stressed onset still aspirates
    assert_eq!(aspir_abd_on("ˈæt.ʌˌtʰɔn", 1), 0.36);
    assert_eq!(aspir_abd_on("ˈætʰ.ʌˌtɔn", 0), 0.36);
    // before a stressed vowel, English aspirates
    assert_eq!(aspir_abd_on("ˈætˈʌ", 0), 0.36);
    // A marked VOT keeps the front end on the original aspiration path.
    let mut segs = tokenize_ipa("ˈætʌ");
    segs[1].vot = Some(0.08);
    assert_eq!(say(segs), 0.36);
}

/// The planned nucleus leaves time after the first stop's aspiration ramp and before the
/// next stop starts to close.
#[test]
fn unstressed_vowel_between_stops_keeps_a_nucleus() {
    let plan = plan_utterance(&tokenize_ipa("ˈæt.ʌˌtʰɔn"), &PlanOptions { body: BodyMix::FEMALE, ..Default::default() });
    let v = plan.segments.iter().find(|s| s.ipa == "ʌ").expect("ʌ");
    let dur = v.t1 - v.t0;
    assert!((dur - 0.1).abs() < 1e-9, "ʌ duration {:.2} ms", 1000.0 * dur);

    let release = plan.gestures.iter().find(|g| g.label_str() == "aspiration").unwrap().end();
    let next_closure = plan.gestures.iter()
        .filter(|g| g.label_str() == "t" && g.t0 > v.t0 && g.t0 < v.t1)
        .map(|g| g.t0)
        .min_by(f64::total_cmp)
        .expect("the second stop closes into the vowel");
    assert!(next_closure - release >= 0.04, "less than 40 ms between aspiration and closure");
}
