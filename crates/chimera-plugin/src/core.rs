//! The instrument on the audio thread, with no plugin-API types in it, so it can be tested
//! anywhere. [`Shared`] is everything it shares with the editor. See docs/plugin.md.
//!
//! It has [`MAX_VOICES`] voices, each a whole singer (an engine and the note it sings), and the Voices
//! parameter says how many play. A note goes to a voice of its own while there is one, so a chord
//! sounds, and each voice follows its own note's channel: bend, pressure, slide. The first voice is
//! also the one that speaks and beatboxes.
//!
//! Real time: [`Core::begin_block`], [`Core::event`], [`Core::render`], [`Core::end_block`] and
//! [`Core::reset`] don't block, allocate or free. The audio thread only `try_lock`s, and what it
//! replaces it hands back through [`Pending`] for the main thread to free.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU32, AtomicU64, Ordering};

use chimera::anatomy::human::{ArticKey, ArticPatch, BodyMix};
use chimera::control::score::{Gesture, Label};
use chimera::control::singer::{Lyric, Singer, syllables_from};
use chimera::engine::{Engine, SUB_BLOCK};
use chimera::math::{exp, midi_to_hz, pow, tanh};
use chimera::phonetics::Syllable;
use chimera::phonetics::beatbox::{BEAT_PRESETS, BeatStep, parse_beat};
use chimera::phonetics::sing::LiveSyllable;
use chimera::phonetics::syllables::lyrics_to_syllables;
use chimera::voice::human_voice::{HumanVoice, SourcePatch, TELE_LABELS, VoiceTelemetry};
use serde::{Deserialize, Serialize};

use crate::close_mic::CloseMic;
use crate::g2p;
use crate::speech::{SpeechSettings, SpokenLine, plan_lines};

/// The lowest sample rate the engine is stable at (Hz): the airway gives NaN at 22.05 kHz.
pub const MIN_SAMPLE_RATE: f64 = 32000.0;
pub const DEFAULT_LYRICS: &str = "Hello there, I am a voice made of air.";
/// The most voices the Voices parameter can ask for. All are built with the instrument, because the
/// audio thread can't build one.
pub const MAX_VOICES: usize = 8;
/// A note that starts this soon after another (s), on a voice of its own, is part of the same chord:
/// it sings the same syllable.
const CHORD_WINDOW: f64 = 0.05;
/// A voice the Voices parameter takes away is rendered this much longer (s), for its release.
const VOICE_TAIL: f64 = 1.0;
/// [`Core::key_voice`]: no voice has this key.
const NO_VOICE: u8 = u8::MAX;
/// Notes from the on-screen keys waiting for the audio thread.
const MAX_UI_NOTES: usize = 256;
/// Pitch bend range, each way (semitones).
const BEND_RANGE: f64 = 2.0;
/// A note channel's pitch bend range in MPE, each way (semitones): the MPE default.
const MPE_BEND_RANGE: f64 = 48.0;
/// Full pressure raises the breath by this fraction of the Breath parameter.
const PRESSURE_DEPTH: f64 = 0.75;
/// How far a full slide moves the brightness from where the note started.
const SLIDE_DEPTH: f64 = 1.0;
/// A slide this soon after its note starts (s) is where the finger landed. A controller sends the
/// starting slide before the note, but clap-wrapper's VST3 layer passes on one value a block, at
/// the block's last one's time: after the note, by up to a block.
const SLIDE_LANDING: f64 = 0.05;
/// The Breath parameter's maximum (cmH2O): pressure doesn't push past it.
pub const BREATH_MAX: f32 = 22.0;
/// Pressure and brightness glide to a new value with this time constant (s), a step every this
/// many samples.
const EXPR_GLIDE: f64 = 0.012;
const EXPR_STEP: usize = 64;
/// Telemetry snapshots a second, while someone is watching (as the browser's ~60 Hz).
const TELEMETRY_RATE: f64 = 60.0;
/// A poll keeps telemetry flowing for this long (s).
const WATCH_TIME: f64 = 0.25;
/// Output samples kept for the editor's spectrum: more than a poll's worth at any sample rate.
const SCOPE_SAMPLES: usize = 16384;
/// The key that speaks at the body's natural pitch, and how far a key may move it (semitones).
const SPEECH_ROOT: f64 = 60.0;
const SPEECH_RANGE: f64 = 24.0;
/// A spoken line's time 0 comes this long after its note (s), as on the website.
const SPEECH_LEAD: f64 = 0.03;
/// What Beatbox reports as latency, and how far after its point on the timeline each hit lands (s):
/// a little more than the slowest sound's set-up (174 ms). See docs/plugin.md, "Beatbox".
pub const BEAT_LATENCY: f64 = 0.18;
/// Host notes from this one up hum, this many semitones down; below it they are pads.
const HUM_FROM: u8 = 60;
const HUM_DOWN: u8 = 24;
/// The tag of a hum held by a key (the singer's tags are positive), and how long it can last (s).
const HUM_TAG: f64 = -1.0;
const HUM_LEN: f64 = 3600.0;
const HUM: &str = "m";
/// A hit that isn't a step has no next step to last until (s), as on the website.
const BEAT_HIT_LEN: f64 = 0.18;
/// Pads pressed on the page waiting for the audio thread: more are dropped.
const MAX_UI_HITS: usize = 32;
/// [`Shared::spoken_line`]: nothing is being said.
pub const NOT_SPEAKING: i32 = -1;
/// [`Shared::spoken_line`]: the consonant chart's one-shot is being said.
pub const SAYING_PAD: i32 = -2;

/// What the MIDI keys do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// each note sings the next syllable
    #[default]
    Sing,
    /// the keys set the pitch of the posture the user has set, and breathe it
    Play,
    /// each note says the next line
    Speak,
    /// the sequencer follows the host, and notes are pads or a hum
    Beatbox,
}

/// The sound a host note plays in Beatbox: General MIDI's drum kit on the ten mouth sounds.
pub fn drum_pad(midi: u8) -> Option<&'static str> {
    Some(match midi {
        35 | 36 => "B",
        37 => "K",
        38 => "Pf",
        39 => "!",
        40 => "^K",
        41 | 43 | 45 | 47 | 48 | 50 => "P!",
        42 | 44 | 51 | 53 | 59 => "t",
        46 => "tss",
        49 | 52 | 55 | 57 => "kch",
        _ => return None,
    })
}

/// A saved beat.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BeatSettings {
    pub steps: Vec<Option<BeatStep>>,
    pub bpm: f64,
    pub swing: f64,
    /// the example it came from (none once edited)
    pub preset: Option<String>,
    /// the close-mic sound
    pub mic: bool,
    /// the sequencer follows the host's transport
    pub follow: bool,
}

impl Default for BeatSettings {
    fn default() -> Self {
        let p = &BEAT_PRESETS[0];
        BeatSettings { steps: parse_beat(p.text), bpm: p.bpm, swing: p.swing, preset: Some(p.name.to_string()), mic: true, follow: true }
    }
}

/// Who holds a key, in Beatbox: the same number means a different pitch from each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Owner {
    Host,
    Ui,
}

/// A held key humming.
#[derive(Clone, Copy, Debug)]
struct Hum {
    owner: Owner,
    midi: u8,
    /// the pitch it sounds (MIDI)
    pitch: f64,
    accent: f64,
}

/// What a project saves besides the host parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub version: u32,
    pub lyrics: String,
    pub body: BodyMix,
    pub mode: Mode,
    /// the articulators changed from neutral
    pub artic: ArticPatch,
    pub tip_free: bool,
    pub voiced: bool,
    /// the pitch set by hand, in Play mode (Hz)
    pub f0: f64,
    /// where the lyrics restart when the transport starts
    pub start_syllable: usize,
    pub speech: SpeechSettings,
    pub beat: BeatSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { version: 1, lyrics: DEFAULT_LYRICS.to_string(), body: BodyMix::MALE, mode: Mode::Sing, artic: ArticPatch::default(), tip_free: true, voiced: true, f0: 110.0, start_syllable: 0, speech: SpeechSettings::default(), beat: BeatSettings::default() }
    }
}

/// The host parameters' values for one block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoiceParams {
    pub vibrato: f32,
    pub vibrato_rate: f32,
    pub breath: f32,
    pub pressed: f32,
    pub bright: f32,
    pub breathy: f32,
    pub rough: f32,
    pub fry: f32,
    pub glide: f32,
    pub ring: f32,
    pub restart: bool,
    /// MIDI channels 1–15 are MPE note channels, each with its own bend and slide
    pub mpe: bool,
    /// how many voices play (1 to [`MAX_VOICES`])
    pub voices: usize,
}

