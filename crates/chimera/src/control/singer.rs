//! Live singing from notes: the lyrics are syllables with a cursor, and each note sings the next
//! syllable at its pitch. A key pressed while the previous one is down sings legato; a "-" in the
//! lyrics holds the vowel onto a new pitch (melisma); releasing the sounding key while another is
//! down glides back to it; velocity sets the breath; the release sings the coda.
//!
//! Real time: once built, nothing here allocates or frees, so a plugin can call it from its audio
//! thread. Only [`syllables_from`] and building a [`Singer`] allocate; do those elsewhere and hand
//! the result to [`Singer::set_syllables`]. The browser has its own copy of this logic in
//! `src/ui/singing.ts`, which also keeps the piano roll.
//!
//! One `Singer` sings one note at a time. A choir has one for each voice, and one of them holds the
//! lyrics: it hands out the syllables ([`Singer::next_lyric`]) and each voice sings its own
//! ([`Singer::sing`], [`Singer::lift`]).

use crate::anatomy::human::BodyMix;
use crate::control::score::{ControlKey, Gesture};
use crate::engine::Engine;
use crate::phonetics::Syllable;
use crate::phonetics::planner::EXHALE_LABEL;
use crate::phonetics::sing::{LiveNoteOn, LiveSyllable, MelismaOptions, plan_melisma_into, plan_note_off_into, plan_note_on_into};

/// Gestures planned on arrival start this far ahead of the voice clock (as the browser does).
const LEAD: f64 = 0.006;
/// Keys the singer tracks: one per MIDI note.
const MAX_KEYS: usize = 128;
/// Endings remembered, so a following note can silence their breath-out.
const MAX_TAILS: usize = 8;
/// Room for more gestures than a live note plans (18 segments at most).
const SCRATCH: usize = 256;

#[derive(Clone, Copy)]
struct Sounding {
    midi: u8,
    /// the syllable whose vowel is sounding (a melisma keeps its parent's, for the coda)
    syl: LiveSyllable,
    tag: f64,
}

/// A syllable of the lyrics, as a note sings it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lyric {
    /// where it is in the lyrics
    pub index: usize,
    pub syllable: LiveSyllable,
    /// the syllable a melisma continues (the syllable itself otherwise)
    pub parent: LiveSyllable,
}

/// Lyrics as a [`Singer`] holds them. Never empty: no words sing "ah". Allocates.
pub fn syllables_from(syllables: &[Syllable]) -> Vec<LiveSyllable> {
    if syllables.is_empty() {
        return vec![LiveSyllable::ah()];
    }
    syllables.iter().map(LiveSyllable::from).collect()
}

/// Breath level for a MIDI velocity (1 at the on-screen keys' default of 96).
fn breath_for(velocity: f64) -> f64 {
    0.62 + 0.4 * (velocity / 96.0).clamp(0.0, 1.0) + 0.12 * ((velocity - 96.0) / 31.0).max(0.0)
}

pub struct Singer {
    syllables: Vec<LiveSyllable>,
    cursor: usize,
    /// the last syllable the cursor passed that isn't a melisma, while it has moved in order since
    /// (the parent of a melisma at the cursor, without looking for it)
    plain: Option<usize>,
    sung: Option<usize>,
    /// keys down, oldest first
    keys: Vec<u8>,
    sounding: Option<Sounding>,
    /// gesture tags: a note uses tag (articulation, breath) and tag + 1 (pitch, vibrato)
    next_tag: f64,
    /// endings (glide, coda) that may still be playing: tag, and tag + 1 for the breath-out
    tails: Vec<f64>,
    /// endings whose breath-out has not been dropped by a following note
    exhales_live: bool,
    scratch: Vec<Gesture>,
    pub body: BodyMix,
    /// vibrato depth on held vowels (cents)
    pub vibrato: f64,
}

impl Default for Singer {
    fn default() -> Self {
        Self::new()
    }
}

impl Singer {
    pub fn new() -> Singer {
        Singer {
            syllables: vec![LiveSyllable::ah()],
            cursor: 0,
            plain: None,
            sung: None,
            keys: Vec::with_capacity(MAX_KEYS),
            sounding: None,
            next_tag: 2.0,
            tails: Vec::with_capacity(MAX_TAILS + 1),
            exhales_live: false,
            scratch: Vec::with_capacity(SCRATCH),
            body: BodyMix::MALE,
            vibrato: 35.0,
        }
    }

    /// The syllable the next note sings.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// The syllable being sung now.
    pub fn sung(&self) -> Option<usize> {
        self.sung
    }

    /// The MIDI note sounding now.
    pub fn sounding_note(&self) -> Option<u8> {
        self.sounding.as_ref().map(|s| s.midi)
    }

    /// Whether any key is down.
    pub fn any_key_down(&self) -> bool {
        !self.keys.is_empty()
    }

    /// New lyrics from [`syllables_from`]: the cursor goes back to the first syllable, a note
    /// still sounding keeps its own coda. Returns the old lyrics, for the caller to free
    /// somewhere it may. An empty `Vec` is ignored and handed back.
    pub fn set_syllables(&mut self, syllables: Vec<LiveSyllable>) -> Vec<LiveSyllable> {
        if syllables.is_empty() {
            return syllables;
        }
        self.cursor = 0;
        self.plain = None;
        self.sung = None;
        core::mem::replace(&mut self.syllables, syllables)
    }

    /// Point the cursor at a syllable (clamped to the lyrics).
    pub fn set_cursor(&mut self, i: usize) {
        self.cursor = i.min(self.syllables.len() - 1);
        self.plain = None;
    }

    /// The lyrics it sang from are gone (another singer holds them): its syllable's index means
    /// nothing now. The note itself carries on.
    pub fn forget_sung(&mut self) {
        self.sung = None;
    }

