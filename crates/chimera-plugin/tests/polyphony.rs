//! Several voices in the plugin: one for each finger. With one voice it is the instrument it was.
use chimera::phonetics::beatbox::beat_ready;
use chimera::voice::human_voice::HumanVoice;
use chimera_plugin::core::{Core, Event, MAX_VOICES, Mode, Prepared, Settings, Shared, Transport, VoiceParams};

const FS: f64 = 44100.0;

fn params() -> VoiceParams {
    VoiceParams { vibrato: 35.0, vibrato_rate: 6.0, breath: 8.0, pressed: 0.5, bright: 0.25, breathy: 0.15, rough: 0.1, fry: 0.0, glide: 0.06, ring: 0.0, restart: true, mpe: true, voices: 1 }
}

fn core_with(settings: &Settings, shared: &Shared, p: &VoiceParams) -> Core {
    let prepared = Prepared::of(settings, shared);
    shared.restart(FS);
    let c = Core::new(FS, settings, &prepared, *p, shared);
    shared.adopt(prepared);
    c
}

/// Run `samples` in blocks of `block`, taking each event on its sample, as `process` does.
fn run(c: &mut Core, shared: &Shared, p: &VoiceParams, events: &[(usize, Event)], samples: usize, block: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; samples];
    let (mut pos, mut next) = (0, 0);
    while pos < samples {
        let n = block.min(samples - pos);
        c.begin_block(p, Transport::default(), shared);
        let mut at = pos;
        while next < events.len() && events[next].0 < pos + n {
            let t = events[next].0;
            if t > at {
                c.render(&mut out[at..t]);
                at = t;
            }
            c.event(events[next].1);
            next += 1;
        }
        if at < pos + n {
            c.render(&mut out[at..pos + n]);
        }
        c.end_block(n, shared);
        pos += n;
    }
    out
}

fn on(midi: u8, channel: u8) -> Event {
    Event::NoteOn { midi, velocity: 96.0, channel }
}

fn off(midi: u8) -> Event {
    Event::NoteOff { midi }
}

fn peak(x: &[f32]) -> f32 {
    x.iter().fold(0.0, |p, v| p.max(v.abs()))
}

/// FNV-1a over the samples' bits.
fn fingerprint(x: &[f32]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for b in x.iter().flat_map(|v| v.to_bits().to_le_bytes()) {
        h = (h ^ b as u64).wrapping_mul(0x100000001b3);
    }
    h
}

/// Three fingers of an MPE controller: overlapping notes, each channel's bend, pressure and slide,
/// poly pressure, and the wheel.
fn performance() -> Vec<(usize, Event)> {
    let s = FS as usize;
    let (bend, pressure, slide) = (|channel, value| Event::Bend { channel, value }, |channel, value| Event::Pressure { channel, value }, |channel, value| Event::Slide { channel, value });
    vec![
        (900, slide(1, 0.4)),
        (1000, on(57, 1)),
        (3000, pressure(1, 0.6)),
        (s / 4, bend(1, 0.52)),
        (s / 2, slide(2, 0.5)),
        (s / 2 + 5, on(61, 2)),
        (s / 2 + 700, on(64, 3)),
        (s / 2 + 9000, slide(2, 0.9)),
        (s, pressure(3, 1.0)),
        (s + 100, Event::KeyPressure { midi: 61, value: 0.8 }),
        (s + s / 4, off(64)),
        (s + s / 2, bend(0, 0.6)),
        (3 * s / 2 + 3000, off(61)),
        (2 * s, off(57)),
        (2 * s + s / 4, on(62, 1)),
        (2 * s + s / 4 + 4000, bend(1, 0.48)),
        (3 * s, off(62)),
    ]
}

/// The plugin's output with one voice, as it was before it had more (commit b4d513d). Polyphony must
/// not move one sample of it.
#[test]
fn one_voice_sounds_as_it_did_before_polyphony() {
    let n = 7 * FS as usize / 2;
    let sing = Settings::default();
    let play = Settings { mode: Mode::Play, ..Default::default() };
    let speak = Settings { mode: Mode::Speak, ..Default::default() };
    let beatbox = Settings { mode: Mode::Beatbox, artic: beat_ready(sing.body), ..Default::default() };
    let pads: Vec<(usize, Event)> = vec![(500, on(36, 0)), (9000, on(64, 0)), (15000, on(38, 0)), (30000, on(67, 0)), (40000, off(64)), (60000, off(67)), (70000, on(42, 0))];
    let cases = [("sing", sing, performance(), 0xbed5010e2e687de8u64), ("play", play, performance(), 0x61ac1399663bf617), ("speak", speak, performance(), 0x324017c331cb7f17), ("beatbox", beatbox, pads, 0xe32719fbea2be333)];
    let mut moved = Vec::new();
    for (name, settings, events, expected) in cases {
        let shared = Shared::default();
        let out = run(&mut core_with(&settings, &shared, &params()), &shared, &params(), &events, n, 480);
        assert!(peak(&out) > 0.01, "{name} is silent");
        if fingerprint(&out) != expected {
            moved.push(format!("{name}: 0x{:016x}", fingerprint(&out)));
        }
    }
    assert!(moved.is_empty(), "the sound moved: {moved:?}");
}