/// What the host says about the transport this block.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Transport {
    pub playing: bool,
    /// where it is (beats where the host gives them), to notice a loop or a seek
    pub position: Option<f64>,
    /// where it is in beats, and its tempo, if the host says
    pub beats: Option<f64>,
    pub tempo: Option<f64>,
    /// samples in this block
    pub samples: usize,
}

/// A host event, at the point in the block [`Core::event`] is called. Channels are 0–15.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// `velocity` in MIDI units (0–127)
    NoteOn { midi: u8, velocity: f64, channel: u8 },
    NoteOff { midi: u8 },
    /// all notes off, all sound off, a choke
    AllOff,
    /// 0..1, 0.5 centred
    Bend { channel: u8, value: f32 },
    /// channel pressure, 0..1
    Pressure { channel: u8, value: f32 },
    /// poly pressure, 0..1
    KeyPressure { midi: u8, value: f32 },
    /// CC 74, 0..1
    Slide { channel: u8, value: f32 },
}

/// A note from the on-screen keys.
#[derive(Clone, Copy, Debug)]
pub struct UiNote {
    pub midi: u8,
    pub velocity: f64,
    pub on: bool,
}

/// What the editor has asked of the audio thread and it hasn't taken yet. Values merge, so a host
/// that stops processing can't make this grow; only notes queue, and they are bounded.
#[derive(Default)]
pub struct Pending {
    pub artic: ArticPatch,
    pub tip_free: Option<bool>,
    pub source: SourcePatch,
    pub hold: Option<bool>,
    pub mode: Option<Mode>,
    pub cursor: Option<usize>,
    pub speech_cursor: Option<usize>,
    /// say the next line (the Speak button)
    pub speak_next: bool,
    pub stop_speech: bool,
    pub beat_bpm: Option<f64>,
    pub beat_swing: Option<f64>,
    pub beat_playing: Option<bool>,
    pub mic: Option<bool>,
    pub follow_host: Option<bool>,
    beat_steps: Option<Vec<Option<BeatStep>>>,
    hits: Vec<BeatStep>,
    /// a new body: one voice for each of the instrument's
    voice: Option<(Vec<Box<HumanVoice>>, BodyMix)>,
    syllables: Option<Vec<LiveSyllable>>,
    speech: Option<NewSpeech>,
    pad: Option<(Vec<Gesture>, u32)>,
    notes: Vec<UiNote>,
    /// release every key the UI holds (the window closed, or notes were dropped)
    clear_ui_keys: bool,
    /// what the audio thread replaced, for the main thread to free
    retired_voice: Option<Vec<Box<HumanVoice>>>,
    retired_syllables: Option<Vec<LiveSyllable>>,
    retired_speech: Option<Vec<Vec<Gesture>>>,
    retired_pad: Option<Vec<Gesture>>,
    retired_steps: Option<Vec<Option<BeatStep>>>,
    /// the rate a new voice must have (0: not active)
    sample_rate: f64,
}

/// New plans for the spoken lines.
struct NewSpeech {
    plans: Vec<Vec<Gesture>>,
    generation: u32,
    /// the text changed: back to its first line
    reset: bool,
}

/// What's worked out from a project's text, off the audio thread.
pub struct Prepared {
    pub syllables: Vec<Syllable>,
    pub speech: Vec<SpokenLine>,
    pub generation: u32,
}

impl Prepared {
    pub fn of(settings: &Settings, shared: &Shared) -> Prepared {
        Prepared { syllables: lyrics_to_syllables(&settings.lyrics, g2p()), speech: plan_lines(&settings.speech, settings.body, g2p()), generation: shared.next_generation() }
    }
}

/// The spoken lines and the last one-shot, for the editor to show. Each set of plans has a
/// generation, so the page can tell a plan from the one really sounding.
#[derive(Default)]
pub struct SpeechDoc {
    pub lines: Vec<SpokenLine>,
    pub lines_generation: u32,
    pub pad: Option<SpokenLine>,
    pub pad_generation: u32,
}

fn gestures_of(lines: &[SpokenLine]) -> Vec<Vec<Gesture>> {
    lines.iter().map(|l| l.plan.gestures.clone()).collect()
}

/// An f32 in an atomic (bit pattern).
#[derive(Default)]
pub struct AtomicF32(AtomicU32);

impl AtomicF32 {
    pub fn load(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }
    pub fn store(&self, x: f32) {
        self.0.store(x.to_bits(), Ordering::Relaxed)
    }
    pub fn swap(&self, x: f32) -> f32 {
        f32::from_bits(self.0.swap(x.to_bits(), Ordering::Relaxed))
    }
}

/// The latest telemetry, for the editor.
pub struct TelemetryFrame {
    pub block: Vec<f64>,
    pub labels: [u8; TELE_LABELS],
    pub body: BodyMix,
    /// not read yet
    pub fresh: bool,
}

impl Default for TelemetryFrame {
    fn default() -> Self {
        TelemetryFrame { block: Vec::new(), labels: [0; TELE_LABELS], body: BodyMix::MALE, fresh: false }
    }
}

/// What the editor and the audio thread share.
#[derive(Default)]
pub struct Shared {
    pending: Mutex<Pending>,
    dirty: AtomicBool,
    /// held while the project's settings and `pending` change together, so an edit from the page
    /// and a restore can't interleave. Never touched on the audio thread.
    transaction: Mutex<()>,
    pub telemetry: Mutex<TelemetryFrame>,
    /// the output since the editor last read it, for its spectrum (never grows past its capacity)
    pub scope: Mutex<Vec<f32>>,
    /// samples of telemetry still wanted
    watch: AtomicI64,
    pub syllables: Mutex<Vec<Syllable>>,
    /// bumped when `syllables` changes
    pub syllables_version: AtomicU32,
    /// bumped when a state restore replaces everything
    pub revision: AtomicU32,
    pub cursor: AtomicI32,
    /// syllable sounding now (−1: none)
    pub sung: AtomicI32,
    /// MIDI note sounding now (−1: none)
    pub note: AtomicI32,
    /// bumped by every press, so the editor can tell a new note from the same pitch held
    pub note_serial: AtomicU32,
    /// the voice the head shows, and each voice's `note`, `sung` and `note_serial`, for the keys
    /// and the roll (a voice that doesn't play: −1, −1)
    pub shown_voice: AtomicI32,
    pub voice_note: [AtomicI32; MAX_VOICES],
    pub voice_sung: [AtomicI32; MAX_VOICES],
    pub voice_serial: [AtomicU32; MAX_VOICES],
    /// each voice's pitch (Hz) and voicing (0 to 1) now, as f64 bits, for the pitch lines (a voice
    /// that isn't rendered: 0, 0)
    pub voice_f0: [AtomicU64; MAX_VOICES],
    pub voice_voicing: [AtomicU64; MAX_VOICES],
    pub speech: Mutex<SpeechDoc>,
    generations: AtomicU32,
    /// the line a note says next
    pub speech_cursor: AtomicI32,
    /// bumped whenever what's being said changes, and written last: read it first
    pub spoken_serial: AtomicU32,
    /// the line being said ([`NOT_SPEAKING`], [`SAYING_PAD`])
    pub spoken_line: AtomicI32,
    /// the generation of the plan being said
    pub spoken_generation: AtomicU32,
    /// the voice clock at the plan's time 0 (f64 bits)
    pub spoken_t0: AtomicU64,
    /// the step under the playhead (−1: none)
    pub beat_step: AtomicI32,
    /// the pattern is running, and whether that's the host's doing
    pub beat_running: AtomicBool,
    pub beat_following: AtomicBool,
    /// the host's tempo (f64 bits; 0: not known)
    pub host_bpm: AtomicU64,
    pub playing: AtomicBool,
    /// time spent in `process` and audio time rendered since the editor last read them (ns), and
    /// the worst block (a fraction of its duration)
    pub busy_ns: AtomicU64,
    pub audio_ns: AtomicU64,
    pub load_peak: AtomicF32,
    sample_rate: AtomicU64,
    /// the engine stopped (a panic, or a non-finite sample): silent until it is re-created
    pub faulted: AtomicBool,
}