    fn take_tag(&mut self) -> f64 {
        let t = self.next_tag;
        self.next_tag += 2.0;
        t
    }

    fn start_note(&mut self, e: &mut Engine, syl: LiveSyllable, midi: u8, velocity: f64) {
        let tag = self.take_tag();
        // earlier codas may finish, but their breath-out must not open the folds on this onset
        if self.exhales_live {
            for t in &self.tails {
                e.release(t + 1.0);
            }
        }
        self.exhales_live = false;
        let from = self.sounding.take();
        if let Some(f) = &from {
            e.release(f.tag);
            e.release(f.tag + 1.0);
        }
        self.scratch.clear();
        let o = LiveNoteOn { body: self.body, vibrato: self.vibrato, tag, legato_from: from.map(|f| (f.syl, f.midi as f64)) };
        plan_note_on_into(&syl, midi as f64, &o, &mut self.scratch);
        let breath = breath_for(velocity);
        for g in self.scratch.iter_mut() {
            if let Some(b) = g.targets.get(ControlKey::BREATH) {
                g.targets.set(ControlKey::BREATH, b * breath);
            }
        }
        e.schedule(&self.scratch, LEAD, false);
        self.sounding = Some(Sounding { midi, syl, tag });
    }

    /// The held vowel moves to a new pitch (melisma, or back to a key still down).
    fn glide(&mut self, e: &mut Engine, midi: u8) {
        let Some(s) = self.sounding.as_mut() else { return };
        e.release(s.tag + 1.0);
        s.midi = midi;
        let tag = s.tag;
        self.scratch.clear();
        plan_melisma_into(midi as f64, &MelismaOptions { vibrato: self.vibrato, tag }, &mut self.scratch);
        e.schedule(&self.scratch, LEAD, false);
    }

    /// The syllable at the cursor, for the next note; the cursor moves on.
    pub fn next_lyric(&mut self) -> Lyric {
        let index = self.cursor % self.syllables.len();
        let syllable = self.syllables[index];
        self.cursor = (index + 1) % self.syllables.len();
        let parent = if syllable.melisma {
            // the syllable this melisma continues: remembered while the cursor moves in order, so a
            // long run of melismas is looked through once, not once a note
            let at = self.plain.filter(|at| *at < index).or_else(|| self.syllables[..index].iter().rposition(|s| !s.melisma));
            self.plain = at;
            at.map_or_else(LiveSyllable::ah, |at| self.syllables[at])
        } else {
            self.plain = Some(index);
            syllable
        };
        Lyric { index, syllable, parent }
    }

    /// Whether the next note continues the vowel now sung (a melisma).
    pub fn next_is_melisma(&self) -> bool {
        self.syllables.get(self.cursor).is_some_and(|n| n.melisma)
    }

    /// A key went down. `velocity` in MIDI units (0–127).
    pub fn note_on(&mut self, e: &mut Engine, midi: u8, velocity: f64) {
        let lyric = self.next_lyric();
        self.sing(e, &lyric, midi, velocity);
    }

    /// A key went down, and it sings this syllable (from these lyrics or another singer's).
    pub fn sing(&mut self, e: &mut Engine, lyric: &Lyric, midi: u8, velocity: f64) {
        self.keys.retain(|&k| k != midi);
        self.keys.push(midi);
        let syl = lyric.syllable;
        if syl.melisma && self.sounding.is_some() {
            self.glide(e, midi);
        } else if syl.melisma {
            // a melisma with nothing held: sing its vowel again, without the onset
            let parent = LiveSyllable { onset: Default::default(), ..lyric.parent };
            self.start_note(e, parent, midi, velocity);
        } else {
            self.start_note(e, syl, midi, velocity);
        }
        self.sung = Some(lyric.index);
    }

    /// A key went up.
    pub fn note_off(&mut self, e: &mut Engine, midi: u8) {
        self.lift(e, midi, self.next_is_melisma());
    }

    /// A key went up. `melisma_next`: the lyrics' next syllable is a melisma (see
    /// [`Singer::next_is_melisma`], asked of the singer that holds the lyrics).
    pub fn lift(&mut self, e: &mut Engine, midi: u8, melisma_next: bool) {
        self.keys.retain(|&k| k != midi);
        let Some(s) = self.sounding else { return };
        if s.midi != midi {
            return;
        }
        if let Some(&k) = self.keys.last() {
            self.glide(e, k);
            return;
        }
        self.sounding = None;
        e.release(s.tag);
        e.release(s.tag + 1.0);
        self.sung = None;
        // a melisma comes next: this vowel carries on there, and the coda waits for the chain's end
        if melisma_next {
            return;
        }
        self.scratch.clear();
        plan_note_off_into(&s.syl, midi as f64, self.body, &mut self.scratch);
        if self.scratch.is_empty() {
            return;
        }
        let tag = self.take_tag();
        for g in self.scratch.iter_mut() {
            g.tag = Some(if g.label.is_some_and(|l| l.as_str() == EXHALE_LABEL) { tag + 1.0 } else { tag });
        }
        e.schedule(&self.scratch, LEAD, false);
        self.tails.push(tag);
        if self.tails.len() > MAX_TAILS {
            self.tails.remove(0); // long finished by now
        }
        self.exhales_live = true;
    }

    /// Stop everything sung (transport stop, all-notes-off, reset). The cursor stays.
    pub fn stop_all(&mut self, e: &mut Engine) {
        if let Some(s) = self.sounding.take() {
            e.release(s.tag);
            e.release(s.tag + 1.0);
        }
        for t in &self.tails {
            e.release(*t);
            e.release(t + 1.0);
        }
        self.tails.clear();
        self.exhales_live = false;
        self.keys.clear();
        self.sung = None;
    }
}