/// Long enough for a 12 ms glide to arrive.
const GLIDED: usize = 8192;

fn voices(n: usize) -> VoiceParams {
    VoiceParams { voices: n, ..params() }
}

fn playing(n: usize, shared: &Shared) -> Core {
    core_with(&Settings { mode: Mode::Play, ..Default::default() }, shared, &voices(n))
}

fn body(c: &Core, v: usize) -> &HumanVoice {
    c.voice(v).0.voice()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-3 * b.abs().max(1.0)
}

fn hz(midi: u8) -> f64 {
    chimera::math::midi_to_hz(midi as f64)
}

/// The pitch each voice is on, 0 for a voice with its breath off.
fn chord(c: &Core) -> Vec<f64> {
    (0..MAX_VOICES).map(|v| if body(c, v).gate > 0.5 { body(c, v).source.f0 } else { 0.0 }).collect()
}

#[test]
fn each_note_of_a_chord_gets_a_voice_and_gives_it_back() {
    let shared = Shared::default();
    let p = voices(3);
    let mut c = playing(3, &shared);
    let out = run(&mut c, &shared, &p, &[(0, on(57, 1)), (200, on(61, 2)), (400, on(64, 3))], GLIDED, 256);
    assert_eq!(chord(&c)[..4], [hz(57), hz(61), hz(64), 0.0]);
    assert!(out.iter().all(|x| x.is_finite()) && peak(&out) > 0.01);
    // the middle finger lifts: its voice stops, the others don't move
    run(&mut c, &shared, &p, &[(0, off(61))], GLIDED * 2, 256);
    assert_eq!(chord(&c)[..3], [hz(57), 0.0, hz(64)]);
    // a new finger takes the free voice
    run(&mut c, &shared, &p, &[(0, on(66, 4))], GLIDED, 256);
    assert_eq!(chord(&c)[..3], [hz(57), hz(66), hz(64)]);
    run(&mut c, &shared, &p, &[(0, Event::AllOff)], GLIDED * 2, 256);
    assert_eq!(chord(&c), [0.0; MAX_VOICES]);
}

#[test]
fn a_free_voice_starts_on_its_pitch_and_the_single_voice_still_glides() {
    for (n, snaps) in [(1, false), (3, true)] {
        let shared = Shared::default();
        let p = VoiceParams { glide: 0.3, ..voices(n) };
        let mut c = core_with(&Settings { mode: Mode::Play, ..Default::default() }, &shared, &p);
        // far from the 110 Hz it rests on, with the finger already bent up an octave
        run(&mut c, &shared, &p, &[(0, Event::Bend { channel: 2, value: 0.625 }), (0, on(81, 2))], 256, 256);
        shared.watch();
        run(&mut c, &shared, &p, &[], 2048, 256);
        // within the vibrato's reach
        let f0 = shared.telemetry.lock().unwrap().block[3];
        assert_eq!((f0 / (2.0 * hz(81)) - 1.0).abs() < 0.05, snaps, "{n} voices: {f0} Hz a moment into a note at {}", 2.0 * hz(81));
    }
}

#[test]
fn each_finger_moves_only_its_own_voice() {
    let shared = Shared::default();
    let p = voices(3);
    let mut c = playing(3, &shared);
    let slide = |channel, value| Event::Slide { channel, value };
    run(&mut c, &shared, &p, &[(0, slide(3, 0.5)), (0, on(57, 1)), (0, on(61, 2)), (0, on(64, 3))], GLIDED, 256);
    let events = [(0, Event::Bend { channel: 2, value: 0.625 }), (0, Event::Pressure { channel: 1, value: 1.0 }), (0, slide(3, 0.7)), (0, Event::KeyPressure { midi: 64, value: 0.5 })];
    run(&mut c, &shared, &p, &events, GLIDED, 256);
    let state = |v: usize| (body(&c, v).bend, body(&c, v).source.pressure, body(&c, v).source.brightness);
    let (a, b, d) = (state(0), state(1), state(2));
    assert!(close(a.0, 1.0) && close(a.1, 8.0 * 1.75) && close(a.2, 0.25), "pressed: {a:?}");
    assert!(close(b.0, 2.0) && close(b.1, 8.0) && close(b.2, 0.25), "bent an octave: {b:?}");
    assert!(close(d.0, 1.0) && close(d.1, 8.0 * 1.375) && close(d.2, 0.45), "slid, with poly pressure: {d:?}");
    // the wheel is everyone's
    run(&mut c, &shared, &p, &[(0, Event::Bend { channel: 0, value: 1.0 })], GLIDED, 256);
    let wheel = 2f64.powf(2.0 / 12.0);
    assert!(close(body(&c, 0).bend, wheel) && close(body(&c, 1).bend, 2.0 * wheel) && close(body(&c, 2).bend, wheel));
}

