//! Alien bodies: the human airway stretched past human proportions, plus organs no human has.
//!
//! The shape scales the human body's own dimensions (`HumanBody`), so speech and singing keep
//! working: the phonetic targets are in template coordinates and the tract maps them to whatever
//! size it is. The organs are extra parts of the airway network, built only when present and
//! after every human part, so `AlienShape::HUMAN` builds exactly the human network.
//!
//! - **Throat sac**: an inflatable Helmholtz cavity on the pharynx just above the larynx, as in a
//!   frog's or a howler monkey's vocal sac. A low resonance and an antiresonance under the voice.
//! - **Second voicebox**: a second pair of folds in parallel with the first, at an interval from
//!   it (biphonation, as in the songbird's two-sided syrinx). It shares the lungs and the tract.
//! - **Crest**: a long tube from the top of the throat out through the top of the head, open at its
//!   end, like a lambeosaur's crest. Its resonances sit under the vowels as a drone.
//!
//! Beyond the airway, an alien may have a second whole mouth: a complete voice (lungs, voicebox,
//! tract and organs) of the same shape at an interval from the first, so one head sings a chord
//! (four notes with both mouths' second voiceboxes). Every mouth costs a whole voice's CPU, and
//! two is all that renders in real time in the browser (`MAX_MOUTHS`).

use crate::anatomy::human::HumanBody;
use crate::math::clamp;
use crate::voice::human_voice::SourcePatch;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AlienShape {
    /// overall size × the body's (0.6 … 2.5; at half size the airway is too short to stay stable)
    pub size: f64,
    /// throat length × the body's, against the mouth (0.6 … 1.8)
    pub throat: f64,
    /// tongue bulk × (0.85 … 1.2): beyond that the tongue no longer fits the palate
    pub tongue: f64,
    /// lip width × (0.5 … 2)
    pub lips: f64,
    /// voice pitch × (0.25 … 4): everything it speaks or sings, transposed
    pub pitch: f64,
    /// throat sac 0 (none) … 1 (huge)
    pub sac: f64,
    /// second voicebox level 0 (none) … 1 (as loud as the first)
    pub twin: f64,
    /// second voicebox's interval from the first (semitones, −24 … 24)
    pub interval: f64,
    /// crest 0 (none) … 1 (longest)
    pub crest: f64,
    /// whole mouths, each its own voice (1 … `MAX_MOUTHS`, whole numbers)
    pub mouths: f64,
    /// the second mouth's interval from the first (semitones, −24 … 24)
    pub mouth2: f64,
}

impl Default for AlienShape {
    fn default() -> Self {
        AlienShape::HUMAN
    }
}

impl AlienShape {
    /// No change: the human body as it is.
    pub const HUMAN: AlienShape = AlienShape { size: 1.0, throat: 1.0, tongue: 1.0, lips: 1.0, pitch: 1.0, sac: 0.0, twin: 0.0, interval: 7.0, crest: 0.0, mouths: 1.0, mouth2: 12.0 };

    /// The most mouths an alien can have. Each is a whole voice to render: in wasm at 48 kHz one
    /// takes about a third of real time (Chromium: 34 % for one, 68 % for two, 102 % for three).
    pub const MAX_MOUTHS: usize = 2;

    /// Clamped to the ranges above (non-finite values fall back to human).
    pub fn new(s: AlienShape) -> AlienShape {
        let c = |x: f64, lo: f64, hi: f64, d: f64| if x.is_finite() { clamp(x, lo, hi) } else { d };
        let h = AlienShape::HUMAN;
        AlienShape {
            size: c(s.size, 0.6, 2.5, h.size),
            throat: c(s.throat, 0.6, 1.8, h.throat),
            tongue: c(s.tongue, 0.85, 1.2, h.tongue),
            lips: c(s.lips, 0.5, 2.0, h.lips),
            pitch: c(s.pitch, 0.25, 4.0, h.pitch),
            sac: c(s.sac, 0.0, 1.0, h.sac),
            twin: c(s.twin, 0.0, 1.0, h.twin),
            interval: c(s.interval, -24.0, 24.0, h.interval),
            crest: c(s.crest, 0.0, 1.0, h.crest),
            mouths: crate::math::js_round(c(s.mouths, 1.0, AlienShape::MAX_MOUTHS as f64, h.mouths)),
            mouth2: c(s.mouth2, -24.0, 24.0, h.mouth2),
        }
    }

    pub fn is_human(&self) -> bool {
        *self == AlienShape::HUMAN
    }

    /// The body with this shape applied (exactly `body` when human).
    pub fn apply(&self, body: HumanBody) -> HumanBody {
        if self.is_human() {
            return body;
        }
        HumanBody { scale: body.scale * self.size, pharynx: body.pharynx * self.throat, tongue_radius: body.tongue_radius * self.tongue, lip_width: body.lip_width * self.lips, ..body }
    }

    /// How many whole mouths it sings with (1 … `MAX_MOUTHS`).
    pub fn mouth_count(&self) -> usize {
        self.mouths as usize
    }

    /// The extra mouths' intervals from the first (semitones), as many as it has beyond one.
    pub fn mouth_intervals(&self) -> impl Iterator<Item = f64> {
        [self.mouth2].into_iter().take(self.mouth_count().saturating_sub(1))
    }