impl Shared {
    fn lock(&self) -> std::sync::MutexGuard<'_, Pending> {
        self.pending.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// See [`Shared::transaction`](#structfield.transaction). Don't call into the host under this.
    pub fn transaction(&self) -> std::sync::MutexGuard<'_, ()> {
        self.transaction.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Change what the audio thread will take next. Main thread.
    pub fn edit(&self, f: impl FnOnce(&mut Pending)) {
        let mut p = self.lock();
        f(&mut p);
        self.dirty.store(true, Ordering::Release);
    }

    /// A note from the on-screen keys. If the queue is full, every UI key lifts instead, so a
    /// dropped release can't leave a note stuck.
    pub fn ui_note(&self, note: UiNote) {
        self.edit(|p| {
            if p.notes.len() < MAX_UI_NOTES {
                p.notes.push(note);
            } else {
                p.notes.clear();
                p.clear_ui_keys = true;
            }
        });
    }

    /// Lift every key the UI holds (the window closed).
    pub fn clear_ui_keys(&self) {
        self.edit(|p| {
            p.notes.clear();
            p.clear_ui_keys = true;
        });
    }

    /// Send a voice built off the audio thread. Dropped if it isn't for the current sample rate.
    pub fn send_voice(&self, voice: Box<HumanVoice>, body: BodyMix) {
        // every voice of the instrument gets the body, whatever the Voices parameter says now. Each
        // is built, not cloned: a clone keeps a buffer's contents but not the room it had reserved
        let mut voices = Vec::with_capacity(MAX_VOICES);
        for _ in 1..MAX_VOICES {
            voices.push(Engine::prepare_voice(voice.fs_out, body));
        }
        voices.push(voice);
        let mut p = self.lock();
        if voices[0].fs_out != p.sample_rate {
            drop(p);
            return;
        }
        let freed = (p.retired_voice.take(), p.voice.replace((voices, body)));
        self.dirty.store(true, Ordering::Release);
        drop(p);
        drop(freed); // outside the lock: the audio thread must not find it busy for this long
    }

    /// New lyrics: for the audio thread, and for the editor to show.
    pub fn send_lyrics(&self, syllables: Vec<Syllable>) {
        let live = syllables_from(&syllables);
        *self.syllables.lock().unwrap_or_else(|e| e.into_inner()) = syllables;
        self.syllables_version.fetch_add(1, Ordering::Relaxed);
        let mut p = self.lock();
        let freed = (p.retired_syllables.take(), p.syllables.replace(live));
        self.dirty.store(true, Ordering::Release);
        drop(p);
        drop(freed);
    }

    /// A number no set of speech plans has had.
    pub fn next_generation(&self) -> u32 {
        self.generations.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// New plans for the lines: for the audio thread, and for the editor to show.
    pub fn send_speech(&self, lines: Vec<SpokenLine>, reset: bool) {
        let generation = self.next_generation();
        let plans = gestures_of(&lines);
        {
            let mut doc = self.speech.lock().unwrap_or_else(|e| e.into_inner());
            doc.lines = lines;
            doc.lines_generation = generation;
        }
        let mut p = self.lock();
        let reset = reset || p.speech.as_ref().is_some_and(|s| s.reset);
        let freed = (p.retired_speech.take(), p.speech.replace(NewSpeech { plans, generation, reset }));
        self.dirty.store(true, Ordering::Release);
        drop(p);
        drop(freed);
    }

    /// A new pattern for the sequencer.
    pub fn send_beat_steps(&self, steps: Vec<Option<BeatStep>>) {
        let mut p = self.lock();
        let freed = (p.retired_steps.take(), p.beat_steps.replace(steps));
        self.dirty.store(true, Ordering::Release);
        drop(p);
        drop(freed);
    }

    /// A pad pressed on the page. If too many are waiting (a host that stopped processing), it's lost.
    pub fn ui_hit(&self, step: BeatStep) {
        self.edit(|p| {
            if p.hits.len() < MAX_UI_HITS {
                p.hits.push(step);
            }
        });
    }

    /// Say this once, now, if Speak is the mode (the consonant chart).
    pub fn send_pad(&self, line: SpokenLine) {
        let generation = self.next_generation();
        let gestures = line.plan.gestures.clone();
        {
            let mut doc = self.speech.lock().unwrap_or_else(|e| e.into_inner());
            doc.pad = Some(line);
            doc.pad_generation = generation;
        }
        let mut p = self.lock();
        let freed = (p.retired_pad.take(), p.pad.replace((gestures, generation)));
        self.dirty.store(true, Ordering::Release);
        drop(p);
        drop(freed);
    }

    /// What was prepared for a new [`Core`], for the editor to show.
    pub fn adopt(&self, prepared: Prepared) {
        *self.syllables.lock().unwrap_or_else(|e| e.into_inner()) = prepared.syllables;
        self.syllables_version.fetch_add(1, Ordering::Relaxed);
        let mut doc = self.speech.lock().unwrap_or_else(|e| e.into_inner());
        doc.lines = prepared.speech;
        doc.lines_generation = prepared.generation;
    }

    /// Start afresh, as `initialize` does: nothing queued is for the new [`Core`].
    pub fn restart(&self, sample_rate: f64) {
        let fresh = Pending { sample_rate, notes: Vec::with_capacity(MAX_UI_NOTES), hits: Vec::with_capacity(MAX_UI_HITS), ..Default::default() };
        let mut p = self.lock();
        let old = std::mem::replace(&mut *p, fresh);
        // under the same lock: an edit published right after must keep its notification
        self.dirty.store(false, Ordering::Release);
        drop(p);
        drop(old);
        let mut scope = self.scope.lock().unwrap_or_else(|e| e.into_inner());
        scope.clear();
        scope.reserve_exact(SCOPE_SAMPLES);
        drop(scope);
        self.sample_rate.store(sample_rate.to_bits(), Ordering::Relaxed);
        self.faulted.store(false, Ordering::Relaxed);
        self.revision.fetch_add(1, Ordering::Relaxed);
    }

    /// The current sample rate (0: not active).
    pub fn sample_rate(&self) -> f64 {
        f64::from_bits(self.sample_rate.load(Ordering::Relaxed))
    }

    /// Keep this block of output for the editor, if someone is looking. On the audio thread: a busy
    /// lock or a slow editor loses some of the display, never time.
    pub fn feed_scope(&self, block: &[f32]) {
        if self.watch.load(Ordering::Relaxed) <= 0 {
            return;
        }
        if let Ok(mut scope) = self.scope.try_lock() {
            let room = scope.capacity() - scope.len();
            scope.extend_from_slice(&block[..block.len().min(room)]);
        }
    }

    /// Someone is looking: collect telemetry for a little longer.
    pub fn watch(&self) {
        self.watch.store((WATCH_TIME * self.sample_rate()) as i64, Ordering::Relaxed);
    }

    /// Account for one block's cost.
    pub fn note_load(&self, busy_ns: u64, samples: usize) {
        let audio = (samples as f64 / self.sample_rate() * 1e9) as u64;
        self.busy_ns.fetch_add(busy_ns, Ordering::Relaxed);
        self.audio_ns.fetch_add(audio, Ordering::Relaxed);
        let used = busy_ns as f32 / audio.max(1) as f32;
        if used > self.load_peak.load() {
            self.load_peak.store(used);
        }
    }
}

/// The limiter on the sum of several voices. Each voice ends in one (`HumanVoice::render`: a loud
/// vowel sits at 0.75), so a chord's peaks add up to well over full scale: 1.5 for two voices at
/// full pressure, 2.4 for four. This is the same law on the sum: a peak envelope held for longer
/// than a glottal period, so a loud chord is turned down by a steady gain and not squared off, and
/// a soft clip for what rises faster. Below its ceiling it does nothing.
struct ChordLimiter {
    env: f64,
    hold: usize,
    hold_n: usize,
    release: f64,
}

impl ChordLimiter {
    const CEILING: f64 = 0.75;
    const KNEE: f64 = 0.85;

    fn new(sample_rate: f64) -> ChordLimiter {
        ChordLimiter { env: 0.0, hold: 0, hold_n: (0.025 * sample_rate).round() as usize, release: exp(-1.0 / (0.2 * sample_rate)) }
    }