#[test]
fn with_more_fingers_than_voices_the_oldest_note_gives_way_and_comes_back() {
    let shared = Shared::default();
    let p = voices(2);
    let mut c = playing(2, &shared);
    run(&mut c, &shared, &p, &[(0, on(57, 1)), (100, on(61, 2)), (200, on(64, 3))], GLIDED, 256);
    assert_eq!(chord(&c)[..3], [hz(64), hz(61), 0.0], "the third note takes the first one's voice");
    run(&mut c, &shared, &p, &[(0, off(64))], GLIDED, 256);
    assert_eq!(chord(&c)[..2], [hz(57), hz(61)], "and the first note, still held, comes back");
    // now the second note is the one that started longest ago: it gives way, and lifts unheard
    run(&mut c, &shared, &p, &[(0, on(64, 3)), (100, off(61))], GLIDED, 256);
    assert_eq!(chord(&c)[..2], [hz(57), hz(64)]);
    run(&mut c, &shared, &p, &[(0, off(64)), (0, off(57))], GLIDED * 2, 256);
    assert_eq!(chord(&c), [0.0; MAX_VOICES]);
}

#[test]
fn a_finger_keeps_its_voice_from_note_to_note() {
    let shared = Shared::default();
    let p = voices(3);
    let mut c = playing(3, &shared);
    run(&mut c, &shared, &p, &[(0, on(57, 5)), (1000, off(57)), (2000, on(61, 6))], GLIDED, 256);
    assert_eq!(chord(&c)[..3], [0.0, hz(61), 0.0], "a new finger takes the voice free the longest");
    // voice 2 has been free the longest, but voice 0's release still follows channel 5
    run(&mut c, &shared, &p, &[(0, on(64, 5))], GLIDED, 256);
    assert_eq!(chord(&c)[..3], [hz(64), hz(61), 0.0]);
    // with MPE off a channel means nothing
    let plain = VoiceParams { mpe: false, ..p };
    run(&mut c, &shared, &plain, &[(0, off(64)), (0, off(61)), (1000, on(66, 6))], GLIDED, 256);
    assert_eq!(chord(&c)[..3], [0.0, 0.0, hz(66)]);
}

/// The cursor and each voice's syllable, after these notes.
fn sung(n: usize, events: &[(usize, Event)]) -> (usize, Vec<Option<usize>>) {
    let shared = Shared::default();
    let settings = Settings { lyrics: "one two three four five six".into(), ..Default::default() };
    let mut c = core_with(&settings, &shared, &voices(n));
    run(&mut c, &shared, &voices(n), events, 30000, 256);
    (c.singer().cursor(), (0..n).map(|v| c.voice(v).1.sung()).collect())
}

#[test]
fn a_chord_sings_one_syllable_and_a_later_note_the_next() {
    let ms = |t: usize| t * FS as usize / 1000;
    let together = [(0, on(57, 1)), (ms(20), on(61, 2)), (ms(40), on(64, 3))];
    assert_eq!(sung(3, &together), (1, vec![Some(0), Some(0), Some(0)]), "three fingers 20 ms apart");
    assert_eq!(sung(1, &together), (3, vec![Some(2)]), "one voice sings each note's syllable, as before");
    // a melody over a held note moves through the words
    let melody = [(0, on(57, 1)), (ms(200), on(61, 2)), (ms(400), on(64, 3))];
    assert_eq!(sung(3, &melody), (3, vec![Some(0), Some(1), Some(2)]));
    // a chord, then the next chord
    let chords = [(0, on(57, 1)), (ms(5), on(61, 2)), (ms(300), off(57)), (ms(300), off(61)), (ms(400), on(59, 1)), (ms(410), on(62, 2))];
    assert_eq!(sung(2, &chords), (2, vec![Some(1), Some(1)]));
    // the fourth finger of a chord on three voices sings as a second key does on one: the next
    // syllable, legato, on the voice of the oldest note
    let four = [(0, on(57, 1)), (ms(5), on(61, 2)), (ms(10), on(64, 3)), (ms(15), on(67, 4))];
    assert_eq!(sung(3, &four), (2, vec![Some(1), Some(0), Some(0)]));
}

