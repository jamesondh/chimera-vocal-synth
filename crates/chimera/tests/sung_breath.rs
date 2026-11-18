//! A breath inside a melody (`SongOptions::breath`, opt-in): at a phrase end the note before
//! gives up the breath's time, the next vowel stays on its beat, and the breath is drawn in
//! audibly: ingressive flow through parted folds, noise without voice, far under the singing.
use chimera::anatomy::human::BODY_MALE;
use chimera::control::score::ControlKey;
use chimera::phonetics::sing::{CATCH_BREATH, INHALE_LABEL, SongOptions, SungNote, midi_to_hz, plan_inhale, plan_song};
use chimera::phonetics::{Plan, Segment, Syllable};
use chimera::voice::human_voice::{HumanVoice, SourcePatch};

const BPM: f64 = 60.0;
const T0: f64 = 0.5;

fn note(onset: &[&str], nucleus: &str, beats: f64, phrase_end: bool) -> SungNote {
    SungNote { midi: 57.0, beats, syl: Syllable { text: nucleus.into(), onset: onset.iter().map(|c| Segment::new(c)).collect(), nucleus: Segment::new(nucleus), coda: vec![], word_end: true, phrase_end, ..Default::default() } }
}

/// "la | so la": a breath mark after the first note, whose length is `beats`.
fn melody(beats: f64) -> Vec<SungNote> {
    vec![note(&["l"], "a", beats, true), note(&["s"], "o", 1.0, false), note(&["l"], "a", 1.0, false)]
}

fn sing(notes: &[SungNote], breath: Option<f64>) -> Plan {
    plan_song(notes, &SongOptions { body: "male".into(), bpm: BPM, t0: T0, vibrato: 0.0, breath, ..Default::default() })
}

/// (end of the first phrase's sound, start of the second's)
fn gap(p: &Plan) -> (f64, f64) {
    let first_vowel = p.segments.iter().position(|s| s.kind == "vowel").unwrap();
    (p.segments[first_vowel].t1, p.segments[first_vowel + 1].t0)
}

fn inhales(p: &Plan) -> Vec<&chimera::control::score::Gesture> {
    p.gestures.iter().filter(|g| g.label.is_some_and(|l| l.as_str() == INHALE_LABEL)).collect()
}

#[test]
fn the_note_before_gives_up_the_breath_and_the_next_vowel_stays_on_its_beat() {
    let notes = melody(2.0);
    let (plain, breathing) = (sing(&notes, None), sing(&notes, Some(CATCH_BREATH)));
    let (a, b) = gap(&breathing);
    assert!((b - a - CATCH_BREATH).abs() < 1e-9, "the gap is {:.0} ms", (b - a) * 1000.0);
    let (a0, b0) = gap(&plain);
    assert!(b0 - a0 < 0.1, "without the option the lift stays short: {:.0} ms", (b0 - a0) * 1000.0);
    assert!(inhales(&plain).is_empty());
    // every vowel on its beat, and everything after the breath planned exactly as without it
    let vowels = |p: &Plan| p.segments.iter().filter(|s| s.kind == "vowel").map(|s| s.t0).collect::<Vec<_>>();
    for (v, beat) in vowels(&breathing).iter().zip([T0, T0 + 2.0, T0 + 3.0]) {
        assert!((v - beat).abs() < 1e-6, "a vowel at {v:.4} s, its beat is {beat:.4} s");
    }
    assert_eq!(format!("{:?}", &plain.segments[2..]), format!("{:?}", &breathing.segments[2..]));
    // the inhalation: negative breath with the folds apart, inside the gap, done before the next sound
    let g = inhales(&breathing);
    assert_eq!(g.len(), 2);
    let breath = g.iter().find(|g| g.targets.has(ControlKey::BREATH)).unwrap();
    assert!(breath.targets.get(ControlKey::BREATH).unwrap() < 0.0);
    assert!(g.iter().any(|g| g.targets.get(ControlKey::ABDUCTION).is_some_and(|x| x > 0.3)));
    assert!(breath.t0 - breath.ramp_in > a && breath.t1 + breath.ramp_out < b, "the inhalation {:.3}..{:.3} s leaves the gap {a:.3}..{b:.3} s", breath.t0, breath.t1);
}

#[test]
fn a_short_note_keeps_half_of_itself_and_a_gap_too_short_stays_silent() {
    // a sixteenth at 60 bpm before the breath: 125 ms is all it gives, the [s] takes most of it
    let p = sing(&melody(0.25), Some(CATCH_BREATH));
    let first_vowel = p.segments.iter().find(|s| s.kind == "vowel").unwrap();
    assert!((first_vowel.t1 - (T0 + 0.125)).abs() < 1e-9, "the note ends at {:.3} s", first_vowel.t1);
    assert!(inhales(&p).is_empty());
    assert!(plan_inhale(1.0, 1.2).is_empty());
    assert_eq!(plan_inhale(1.0, 1.4).len(), 2);
}

#[test]
fn the_breath_is_heard_as_quiet_noise_without_voice() {
    const FS: f64 = 48000.0;
    let notes = melody(2.0);
    let rms = |x: &[f32]| (x.iter().map(|v| (*v as f64) * (*v as f64)).sum::<f64>() / x.len() as f64).sqrt();
    let render = |p: &Plan| {
        let mut v = HumanVoice::new(FS, BODY_MALE, 2);
        v.set_source(&SourcePatch { f0: Some(midi_to_hz(57.0)), voiced: Some(true), pressure: Some(9.0), tension: Some(0.55), ..Default::default() });
        v.schedule(&p.gestures);
        let mut out = vec![0f32; (FS * (p.end + 0.3)) as usize];
        v.render(&mut out);
        out
    };
    let with = sing(&notes, Some(CATCH_BREATH));
    let (a, b) = gap(&with);
    let at = |t: f64| (t * FS) as usize;
    let out = render(&with);
    assert!(out.iter().all(|x| x.is_finite()));
    let vowel = rms(&out[at(T0 + 0.3)..at(T0 + 1.0)]);
    // the middle of the gap, clear of the voice's release and of the next onset
    let window = at(a + 0.14)..at(b - 0.12);
    let inhale = rms(&out[window.clone()]);
    let db = 20.0 * (inhale / vowel).log10();
    assert!((-45.0..=-24.0).contains(&db), "the inhalation is {db:.1} dB against the sung vowel");
    // noise, not voice: no peak at the note's pitch period in the autocorrelation
    let x: Vec<f64> = out[window].iter().map(|v| *v as f64).collect();
    let lag = (FS / midi_to_hz(57.0)).round() as usize;
    let r0: f64 = x.iter().map(|v| v * v).sum();
    let r: f64 = x.iter().zip(&x[lag..]).map(|(p, q)| p * q).sum();
    assert!(r / r0 < 0.3, "the inhalation is voiced: autocorrelation {:.2} at the pitch period", r / r0);
    // the same stretch of time without the option is still the first note's vowel
    let plain = render(&sing(&notes, None));
    assert!(rms(&plain[at(a + 0.14)..at(b - 0.12)]) > 0.3 * vowel);
}