    fn process(&mut self, out: &mut [f32]) {
        for o in out {
            let mut v = *o as f64;
            let av = v.abs();
            if av >= self.env {
                self.env = av;
                self.hold = self.hold_n;
            } else if self.hold > 0 {
                self.hold -= 1;
            } else {
                self.env *= self.release;
                // nothing is nothing: no denormals in a silence
                if self.env < 1e-20 {
                    self.env = 0.0;
                }
            }
            if self.env > Self::CEILING {
                v *= Self::CEILING / self.env;
            }
            let al = v.abs();
            if al > Self::KNEE {
                v = v.signum() * (Self::KNEE + (1.0 - Self::KNEE) * tanh((al - Self::KNEE) / (1.0 - Self::KNEE)));
            }
            *o = v as f32;
        }
    }
}

/// One singer: a body, and the note it is singing.
struct Voice {
    engine: Engine,
    /// its notes. The first voice's also holds the lyrics, for all of them
    singer: Singer,
    /// keys down in Play, oldest first
    play_keys: Vec<u8>,
    /// the note whose channel's expression moves this voice, and that channel
    expr_note: Option<u8>,
    expr_channel: u8,
    /// the pressure and brightness its engine has now, gliding to `Core::expression`
    expr_now: (f64, f64),
    expr_moving: bool,
    /// when its newest note started, or when its last key lifted if none is down (`Core::order`)
    order: u64,
    /// presses it has taken (see [`Shared::voice_serial`])
    presses: u32,
}

pub struct Core {
    /// every voice the instrument can have: the first `count` play
    voices: Vec<Voice>,
    count: usize,
    /// voices rendered: `count`, and for `tail_left` samples more the ones just taken away
    rendered: usize,
    tail_left: usize,
    /// on the sum, while more than one voice is rendered
    limiter: ChordLimiter,
    /// the voice each key down is on (`NO_VOICE`: none), in Sing and Play
    key_voice: [u8; 128],
    /// counts the notes started and the voices freed
    order: u64,
    /// the voice with the newest note sounding: the one the editor shows
    shown: usize,
    /// the syllable the last note sang, and the samples left in which a note joins its chord
    chord: Option<Lyric>,
    chord_left: usize,
    mode: Mode,
    params: VoiceParams,
    /// the Hold breath toggle (Play): the first voice breathes with no key down
    hold: bool,
    /// keys down per owner, as bit masks: a pitch lifts when its last owner lets go
    host_keys: u128,
    ui_keys: u128,
    start_syllable: usize,
    transport: Transport,
    tele: VoiceTelemetry,
    block: Vec<f64>,
    labels: [u8; TELE_LABELS],
    /// presses so far (see [`Shared::note_serial`])
    presses: u32,
    /// each line's gestures, and the generation of those plans
    speech: Vec<Vec<Gesture>>,
    speech_generation: u32,
    speech_cursor: usize,
    speech_start: usize,
    /// the last one-shot, kept until the main thread can free it
    pad: Vec<Gesture>,
    /// what's being said, as [`Shared`] shows it
    spoken_serial: u32,
    spoken_line: i32,
    spoken_generation: u32,
    spoken_t0: f64,
    /// pitch ratio from the key that said this line
    key_bend: f64,
    /// each channel's bend, pressure and slide, and its slide when its last note started
    bend: [f32; 16],
    pressure: [f32; 16],
    slide: [f32; 16],
    slide_origin: [f32; 16],
    /// samples left of each channel's landing (`SLIDE_LANDING` after a note starts, `landing`
    /// samples): a slide in it is where the finger landed, not a move
    slide_landing: [usize; 16],
    landing: usize,
    /// the last bend from any channel: the wheel when MPE is off
    last_bend: f32,
    /// the channel each key last started on (the on-screen keys: 0)
    note_channel: [u8; 128],
    /// the last note started (Speak, Beatbox)
    last_on: Option<u8>,
    /// each key's poly pressure
    key_pressure: [f32; 128],
    /// Each voice's pressure and brightness glide to its `expression()`: a step every `EXPR_STEP`
    /// samples (`expr_left` to the next), each `expr_coef` of the way left. `expr_moving`: some
    /// voice's are on their way
    expr_moving: bool,
    expr_left: usize,
    expr_coef: f64,
    /// the sequencer is following the host's transport
    following: bool,
    follow_host: bool,
    /// the host is playing, as of this block: what was queued before it starts is for its timeline
    host_playing: bool,
    /// the user's tempo: following the host overwrites the sequencer's
    beat_bpm: f64,
    mic_on: bool,
    close_mic: CloseMic,
    /// keys humming in Beatbox, oldest first
    hums: Vec<Hum>,
    /// samples until the next telemetry snapshot
    tele_due: i64,
    tele_every: i64,
}

impl Core {
    /// Build the instrument from a project's state. Allocates: not on the audio thread.
    pub fn new(sample_rate: f64, settings: &Settings, prepared: &Prepared, params: VoiceParams, shared: &Shared) -> Core {
        // a new male engine is the largest airway at this rate, so telemetry sized for it fits any
        let engines: Vec<Engine> = (0..MAX_VOICES).map(|_| Engine::new(sample_rate)).collect();
        let mut tele = VoiceTelemetry::default();
        let mut block = Vec::new();
        let info = engines[0].info();
        tele.reserve(info.n_ph + info.n_or, &mut block);
        // the other buffer of the swap with the editor
        let mut frame = shared.telemetry.lock().unwrap_or_else(|e| e.into_inner());
        frame.block = Vec::with_capacity(block.capacity());
        frame.fresh = false;
        drop(frame);

        let mut voices: Vec<Voice> = engines
            .into_iter()
            .map(|mut engine| {
                engine.set_body(settings.body);
                let mut singer = Singer::new();
                singer.body = settings.body;
                Voice { engine, singer, play_keys: Vec::with_capacity(128), expr_note: None, expr_channel: 0, expr_now: (0.0, 0.0), expr_moving: false, order: 0, presses: 0 }
            })
            .collect();
        let _ = voices[0].singer.set_syllables(syllables_from(&prepared.syllables));
        voices[0].singer.set_cursor(settings.start_syllable);
        let count = params.voices.clamp(1, MAX_VOICES);
        let mut core = Core {
            voices,
            count,
            rendered: count,
            tail_left: 0,
            limiter: ChordLimiter::new(sample_rate),
            key_voice: [NO_VOICE; 128],
            order: 0,
            shown: 0,
            chord: None,
            chord_left: 0,
            mode: settings.mode,
            params,
            hold: false,
            host_keys: 0,
            ui_keys: 0,
            start_syllable: settings.start_syllable,
            transport: Transport::default(),
            tele,
            block,
            labels: [0; TELE_LABELS],
            presses: 0,
            speech_cursor: settings.speech.start_line.min(prepared.speech.len().saturating_sub(1)),
            speech_start: settings.speech.start_line.min(prepared.speech.len().saturating_sub(1)),
            speech: gestures_of(&prepared.speech),
            speech_generation: prepared.generation,
            pad: Vec::new(),
            spoken_serial: shared.spoken_serial.load(Ordering::Relaxed).wrapping_add(1),
            spoken_line: NOT_SPEAKING,
            spoken_generation: 0,
            spoken_t0: 0.0,
            key_bend: 1.0,
            bend: [0.5; 16],
            pressure: [0.0; 16],
            slide: [0.5; 16],
            slide_origin: [0.5; 16],
            slide_landing: [0; 16],
            landing: (SLIDE_LANDING * sample_rate) as usize,
            last_bend: 0.5,
            note_channel: [0; 128],
            last_on: None,
            key_pressure: [0.0; 128],
            expr_moving: false,
            expr_left: 0,
            expr_coef: 1.0 - exp(-(EXPR_STEP as f64 / sample_rate) / EXPR_GLIDE),
            following: false,
            follow_host: settings.beat.follow,
            host_playing: false,
            beat_bpm: settings.beat.bpm,
            mic_on: settings.beat.mic,
            close_mic: CloseMic::new(sample_rate),
            hums: Vec::with_capacity(2 * 128),
            tele_due: 0,
            tele_every: (sample_rate / TELEMETRY_RATE) as i64,
        };
        for voice in &mut core.voices {
            voice.engine.set_source(&SourcePatch { f0: Some(settings.f0), voiced: Some(settings.voiced), ..Default::default() });
            voice.engine.set_artic(&settings.artic, Some(settings.tip_free));
        }
        core.lead().set_beat_steps(&settings.beat.steps);
        core.lead().set_beat_tempo(Some(settings.beat.bpm), Some(settings.beat.swing));
        core.apply_params();
        core.settle_expression();
        // a voice's first control step sizes some of its buffers: not on the audio thread
        for voice in &mut core.voices {
            voice.engine.render(&mut [0.0; SUB_BLOCK]);
        }
        core
    }

