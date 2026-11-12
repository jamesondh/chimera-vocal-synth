//! Rising diphthongs (Mandarin's medials as one nucleus): sung, the glide comes on the beat and is
//! short, and the second vowel holds; a held melisma continues the vowel, not the glide.
use chimera::phonetics::sing::{SongOptions, SungNote, plan_song};
use chimera::phonetics::{Segment, Syllable};

fn vowels(nucleus: &str, beats: f64) -> Vec<(String, f64)> {
    let syl = Syllable { text: "x".into(), onset: vec![Segment::new("x")], nucleus: Segment::new(nucleus), word_end: true, ..Default::default() };
    let p = plan_song(&[SungNote { midi: 57.0, beats, syl }], &SongOptions { body: "male".into(), bpm: 60.0, t0: 0.5, ..Default::default() });
    p.segments.iter().filter(|s| s.kind == "vowel").map(|s| (s.ipa.clone(), s.t1 - s.t0)).collect()
}

#[test]
fn a_rising_diphthong_sings_a_short_glide_then_holds_its_vowel() {
    let v = vowels("ua", 2.0);
    assert_eq!(v.iter().map(|x| x.0.as_str()).collect::<Vec<_>>(), ["u", "a"]);
    assert!(v[0].1 <= 0.12 + 1e-9, "the glide is at most 120 ms, got {}", v[0].1);
    assert!(v[1].1 > 1.5, "the vowel holds the note");
    // a falling diphthong is the other way round: the first vowel holds
    let f = vowels("aɪ", 2.0);
    assert!(f[0].1 > f[1].1);
}

#[test]
fn a_syllable_can_say_how_long_its_off_glide_is() {
    let glide = |offglide: Option<f64>| {
        let syl = Syllable { text: "x".into(), onset: vec![Segment::new("x")], nucleus: Segment::new("aɪ"), word_end: true, offglide, ..Default::default() };
        let p = plan_song(&[SungNote { midi: 57.0, beats: 2.0, syl }], &SongOptions { body: "male".into(), bpm: 60.0, t0: 0.5, ..Default::default() });
        let v: Vec<f64> = p.segments.iter().filter(|s| s.kind == "vowel").map(|s| s.t1 - s.t0).collect();
        v[1] / (v[0] + v[1])
    };
    // the planner's 20% when unsaid (as before), the syllable's share when said
    assert!((glide(None) - 0.2).abs() < 1e-9);
    assert!((glide(Some(0.35)) - 0.35).abs() < 1e-9);
}
