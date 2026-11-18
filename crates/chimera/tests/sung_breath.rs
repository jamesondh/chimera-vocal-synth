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
    assert!(plan_inhale(1.0, 1.2, 1.0).is_empty());
    assert_eq!(plan_inhale(1.0, 1.4, 1.0).len(), 2);
}

#[test]
fn the_level_scales_only_the_pressure_of_the_breath_drawn_in() {
    let notes = melody(2.0);
    let at = |breath_level: Option<f64>| plan_song(&notes, &SongOptions { body: "male".into(), bpm: BPM, t0: T0, vibrato: 0.0, breath: Some(CATCH_BREATH), breath_level });
    let pressure = |p: &Plan| inhales(p).iter().find_map(|g| g.targets.get(ControlKey::BREATH)).unwrap();
    let rest = |p: &Plan| format!("{:?}", p.gestures.iter().filter(|g| !(g.label.is_some_and(|l| l.as_str() == INHALE_LABEL) && g.targets.has(ControlKey::BREATH))).collect::<Vec<_>>());
    let own = at(None);
    // none named, 1, and anything that is not a positive number: the planner's own level
    for odd in [Some(1.0), Some(0.0), Some(-2.0), Some(f64::NAN), Some(f64::INFINITY)] {
        assert_eq!(format!("{:?}", at(odd)), format!("{own:?}"), "level {odd:?}");
    }
    for level in [0.5, 2.0] {
        let p = at(Some(level));
        assert_eq!(pressure(&p), level * pressure(&own));
        assert_eq!(rest(&p), rest(&own));
        assert_eq!(format!("{:?}", p.segments), format!("{:?}", own.segments));
    }
    // without a breath there is nothing to scale: the plan sung before the voice could breathe in
    let plain = plan_song(&notes, &SongOptions { body: "male".into(), bpm: BPM, t0: T0, vibrato: 0.0, breath_level: Some(2.0), ..Default::default() });
    assert_eq!(format!("{plain:?}"), format!("{:?}", sing(&notes, None)));
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

/// Mandarin's rising rhymes and held nasal codas came to the planner after the breath did: 安 an
/// ends the phrase holding its closure with the velum open, 天 tian starts the next with a medial
/// glide sung before its beat. The breath's gap is still the one asked for, the held vowel still on
/// its beat, the breath still far under the singing, and the voice holds at 16 kHz.
#[test]
fn a_breath_between_a_held_nasal_coda_and_a_rising_rhyme() {
    use chimera::phonetics::mandarin::pinyin_syllable;
    use chimera::voice::human_voice::oversample_for;
    let py = |s: &str, phrase_end: bool| Syllable { phrase_end, ..pinyin_syllable(s, s).unwrap() };
    let notes = vec![SungNote { midi: 57.0, beats: 2.0, syl: py("an1", true) }, SungNote { midi: 57.0, beats: 2.0, syl: py("tian1", false) }];
    let p = sing(&notes, Some(CATCH_BREATH));
    // the first phrase's last sound (the [n]) and the second's first (the [tʰ])
    let k = p.segments.iter().position(|s| s.kind == "stop").unwrap();
    let (a, b) = (p.segments[k - 1].t1, p.segments[k].t0);
    assert!((b - a - CATCH_BREATH).abs() < 1e-9, "the gap is {:.0} ms", (b - a) * 1000.0);
    let held = p.segments.iter().rfind(|s| s.kind == "vowel").unwrap();
    assert!((held.t0 - (T0 + 2.0)).abs() < 1e-6, "tian's held vowel starts at {:.4} s", held.t0);
    assert_eq!(inhales(&p).len(), 2);
    for fs in [48000.0, 16000.0] {
        let mut v = HumanVoice::new(fs, BODY_MALE, oversample_for(fs));
        v.set_source(&SourcePatch { f0: Some(midi_to_hz(57.0)), voiced: Some(true), pressure: Some(9.0), tension: Some(0.55), ..Default::default() });
        v.schedule(&p.gestures);
        let mut out = vec![0f32; (fs * (p.end + 0.3)) as usize];
        v.render(&mut out);
        assert!(out.iter().all(|x| x.is_finite()), "NaN at {fs}");
        let rms = |t0: f64, t1: f64| (out[(t0 * fs) as usize..(t1 * fs) as usize].iter().map(|v| (*v as f64) * (*v as f64)).sum::<f64>() / ((t1 - t0) * fs)).sqrt();
        let db = 20.0 * (rms(a + 0.14, b - 0.12) / rms(T0 + 0.3, T0 + 1.0)).log10();
        assert!((-50.0..=-20.0).contains(&db), "at {fs}: the inhalation is {db:.1} dB against the sung vowel");
    }
}