    /// The shape of the extra mouth at `interval`: this shape with one mouth, transposed (pitch
    /// clamped to its range, so a mouth two octaves above a high voice stops at 4×).
    pub fn mouth_shape(&self, interval: f64) -> AlienShape {
        AlienShape::new(AlienShape { mouths: 1.0, pitch: self.pitch * crate::math::pow(2.0, interval / 12.0), ..*self })
    }

    /// The airway's extra organs.
    pub fn organs(&self) -> Organs {
        Organs { sac: self.sac, twin: self.twin > 0.0, crest: self.crest }
    }
}

/// What the airway builds beyond the human parts (sizes 0..1; 0 = absent).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Organs {
    pub sac: f64,
    pub twin: bool,
    pub crest: f64,
}

impl Organs {
    pub const NONE: Organs = Organs { sac: 0.0, twin: false, crest: 0.0 };
}

/// A named alien: its shape, how its voice sits (a source patch over the performer's), how fast it
/// talks, and a line it says when you meet it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlienPreset {
    pub id: &'static str,
    pub name: &'static str,
    pub blurb: &'static str,
    pub shape: AlienShape,
    pub source: SourcePatch,
    /// speaking-rate multiplier for its line (<1 slower)
    pub rate: f64,
    /// what it says first (English, through the usual G2P)
    pub line: &'static str,
}

const fn shape(size: f64, throat: f64, tongue: f64, lips: f64, pitch: f64, sac: f64, twin: f64, interval: f64, crest: f64) -> AlienShape {
    AlienShape { size, throat, tongue, lips, pitch, sac, twin, interval, crest, ..AlienShape::HUMAN }
}

/// `shape` with a second whole mouth at `interval` from the first.
const fn two_mouths(s: AlienShape, interval: f64) -> AlienShape {
    AlienShape { mouths: 2.0, mouth2: interval, ..s }
}

const fn src(pressure: f64, breathiness: f64, tension: f64, fry: f64, roughness: f64, vibrato_depth: f64) -> SourcePatch {
    SourcePatch { f0: None, glide: None, pressure: Some(pressure), breathiness: Some(breathiness), tension: Some(tension), fry: Some(fry), roughness: Some(roughness), vibrato_depth: Some(vibrato_depth), vibrato_rate: None, drift: None, voiced: Some(true), brightness: None }
}

/// The aliens, the first being the one Alien mode starts with.
pub const ALIEN_PRESETS: &[AlienPreset] = &[
    AlienPreset {
        id: "homesick",
        name: "Homesick",
        blurb: "Small, with a long folding neck and a low, breathy, creaking voice. Far from home.",
        shape: shape(0.72, 1.7, 1.0, 0.7, 0.62, 0.2, 0.0, 7.0, 0.0),
        source: src(13.0, 0.42, 0.35, 0.35, 0.3, 0.0),
        rate: 0.45,
        line: "Home. Go home.",
    },
    AlienPreset {
        id: "choir",
        name: "Two-throat",
        blurb: "Two voiceboxes a fifth apart: every note it sings is a chord.",
        shape: shape(1.1, 1.1, 1.0, 1.1, 1.0, 0.0, 0.8, 7.0, 0.0),
        source: src(8.0, 0.15, 0.55, 0.0, 0.08, 40.0),
        rate: 0.9,
        line: "We sing together.",
    },
    AlienPreset {
        id: "bellows",
        name: "Bellows",
        blurb: "Huge, with a throat sac like a frog's: a deep boom under every word.",
        shape: shape(2.2, 1.2, 1.05, 1.4, 0.45, 0.9, 0.0, 7.0, 0.0),
        source: src(9.0, 0.1, 0.6, 0.2, 0.2, 0.0),
        rate: 0.6,
        line: "I am very large.",
    },
    AlienPreset {
        id: "hornhead",
        name: "Hornhead",
        blurb: "A metre of hollow crest from nose to skull: the voice drones through it like a horn.",
        shape: shape(1.3, 1.0, 1.0, 1.0, 0.8, 0.0, 0.0, 7.0, 0.8),
        source: src(9.0, 0.2, 0.5, 0.0, 0.1, 20.0),
        rate: 0.75,
        line: "Listen to my crest.",
    },
    AlienPreset {
        id: "pip",
        name: "Pip",
        blurb: "Tiny, wide-mouthed and squeaky, with a second voicebox an octave up.",
        shape: shape(0.6, 0.75, 0.9, 1.7, 2.2, 0.0, 0.45, 12.0, 0.0),
        source: src(7.0, 0.25, 0.6, 0.0, 0.15, 30.0),
        rate: 1.3,
        line: "Hello! Hello, friend!",
    },
    AlienPreset {
        id: "quartet",
        name: "Choir",
        blurb: "Two whole mouths an octave apart, each with a second voicebox a fifth up: four notes from one head.",
        shape: two_mouths(shape(1.15, 1.05, 1.0, 1.2, 1.0, 0.0, 0.7, 7.0, 0.0), 12.0),
        source: src(8.0, 0.18, 0.5, 0.0, 0.1, 35.0),
        rate: 0.8,
        line: "We are many. We sing as one.",
    },
];

pub fn alien_preset(id: &str) -> Option<&'static AlienPreset> {
    ALIEN_PRESETS.iter().find(|p| p.id == id)
}