    /// The first voice's engine: the one that also speaks and beatboxes.
    pub fn engine(&self) -> &Engine {
        &self.voices[0].engine
    }

    fn lead(&mut self) -> &mut Engine {
        &mut self.voices[0].engine
    }

    /// The first voice's singer: the one with the lyrics.
    pub fn singer(&self) -> &Singer {
        &self.voices[0].singer
    }

    /// One of the [`MAX_VOICES`] voices' engine and singer.
    pub fn voice(&self, v: usize) -> (&Engine, &Singer) {
        (&self.voices[v].engine, &self.voices[v].singer)
    }

    /// The voice the editor shows: the one with the newest note sounding.
    pub fn shown(&self) -> usize {
        self.shown
    }

    /// The line a note says next.
    pub fn speech_cursor(&self) -> usize {
        self.speech_cursor
    }

    /// The latency to report to the host: only Beatbox has any.
    pub fn latency_samples(&self) -> u32 {
        if self.mode == Mode::Beatbox { (BEAT_LATENCY * self.engine().sample_rate).round() as u32 } else { 0 }
    }

    fn apply_params(&mut self) {
        let p = self.params;
        let count = p.voices.clamp(1, MAX_VOICES);
        if count != self.count {
            self.set_count(count);
        }
        // while singing, the notes' own gestures carry the vibrato: a base depth would shake the onset
        let depth = if self.mode == Mode::Play { p.vibrato as f64 } else { 0.0 };
        for voice in &mut self.voices {
            voice.singer.vibrato = p.vibrato as f64;
            // the Breath and Bright dials glide, with the pressure and slide on them (`render`)
            voice.engine.set_source(&SourcePatch {
                vibrato_depth: Some(depth),
                vibrato_rate: Some(p.vibrato_rate as f64),
                tension: Some(p.pressed as f64),
                breathiness: Some(p.breathy as f64),
                roughness: Some(p.rough as f64),
                fry: Some(p.fry as f64),
                glide: Some(p.glide as f64),
                ..Default::default()
            });
            voice.engine.set_artic(&ArticPatch::default().with(ArticKey::Epilarynx, p.ring as f64), None);
        }
        self.glide_all();
        // the MPE switch changes which channels bend what
        self.update_bend();
    }

    /// The Voices parameter moved. A voice taken away stops its notes, and is rendered a little
    /// longer for their release.
    fn set_count(&mut self, count: usize) {
        for voice in self.voices[..self.count].iter_mut().skip(count) {
            voice.singer.stop_all(&mut voice.engine);
            voice.play_keys.clear();
        }
        for v in &mut self.key_voice {
            if *v as usize >= count {
                *v = NO_VOICE;
            }
        }
        if count < self.count {
            self.tail_left = (VOICE_TAIL * self.engine().sample_rate) as usize;
        }
        // a voice that wasn't rendered starts on the first voice's clock: the editor puts every
        // voice's notes on one timeline, and shows one voice's telemetry at a time
        let clock = self.voices[0].engine.clock();
        for voice in self.voices[..count].iter_mut().skip(self.rendered) {
            voice.engine.set_clock(clock);
        }
        self.rendered = self.rendered.max(count);
        self.count = count;
        if self.shown >= count {
            self.shown = 0;
        }
        self.update_gate();
    }

    /// Each engine's bend: the key's transposition, the wheel, and its sounding note's own bend.
    fn update_bend(&mut self) {
        let mpe = self.params.mpe;
        let wheel = if mpe { self.bend[0] } else { self.last_bend };
        for voice in &mut self.voices {
            let ch = voice.expr_channel as usize;
            let note = if mpe && ch != 0 { pow(2.0, (self.bend[ch] as f64 - 0.5) * 2.0 * MPE_BEND_RANGE / 12.0) } else { 1.0 };
            voice.engine.set_bend(self.key_bend * pow(2.0, (wheel as f64 - 0.5) * 2.0 * BEND_RANGE / 12.0) * note);
        }
    }

    /// A voice's lung pressure and brightness, with its sounding note's pressure and slide on the dials.
    fn expression(&self, voice: &Voice) -> (f64, f64) {
        let ch = voice.expr_channel as usize;
        let press = self.pressure[ch].max(voice.expr_note.map_or(0.0, |note| self.key_pressure[note as usize])) as f64;
        let breath = (self.params.breath as f64 * (1.0 + PRESSURE_DEPTH * press)).min(BREATH_MAX as f64);
        let slide = if self.params.mpe { (self.slide[ch] - self.slide_origin[ch]) as f64 * SLIDE_DEPTH } else { 0.0 };
        (breath, (self.params.bright as f64 + slide).clamp(0.0, 1.0))
    }

    /// A voice's pressure, slide or their dials moved: glide there. Controllers come in steps (7-bit
    /// ones, and through the VST3 layer once a block), which the voice would otherwise repeat.
    fn glide_expression(&mut self, v: usize) {
        if !self.expr_moving {
            self.expr_moving = true;
            self.expr_left = 0;
        }
        self.voices[v].expr_moving = true;
    }

    fn glide_all(&mut self) {
        for v in 0..self.voices.len() {
            self.glide_expression(v);
        }
    }

    /// One step of the glide, for each voice on its way.
    fn step_expression(&mut self) {
        self.expr_moving = false;
        for v in 0..self.voices.len() {
            if !self.voices[v].expr_moving {
                continue;
            }
            let (pressure, brightness) = self.expression(&self.voices[v]);
            let coef = self.expr_coef;
            let voice = &mut self.voices[v];
            let (dp, db) = (pressure - voice.expr_now.0, brightness - voice.expr_now.1);
            if dp.abs() < 0.01 && db.abs() < 0.001 {
                voice.expr_now = (pressure, brightness);
                voice.expr_moving = false;
            } else {
                voice.expr_now.0 += dp * coef;
                voice.expr_now.1 += db * coef;
                self.expr_moving = true;
            }
            voice.engine.set_source(&SourcePatch { pressure: Some(voice.expr_now.0), brightness: Some(voice.expr_now.1), ..Default::default() });
        }
    }

    /// Put each voice's pressure and brightness where they are going, at once.
    fn settle_expression(&mut self) {
        for v in 0..self.voices.len() {
            let now = self.expression(&self.voices[v]);
            let voice = &mut self.voices[v];
            voice.expr_now = now;
            voice.expr_moving = false;
            voice.engine.set_source(&SourcePatch { pressure: Some(now.0), brightness: Some(now.1), ..Default::default() });
        }
        self.expr_moving = false;
    }

    /// The note now sounding on a voice: its channel's expression moves that voice. With nothing
    /// sounding the last one's channel stays, so a release tail keeps following it.
    fn sounding(&self, v: usize) -> Option<u8> {
        let voice = &self.voices[v];
        match self.mode {
            Mode::Sing => voice.singer.sounding_note(),
            Mode::Play => voice.play_keys.last().copied(),
            Mode::Speak | Mode::Beatbox if v == 0 => self.last_on,
            Mode::Speak | Mode::Beatbox => None,
        }
    }

    /// A key starts a note on this channel (before it is pressed).
    fn note_started(&mut self, midi: u8, channel: u8) {
        let ch = channel as usize;
        self.note_channel[midi as usize] = channel;
        self.slide_origin[ch] = self.slide[ch];
        self.slide_landing[ch] = self.landing;
        // a controller needn't send a closing zero
        self.key_pressure[midi as usize] = 0.0;
        self.last_on = Some(midi);
    }

    /// A key went down or up: each voice follows the note now sounding on it.
    fn follow_expression(&mut self) {
        let mut newest = None;
        for v in 0..self.voices.len() {
            let Some(note) = self.sounding(v) else { continue };
            let voice = &mut self.voices[v];
            voice.expr_note = Some(note);
            voice.expr_channel = self.note_channel[note as usize];
            if newest.is_none_or(|(order, _)| voice.order >= order) {
                newest = Some((voice.order, v));
            }
            // the same key on the same channel still starts its slide over
            self.glide_expression(v);
        }
        if let Some((_, v)) = newest {
            self.shown = v;
            self.update_bend();
        }
    }

    /// Every channel's bend, pressure and slide back to rest.
    fn clear_expression(&mut self) {
        self.bend = [0.5; 16];
        self.last_bend = 0.5;
        self.pressure = [0.0; 16];
        self.slide = [0.5; 16];
        self.slide_origin = [0.5; 16];
        self.key_pressure = [0.0; 128];
        self.update_bend();
        self.glide_all();
    }

