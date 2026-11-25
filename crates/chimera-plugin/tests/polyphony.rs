//! Several voices in the plugin: one for each finger. With one voice it is the instrument it was.
use chimera::phonetics::beatbox::beat_ready;
use chimera_plugin::core::{Core, Event, Mode, Prepared, Settings, Shared, Transport, VoiceParams};

const FS: f64 = 44100.0;

fn params() -> VoiceParams {
    VoiceParams { vibrato: 35.0, vibrato_rate: 6.0, breath: 8.0, pressed: 0.5, bright: 0.25, breathy: 0.15, rough: 0.1, fry: 0.0, glide: 0.06, ring: 0.0, restart: true, mpe: true }
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