#[test]
fn the_editor_shows_the_voice_with_the_newest_note() {
    let shared = Shared::default();
    let p = voices(3);
    let mut c = core_with(&Settings::default(), &shared, &p);
    let note = |shared: &Shared| shared.note.load(std::sync::atomic::Ordering::Relaxed);
    run(&mut c, &shared, &p, &[(0, on(57, 1)), (9000, on(61, 2))], GLIDED * 2, 256);
    assert_eq!((c.shown(), note(&shared)), (1, 61));
    run(&mut c, &shared, &p, &[(0, off(61))], GLIDED, 256);
    assert_eq!((c.shown(), note(&shared)), (0, 57), "back to the note still held");
    run(&mut c, &shared, &p, &[(0, off(57))], GLIDED, 256);
    assert_eq!((c.shown(), note(&shared)), (0, -1), "its release stays on the screen");
}

#[test]
fn a_voice_taken_away_stops_its_note_fades_and_then_costs_nothing() {
    let shared = Shared::default();
    let mut c = playing(3, &shared);
    run(&mut c, &shared, &voices(3), &[(0, on(57, 1)), (0, on(61, 2)), (0, on(64, 3))], GLIDED, 256);
    let before = body(&c, 2).clock;
    run(&mut c, &shared, &voices(1), &[], FS as usize / 2, 256);
    assert_eq!(chord(&c)[..3], [hz(57), 0.0, 0.0], "the first voice keeps its note");
    assert!(body(&c, 2).clock > before + 0.4, "the release is rendered");
    run(&mut c, &shared, &voices(1), &[], FS as usize, 256);
    let stopped = body(&c, 2).clock;
    assert!(stopped < before + 1.1, "rendered for {} s after it was taken away", stopped - before);
    run(&mut c, &shared, &voices(1), &[(0, off(61)), (0, off(64)), (100, on(66, 2))], GLIDED, 256);
    assert_eq!(body(&c, 2).clock, stopped);
    assert_eq!(chord(&c)[..3], [hz(66), 0.0, 0.0], "one voice: a second key is legato on it");
    // and back: the keys still down from before start nothing by themselves
    run(&mut c, &shared, &voices(3), &[(100, on(69, 4))], GLIDED, 256);
    assert_eq!(chord(&c)[..3], [hz(66), hz(69), 0.0]);
}

#[test]
fn chords_do_not_depend_on_the_hosts_block_size() {
    let n = 3 * FS as usize;
    for settings in [Settings::default(), Settings { mode: Mode::Play, ..Default::default() }] {
        let shared = Shared::default();
        let reference = run(&mut core_with(&settings, &shared, &voices(4)), &shared, &voices(4), &performance(), n, 512);
        assert!(peak(&reference) > 0.01 && reference.iter().all(|x| x.is_finite()));
        for block in [64, 441, 4096] {
            let out = run(&mut core_with(&settings, &shared, &voices(4)), &shared, &voices(4), &performance(), n, block);
            let first = out.iter().zip(&reference).position(|(a, b)| a.to_bits() != b.to_bits());
            assert_eq!(first, None, "{:?}: blocks of {block} differ from blocks of 512", settings.mode);
        }
    }
}

/// Each voice's own limiter holds a loud vowel at 0.75, so four fingers at full pressure summed to
/// 2.4: over full scale, and a host's output clips there.
#[test]
fn a_loud_chord_stays_below_full_scale() {
    for mode in [Mode::Sing, Mode::Play] {
        let shared = Shared::default();
        let p = VoiceParams { breath: 12.0, ..voices(6) };
        let mut c = core_with(&Settings { mode, ..Default::default() }, &shared, &p);
        let mut events: Vec<(usize, Event)> = [48u8, 55, 60, 64, 67, 72].iter().enumerate().map(|(i, m)| (1000, on(*m, 1 + i as u8))).collect();
        events.extend((1..=6).map(|channel| (3000, Event::Pressure { channel, value: 1.0 })));
        let out = run(&mut c, &shared, &p, &events, 2 * FS as usize, 256);
        let rms = (out.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / out.len() as f64).sqrt();
        assert!(peak(&out) < 1.0 && rms > 0.1, "{mode:?}: peak {}, rms {rms}", peak(&out));
    }
}