    fn update_gate(&mut self) {
        for (v, voice) in self.voices.iter_mut().enumerate() {
            let on = self.mode == Mode::Play && ((self.hold && v == 0) || !voice.play_keys.is_empty());
            voice.engine.set_gate(if on { 1.0 } else { 0.0 });
        }
    }

    /// Whether a voice has a key down (Sing, Play).
    fn busy(&self, v: usize) -> bool {
        match self.mode {
            Mode::Sing => self.voices[v].singer.any_key_down(),
            _ => !self.voices[v].play_keys.is_empty(),
        }
    }

    /// The voice a new note goes to, and whether it was free:
    /// - the voice that has this key already (a repeat);
    /// - or the free voice still following the note's channel, so one finger's notes stay on one
    ///   voice and another voice's release doesn't hear this finger;
    /// - or the voice free for the longest;
    /// - or, with every voice busy, the one whose note is the oldest. It gets the note as a single
    ///   voice gets a second key: legato, and back to the older key when this one lifts.
    fn take_voice(&mut self, midi: u8) -> (usize, bool) {
        let channel = self.note_channel[midi as usize];
        let held = self.key_voice[midi as usize] as usize;
        let oldest = |free: bool| (0..self.count).filter(|v| self.busy(*v) != free).min_by_key(|v| self.voices[*v].order);
        let v = if held < self.count {
            held
        } else if self.params.mpe && channel != 0 && let Some(v) = (0..self.count).find(|v| self.voices[*v].expr_channel == channel && !self.busy(*v)) {
            v
        } else {
            oldest(true).or_else(|| oldest(false)).unwrap_or(0)
        };
        let free = !self.busy(v);
        self.key_voice[midi as usize] = v as u8;
        self.order += 1;
        self.voices[v].order = self.order;
        self.voices[v].presses = self.voices[v].presses.wrapping_add(1);
        (v, free)
    }

    fn press(&mut self, midi: u8, velocity: f64) {
        self.presses = self.presses.wrapping_add(1);
        match self.mode {
            Mode::Sing => {
                let (v, free) = self.take_voice(midi);
                // a chord sings one syllable: a note on a voice of its own, just after another
                let lyric = match self.chord {
                    Some(lyric) if free && self.count > 1 && self.chord_left > 0 => lyric,
                    _ => self.voices[0].singer.next_lyric(),
                };
                self.chord = Some(lyric);
                self.chord_left = (CHORD_WINDOW * self.engine().sample_rate) as usize;
                let voice = &mut self.voices[v];
                voice.singer.sing(&mut voice.engine, &lyric, midi, velocity);
            }
            Mode::Play => {
                let (v, free) = self.take_voice(midi);
                let voice = &mut self.voices[v];
                voice.play_keys.retain(|&k| k != midi);
                voice.play_keys.push(midi);
                voice.engine.set_source(&SourcePatch::f0(midi_to_hz(midi as f64)));
                // one of several voices starts from silence: at its pitch, with its own channel's
                // bend, not sliding in from the last note it sang
                if free && self.count > 1 && !(self.hold && v == 0) {
                    voice.expr_channel = self.note_channel[midi as usize];
                    self.update_bend();
                    self.voices[v].engine.snap_pitch();
                }
                self.update_gate();
            }
            Mode::Speak => {
                let semitones = (midi as f64 - SPEECH_ROOT).clamp(-SPEECH_RANGE, SPEECH_RANGE);
                self.say_next(pow(2.0, semitones / 12.0));
            }
            // by owner: see `beat_key`
            Mode::Beatbox => {}
        }
    }

    fn lift(&mut self, midi: u8) {
        // a line is a one-shot: it ends by itself, or when the next one cuts it
        if matches!(self.mode, Mode::Speak | Mode::Beatbox) {
            return;
        }
        let v = self.key_voice[midi as usize] as usize;
        if v >= self.voices.len() {
            return;
        }
        self.key_voice[midi as usize] = NO_VOICE;
        if self.mode == Mode::Sing {
            let melisma_next = self.voices[0].singer.next_is_melisma();
            let voice = &mut self.voices[v];
            voice.singer.lift(&mut voice.engine, midi, melisma_next);
        } else {
            let voice = &mut self.voices[v];
            voice.play_keys.retain(|&k| k != midi);
            // fall back to the most recent note still held (legato)
            if let Some(&last) = voice.play_keys.last() {
                voice.engine.set_source(&SourcePatch::f0(midi_to_hz(last as f64)));
            }
            self.update_gate();
        }
        if !self.busy(v) {
            self.order += 1;
            self.voices[v].order = self.order;
        }
    }

    /// Stop everything the keys are doing.
    fn stop_notes(&mut self) {
        for voice in &mut self.voices {
            voice.singer.stop_all(&mut voice.engine);
            voice.play_keys.clear();
        }
        self.key_voice = [NO_VOICE; 128];
        self.host_keys = 0;
        self.ui_keys = 0;
        if self.mode == Mode::Speak {
            self.stop_speaking();
        }
        if self.mode == Mode::Beatbox {
            self.hums.clear();
            let now = self.engine().clock();
            self.lead().release_at(HUM_TAG, now);
        }
        self.update_gate();
    }

    /// When a sound asked for now should land: `BEAT_LATENCY` on, while the host's timeline is
    /// running and it compensates; otherwise as soon as the mouth can make it.
    fn beat_time(&mut self, id: &str) -> f64 {
        let wait = if self.host_playing { BEAT_LATENCY } else { self.lead().beat_lead(id) };
        self.engine().clock() + wait
    }

    /// A sound from a pad, or from a host note on the drum map.
    fn beat_hit(&mut self, step: BeatStep) {
        let hit = self.beat_time(step.id.as_str());
        self.voices[0].engine.beat_hit_at(&step, hit, BEAT_HIT_LEN, None);
    }

    /// Hum this key's pitch, in place of whatever was humming.
    fn start_hum(&mut self, hum: Hum) {
        let hit = self.beat_time(HUM);
        self.voices[0].engine.release_at(HUM_TAG, hit);
        let step = BeatStep { id: Label::new(HUM), note: Some(hum.pitch), accent: Some(hum.accent) };
        self.voices[0].engine.beat_hit_at(&step, hit, HUM_LEN, Some(HUM_TAG));
    }

    /// The hums changed: the newest key held hums, or nothing does. `top` is what was humming.
    fn resolve_hums(&mut self, top: Option<(Owner, u8)>) {
        let now = self.hums.last().copied();
        match now {
            Some(h) if top != Some((h.owner, h.midi)) => self.start_hum(h),
            Some(_) => {}
            None if top.is_some() => {
                let t = self.beat_time(HUM);
                self.voices[0].engine.release_at(HUM_TAG, t);
            }
            None => {}
        }
    }

    /// A key went down or up in Beatbox. The two owners' numbers mean different pitches, so the
    /// shared press and release masks don't apply here.
    fn beat_key(&mut self, owner: Owner, midi: u8, velocity: f64, on: bool) {
        let accent = (velocity / 100.0).clamp(0.4, 1.25);
        let pitch = match owner {
            Owner::Ui => midi as f64,
            Owner::Host if midi >= HUM_FROM => (midi - HUM_DOWN) as f64,
            Owner::Host => {
                if on && let Some(id) = drum_pad(midi) {
                    self.presses = self.presses.wrapping_add(1);
                    self.beat_hit(BeatStep { id: Label::new(id), note: None, accent: Some(accent) });
                }
                return;
            }
        };
        let top = self.hums.last().map(|h| (h.owner, h.midi));
        self.hums.retain(|h| !(h.owner == owner && h.midi == midi));
        if on {
            self.presses = self.presses.wrapping_add(1);
            self.hums.push(Hum { owner, midi, pitch, accent });
            // a repeat of the key humming is a new note
            if top == Some((owner, midi)) {
                let h = self.hums[self.hums.len() - 1];
                self.start_hum(h);
                return;
            }
        }
        self.resolve_hums(top);
    }

    /// Follow the host's transport with the sequencer, or stop following it.
    fn follow_beat(&mut self, t: Transport) {
        let host = if self.mode == Mode::Beatbox && self.follow_host && t.playing { t.beats.zip(t.tempo).filter(|(_, bpm)| *bpm > 0.0) } else { None };
        match host {
            Some((beats, bpm)) => {
                if !self.following {
                    self.voices[0].engine.set_beat_playing(false);
                    self.following = true;
                }
                let end = beats + t.samples as f64 / self.engine().sample_rate * bpm / 60.0;
                self.voices[0].engine.beat_follow(beats, end, bpm, BEAT_LATENCY);
            }
            None if self.following => self.stop_following(),
            None => {}
        }
    }

