//! Pitch bends: a syllable's front end shapes the pitch inside its note (start or end, capped in
//! length); none leaves the note as written.
use chimera::control::score::{ControlKey, Gesture};
use chimera::phonetics::sing::{SongOptions, SungNote, midi_to_hz, plan_song};
use chimera::phonetics::{Bend, Edge, Segment, Syllable};

fn syl(onset: &str, v: &str, pitch: Vec<Bend>) -> Syllable {
    Syllable { text: v.into(), onset: vec![Segment::new(onset)], nucleus: Segment::new(v), word_end: true, pitch, ..Default::default() }
}
fn opts() -> SongOptions {
    SongOptions { body: "male".into(), bpm: 60.0, t0: 0.5, ..Default::default() }
}
fn labelled<'a>(gs: &'a [Gesture], l: &str) -> Vec<&'a Gesture> {
    gs.iter().filter(|g| g.label.as_ref().is_some_and(|x| x.as_str() == l)).collect()
}
/// the pitch targets of the gestures that end by the beat (the onset's)
fn onset_f0(gs: &[Gesture]) -> Vec<f64> {
    labelled(gs, "pitch").into_iter().filter(|g| g.t1 <= 0.5 + 1e-9).filter_map(|g| g.targets.get(ControlKey::F0)).collect()
}

#[test]
fn bends_colour_the_start_or_end_of_the_note_and_none_leave_it() {
    let hz = midi_to_hz(57.0);
    let one = |pitch| plan_song(&[SungNote { midi: 57.0, beats: 2.0, syl: syl("l", "a", pitch) }], &opts());
    assert!(labelled(&one(vec![]).gestures, "bend").is_empty());
    let up = Bend { at: Edge::Start, len: 0.35, max: 0.15, st: -2.0 };
    let down = Bend { at: Edge::End, len: 0.35, max: 0.15, st: -3.0 };
    let p = one(vec![up, down]);
    let b = labelled(&p.gestures, "bend");
    assert_eq!(b.len(), 2);
    // the start bend: from just before the beat, at most 0.15 s into a 2 s note, 2 semitones down
    assert!((b[0].targets.get(ControlKey::F0).unwrap() / hz - 2f64.powf(-2.0 / 12.0)).abs() < 1e-9);
    assert!(b[0].t0 < 0.5 && (b[0].t1 - 0.65).abs() < 1e-9);
    // the end bend: the last 0.15 s of the vowel
    assert!(b[1].targets.get(ControlKey::F0).unwrap() < hz && (b[1].t1 - b[1].t0 - 0.17).abs() < 1e-9);
}

#[test]
fn no_bends_plan_exactly_as_before() {
    let notes = [SungNote { midi: 57.0, beats: 1.0, syl: syl("m", "a", vec![]) }, SungNote { midi: 60.0, beats: 1.0, syl: syl("t", "i", vec![]) }];
    let p = plan_song(&notes, &opts());
    assert!(labelled(&p.gestures, "bend").is_empty());
    // with an empty bend list serialised away, the syllables round-trip unchanged
    let j = serde_json::to_value(&notes[0].syl).unwrap();
    assert!(j.get("pitch").is_none());
}

#[test]
fn a_melisma_keeps_its_written_pitches() {
    let bend = vec![Bend { at: Edge::Start, len: 0.35, max: 0.15, st: -2.0 }];
    let first = syl("l", "a", bend.clone());
    let cont = Syllable { text: "~".into(), nucleus: Segment::new("a"), melisma: Some(true), pitch: bend, ..Default::default() };
    let p = plan_song(&[SungNote { midi: 57.0, beats: 1.0, syl: first }, SungNote { midi: 59.0, beats: 1.0, syl: cont }], &opts());
    assert_eq!(labelled(&p.gestures, "bend").len(), 1, "only the syllable's first note bends");
}

#[test]
fn a_start_bend_begins_in_the_onset() {
    // a rising tone starts low: the onset consonant already sings the bend's pitch, not the note's
    let hz = midi_to_hz(57.0);
    let up = Bend { at: Edge::Start, len: 0.35, max: 0.15, st: -3.0 };
    let p = plan_song(&[SungNote { midi: 57.0, beats: 2.0, syl: syl("m", "a", vec![up]) }], &opts());
    let f = onset_f0(&p.gestures);
    assert!(!f.is_empty());
    assert!(f.iter().all(|f| (f / hz - 2f64.powf(-3.0 / 12.0)).abs() < 1e-9), "{f:?}");
    // without a start bend the onset sings the note
    let plain = plan_song(&[SungNote { midi: 57.0, beats: 2.0, syl: syl("m", "a", vec![]) }], &opts());
    assert!(onset_f0(&plain.gestures).iter().all(|f| (f - hz).abs() < 1e-9));
}