    fn stop_following(&mut self) {
        self.following = false;
        self.voices[0].engine.beat_stop_following();
        self.voices[0].engine.set_beat_tempo(Some(self.beat_bpm), None);
        self.hum_again();
    }

    /// Stopping the sequencer clears the score, a hum still held with it.
    fn hum_again(&mut self) {
        if let Some(h) = self.hums.last().copied() {
            self.start_hum(h);
        }
    }

    fn mark_spoken(&mut self, line: i32, generation: u32, t0: f64) {
        self.spoken_serial = self.spoken_serial.wrapping_add(1);
        self.spoken_line = line;
        self.spoken_generation = generation;
        self.spoken_t0 = t0;
    }

    /// Say the next line, at this pitch ratio. The whole contour moves through the bend, so a plan
    /// isn't rebuilt per note.
    fn say_next(&mut self, key_bend: f64) {
        if self.speech_cursor >= self.speech.len() {
            return;
        }
        self.key_bend = key_bend;
        self.update_bend();
        let t0 = self.voices[0].engine.schedule(&self.speech[self.speech_cursor], SPEECH_LEAD, true);
        self.mark_spoken(self.speech_cursor as i32, self.speech_generation, t0);
        self.speech_cursor = (self.speech_cursor + 1) % self.speech.len();
    }

    fn stop_speaking(&mut self) {
        self.voices[0].engine.stop_speech();
        if self.spoken_line != NOT_SPEAKING {
            self.mark_spoken(NOT_SPEAKING, 0, 0.0);
        }
        self.key_bend = 1.0;
        self.update_bend();
    }

    fn lift_ui_keys(&mut self) {
        if self.mode == Mode::Beatbox {
            let top = self.hums.last().map(|h| (h.owner, h.midi));
            self.hums.retain(|h| h.owner != Owner::Ui);
            self.resolve_hums(top);
            return;
        }
        let mine = self.ui_keys;
        self.ui_keys = 0;
        for midi in 0..128u8 {
            let bit = 1u128 << midi;
            if mine & bit != 0 && self.host_keys & bit == 0 {
                self.lift(midi);
            }
        }
    }

    /// Take what the editor has sent, if the lock is free.
    fn intake(&mut self, shared: &Shared) {
        if !shared.dirty.load(Ordering::Acquire) {
            return;
        }
        let Ok(mut p) = shared.pending.try_lock() else { return };
        shared.dirty.store(false, Ordering::Release);
        if let Some((mut bodies, body)) = p.voice.take() {
            // the new voice has no score: a sung note would hang, and its coda has nowhere to go
            self.stop_notes();
            for (voice, new) in self.voices.iter_mut().zip(bodies.iter_mut()) {
                voice.engine.install_voice(new, body);
                voice.singer.body = body;
            }
            p.retired_voice = Some(bodies);
        }
        if let Some(syllables) = p.syllables.take() {
            p.retired_syllables = Some(self.voices[0].singer.set_syllables(syllables));
            self.start_syllable = 0;
        }
        if let Some(new) = p.speech.take() {
            p.retired_speech = Some(std::mem::replace(&mut self.speech, new.plans));
            self.speech_generation = new.generation;
            let last = self.speech.len().saturating_sub(1);
            self.speech_cursor = if new.reset { 0 } else { self.speech_cursor.min(last) };
            self.speech_start = if new.reset { 0 } else { self.speech_start.min(last) };
        }
        if let Some(mode) = p.mode.take()
            && mode != self.mode
        {
            self.stop_notes();
            if self.mode == Mode::Beatbox {
                self.stop_following();
                self.voices[0].engine.set_beat_playing(false);
                // a pad's hit may still be on its way, with no sequencer to have cancelled it
                self.voices[0].engine.stop_speech();
            }
            self.mode = mode;
            // speech and the beat are the first voice's, with or without a note
            self.shown = 0;
            self.hold = false;
            self.update_gate();
            self.apply_params();
        }
        if p.source != SourcePatch::default() {
            for voice in &mut self.voices {
                voice.engine.set_source(&p.source);
            }
            p.source = SourcePatch::default();
        }
        if !p.artic.is_empty() || p.tip_free.is_some() {
            for voice in &mut self.voices {
                voice.engine.set_artic(&p.artic, p.tip_free);
            }
            p.artic = ArticPatch::default();
            p.tip_free = None;
        }
        if let Some(hold) = p.hold.take() {
            self.hold = hold;
            self.update_gate();
        }
        if let Some(i) = p.cursor.take() {
            self.voices[0].singer.set_cursor(i);
            // picked while stopped: that is where playing starts from
            if !self.transport.playing {
                self.start_syllable = self.singer().cursor();
            }
        }
        if let Some(i) = p.speech_cursor.take() {
            self.speech_cursor = i.min(self.speech.len().saturating_sub(1));
            if !self.transport.playing {
                self.speech_start = self.speech_cursor;
            }
        }
        if let Some((gestures, generation)) = p.pad.take() {
            let old = std::mem::replace(&mut self.pad, gestures);
            p.retired_pad = Some(old);
            if self.mode == Mode::Speak {
                self.key_bend = 1.0;
                self.update_bend();
                let t0 = self.voices[0].engine.schedule(&self.pad, SPEECH_LEAD, true);
                self.mark_spoken(SAYING_PAD, generation, t0);
            }
        }
        if let Some(steps) = p.beat_steps.take() {
            self.voices[0].engine.set_beat_steps(&steps);
            p.retired_steps = Some(steps);
        }
        if p.beat_bpm.is_some() || p.beat_swing.is_some() {
            self.beat_bpm = p.beat_bpm.unwrap_or(self.beat_bpm);
            self.voices[0].engine.set_beat_tempo(p.beat_bpm.take(), p.beat_swing.take());
        }
        if let Some(on) = p.mic.take() {
            self.mic_on = on;
        }
        if let Some(on) = p.follow_host.take() {
            self.follow_host = on;
        }
        if let Some(on) = p.beat_playing.take()
            && self.mode == Mode::Beatbox
            && !self.following
        {
            let was = self.voices[0].engine.beat().playing;
            self.voices[0].engine.set_beat_playing(on);
            if was && !on {
                self.hum_again();
            }
        }
        let Pending { hits, .. } = &mut *p;
        for step in hits.drain(..) {
            if self.mode == Mode::Beatbox {
                self.beat_hit(step);
            }
        }
        if std::mem::take(&mut p.speak_next) && self.mode == Mode::Speak {
            self.say_next(1.0);
        }
        if std::mem::take(&mut p.stop_speech) && self.mode == Mode::Speak {
            self.stop_speaking();
        }
        if p.clear_ui_keys {
            p.clear_ui_keys = false;
            self.lift_ui_keys();
            self.follow_expression();
        }
        let Pending { notes, .. } = &mut *p;
        for n in notes.drain(..) {
            if self.mode == Mode::Beatbox {
                if n.on {
                    self.note_started(n.midi & 127, 0);
                }
                self.beat_key(Owner::Ui, n.midi & 127, n.velocity, n.on);
                self.follow_expression();
                continue;
            }
            let bit = 1u128 << (n.midi & 127);
            if n.on {
                self.ui_keys |= bit;
                // a key the host already holds stays the host's note, on its channel
                if self.host_keys & bit == 0 {
                    self.note_started(n.midi & 127, 0);
                    self.press(n.midi & 127, n.velocity);
                }
            } else {
                let held = self.ui_keys & bit != 0;
                self.ui_keys &= !bit;
                if held && self.host_keys & bit == 0 {
                    self.lift(n.midi & 127);
                }
            }
            self.follow_expression();
        }
    }

    fn follow_transport(&mut self, t: Transport, shared: &Shared) {
        let was = self.transport;
        let started = t.playing && !was.playing;
        let jumped = t.playing && was.playing && matches!((t.position, was.position), (Some(now), Some(last)) if now < last - 1e-3);
        if (started || jumped) && self.params.restart {
            self.voices[0].singer.set_cursor(self.start_syllable);
            self.speech_cursor = self.speech_start;
        }
        if was.playing && !t.playing {
            self.stop_notes();
        }
        self.transport = t;
        self.follow_beat(t);
        shared.playing.store(t.playing, Ordering::Relaxed);
    }

    /// Start of a block: what the editor sent, the host parameters, the transport.
    pub fn begin_block(&mut self, params: &VoiceParams, transport: Transport, shared: &Shared) {
        self.host_playing = transport.playing;
        self.intake(shared);
        if *params != self.params {
            self.params = *params;
            self.apply_params();
        }
        self.follow_transport(transport, shared);
    }

    /// A host event. Render up to its sample first.
    pub fn event(&mut self, ev: Event) {
        match ev {
            Event::NoteOn { midi, velocity, channel } => {
                let midi = midi & 127;
                let bit = 1u128 << midi;
                if self.mode == Mode::Beatbox {
                    self.note_started(midi, channel & 15);
                    self.beat_key(Owner::Host, midi, velocity, true);
                } else {
                    self.host_keys |= bit;
                    // a repeat from the same owner is a new note; a key the UI already holds isn't
                    if self.ui_keys & bit == 0 {
                        self.note_started(midi, channel & 15);
                        self.press(midi, velocity);
                    }
                }
                self.follow_expression();
            }
            Event::NoteOff { midi } => {
                let midi = midi & 127;
                let bit = 1u128 << midi;
                if self.mode == Mode::Beatbox {
                    self.beat_key(Owner::Host, midi, 0.0, false);
                } else {
                    let held = self.host_keys & bit != 0;
                    self.host_keys &= !bit;
                    if held && self.ui_keys & bit == 0 {
                        self.lift(midi);
                    }
                }
                self.follow_expression();
            }
            // not a controller reset: the wheel stays where it is
            Event::AllOff => self.stop_notes(),
            Event::Bend { channel, value } => {
                self.bend[(channel & 15) as usize] = value;
                self.last_bend = value;
                self.update_bend();
            }
            Event::Pressure { channel, value } => {
                self.pressure[(channel & 15) as usize] = value;
                self.glide_where(|voice| channel & 15 == voice.expr_channel);
            }
            Event::KeyPressure { midi, value } => {
                self.key_pressure[(midi & 127) as usize] = value;
                self.glide_where(|voice| Some(midi & 127) == voice.expr_note);
            }
            Event::Slide { channel, value } => {
                let ch = (channel & 15) as usize;
                self.slide[ch] = value;
                if self.slide_landing[ch] > 0 {
                    self.slide_origin[ch] = value;
                }
                if self.params.mpe {
                    self.glide_where(|voice| channel & 15 == voice.expr_channel);
                }
            }
        }
    }

    /// Glide the expression of the voices this is true of.
    fn glide_where(&mut self, moved: impl Fn(&Voice) -> bool) {
        for v in 0..self.voices.len() {
            if moved(&self.voices[v]) {
                self.glide_expression(v);
            }
        }
    }

    /// The voices' sum, limited. Each renders in sub-blocks on its own count of samples, so the
    /// chunks here don't show in the output. One voice by itself is that voice, untouched.
    fn render_voices(&mut self, out: &mut [f32]) {
        self.voices[0].engine.render(out);
        if self.rendered == 1 {
            return;
        }
        // a voice the Voices parameter took away: only what is left of its release
        let tail = out.len().min(self.tail_left);
        let mut part = [0.0f32; SUB_BLOCK];
        for v in 1..self.rendered {
            let len = if v < self.count { out.len() } else { tail };
            for chunk in out[..len].chunks_mut(SUB_BLOCK) {
                let part = &mut part[..chunk.len()];
                self.voices[v].engine.render(part);
                for (o, x) in chunk.iter_mut().zip(part.iter()) {
                    *o += *x;
                }
            }
        }
        // to the sample the last of several voices stops at, whatever the host's blocks are
        let len = if self.count > 1 { out.len() } else { tail };
        self.limiter.process(&mut out[..len]);
        if self.rendered > self.count {
            self.tail_left -= tail;
            if self.tail_left == 0 {
                self.rendered = self.count;
                if self.count == 1 {
                    self.limiter.env = 0.0;
                }
            }
        }
    }

    pub fn render(&mut self, out: &mut [f32]) {
        for left in &mut self.slide_landing {
            *left = left.saturating_sub(out.len());
        }
        self.chord_left = self.chord_left.saturating_sub(out.len());
        let mut rest = &mut *out;
        while self.expr_moving && !rest.is_empty() {
            if self.expr_left == 0 {
                self.step_expression();
                self.expr_left = EXPR_STEP;
            }
            let n = self.expr_left.min(rest.len());
            let (now, later) = std::mem::take(&mut rest).split_at_mut(n);
            self.render_voices(now);
            self.expr_left -= n;
            rest = later;
        }
        self.render_voices(rest);
        if self.mode == Mode::Beatbox && self.mic_on {
            self.close_mic.process(out);
        }
    }

    /// End of a block of `samples`: tell the editor where things are.
    pub fn end_block(&mut self, samples: usize, shared: &Shared) {
        // of the voice with the newest note sounding
        let shown = &self.voices[self.shown];
        shared.cursor.store(self.singer().cursor() as i32, Ordering::Relaxed);
        shared.sung.store(shown.singer.sung().map_or(-1, |i| i as i32), Ordering::Relaxed);
        let note_of = |voice: &Voice| match self.mode {
            Mode::Sing => voice.singer.sounding_note().map_or(-1, |m| m as i32),
            Mode::Play => voice.play_keys.last().map_or(-1, |m| *m as i32),
            Mode::Speak | Mode::Beatbox => -1,
        };
        shared.note.store(note_of(shown), Ordering::Relaxed);
        shared.note_serial.store(self.presses, Ordering::Relaxed);
        shared.shown_voice.store(self.shown as i32, Ordering::Relaxed);
        for (v, voice) in self.voices.iter().enumerate() {
            let plays = v < self.count && !matches!(self.mode, Mode::Speak | Mode::Beatbox);
            let (note, sung) = if plays { (note_of(voice), voice.singer.sung().map_or(-1, |i| i as i32)) } else { (-1, -1) };
            shared.voice_note[v].store(note, Ordering::Relaxed);
            shared.voice_sung[v].store(sung, Ordering::Relaxed);
            shared.voice_serial[v].store(voice.presses, Ordering::Relaxed);
            let (f0, voicing) = if v < self.rendered { voice.engine.pitch() } else { (0.0, 0.0) };
            shared.voice_f0[v].store(f0.to_bits(), Ordering::Relaxed);
            shared.voice_voicing[v].store(voicing.to_bits(), Ordering::Relaxed);
        }
        shared.speech_cursor.store(self.speech_cursor as i32, Ordering::Relaxed);
        shared.spoken_line.store(self.spoken_line, Ordering::Relaxed);
        shared.spoken_generation.store(self.spoken_generation, Ordering::Relaxed);
        shared.spoken_t0.store(self.spoken_t0.to_bits(), Ordering::Relaxed);
        shared.spoken_serial.store(self.spoken_serial, Ordering::Release);
        shared.beat_step.store(self.engine().beat_step() as i32, Ordering::Relaxed);
        shared.beat_running.store(self.following || self.engine().beat().playing, Ordering::Relaxed);
        shared.beat_following.store(self.following, Ordering::Relaxed);
        shared.host_bpm.store(self.transport.tempo.unwrap_or(0.0).to_bits(), Ordering::Relaxed);
        let n = samples as i64;
        if shared.watch.fetch_sub(n, Ordering::Relaxed) <= 0 {
            shared.watch.store(0, Ordering::Relaxed);
            return;
        }
        self.tele_due -= n;
        if self.tele_due > 0 {
            return;
        }
        self.tele_due = self.tele_every;
        self.voices[self.shown].engine.telemetry(&mut self.tele);
        self.tele.write_flat(&mut self.block, &mut self.labels);
        if let Ok(mut frame) = shared.telemetry.try_lock() {
            std::mem::swap(&mut frame.block, &mut self.block);
            frame.labels = self.labels;
            frame.body = self.engine().info().body;
            frame.fresh = true;
        }
    }

    /// Back to the start of the lyrics, with nothing sounding. Not a full DSP reset: see
    /// docs/plugin.md.
    pub fn reset(&mut self) {
        self.stop_notes();
        self.stop_following();
        self.voices[0].engine.set_beat_playing(false);
        self.voices[0].engine.stop_speech();
        self.key_bend = 1.0;
        self.clear_expression();
        self.voices[0].singer.set_cursor(self.start_syllable);
        self.speech_cursor = self.speech_start;
        self.transport = Transport::default();
    }
}
