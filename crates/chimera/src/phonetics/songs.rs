//! Demo melodies for Sing mode: [MIDI note, beats] per syllable, in the adult female register.

use serde::Serialize;

/// octave shift per body
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SongOctave {
    pub male: f64,
    pub female: f64,
    pub child: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoSong {
    pub name: &'static str,
    pub bpm: f64,
    pub lyrics: &'static str,
    pub notes: &'static [[f64; 2]],
    /// octave shift per body
    pub octave: SongOctave,
}

#[rustfmt::skip]
pub static DEMO_SONGS: &[DemoSong] = &[
    DemoSong {
        // an original tune for this instrument
        name: "Made of air",
        bpm: 84.0,
        lyrics: "Breathe in, breathe out, I am made of air. Tongue and lips and throat, singing anywhere.",
        notes: &[
            [64.0, 1.0], [60.0, 1.0], [62.0, 1.0], [57.0, 1.0],
            [57.0, 0.5], [60.0, 0.5], [64.0, 1.0], [62.0, 0.5], [60.0, 1.5],
            [65.0, 1.0], [64.0, 0.5], [62.0, 0.5], [60.0, 0.5], [62.0, 1.5],
            [64.0, 0.5], [62.0, 0.5], [60.0, 0.5], [59.0, 0.5], [57.0, 2.0],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
    },
    DemoSong {
        // traditional (public domain)
        name: "Twinkle, twinkle",
        bpm: 100.0,
        lyrics: "Twinkle twinkle little star, how I wonder what you are.",
        notes: &[
            [60.0, 1.0], [60.0, 1.0], [67.0, 1.0], [67.0, 1.0], [69.0, 1.0], [69.0, 1.0], [67.0, 2.0],
            [65.0, 1.0], [65.0, 1.0], [64.0, 1.0], [64.0, 1.0], [62.0, 1.0], [62.0, 1.0], [60.0, 2.0],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
    },
    DemoSong {
        // traditional Turkish folk song (Aşık Veysel - Uzun İnce Bir Yoldayım, Uşşak mode with melismas)
        name: "Uzun İnce Bir Yoldayım",
        bpm: 92.0,
        lyrics: "Uzun ~ ~ ince ~ bir ~ yoldayım ~ ~ ~ , gidiyorum ~ gündüz ~ gece ~ ~ ~ . Bilmiyorum ~ ne ~ haldeyim ~ ~ ~ , gidiyorum ~ gündüz ~ gece ~ ~ , gündüz ~ gece ~ ~ ~ .",
        notes: &[
            // U-zun(~ ~) in-ce(~) bir(~) yol-da-yım(~ ~ ~) -> 15 notes
            [62.0, 0.5], [69.0, 0.75], [71.0, 0.25], [69.0, 0.5],
            [69.0, 0.5], [69.0, 0.25], [67.0, 0.25],
            [67.0, 0.75], [65.0, 0.25],
            [65.0, 0.75], [67.0, 0.5], [69.0, 0.75], [71.0, 0.25], [69.0, 0.5], [67.0, 0.5],
            // gi-di-yo-rum(~) gün-düz(~) ge-ce(~ ~ ~) -> 13 notes
            [67.0, 0.5], [65.0, 0.5], [67.0, 0.75], [65.0, 0.75], [64.0, 0.5],
            [64.0, 0.75], [62.0, 0.5], [64.0, 0.25],
            [64.0, 0.5], [65.0, 1.0], [67.0, 0.25], [65.0, 0.5], [64.0, 0.75],
            // Bil-mi-yo-rum(~) ne(~) hal-de-yim(~ ~ ~) -> 13 notes
            [65.0, 0.5], [67.0, 0.5], [65.0, 0.75], [64.0, 0.75], [62.0, 0.5],
            [62.0, 0.75], [60.0, 0.25],
            [60.0, 0.75], [62.0, 0.5], [64.0, 0.75], [65.0, 0.25], [64.0, 0.5], [62.0, 0.5],
            // gi-di-yo-rum(~) gün-düz(~) ge-ce(~ ~) -> 12 notes
            [62.0, 0.5], [60.0, 0.5], [62.0, 0.75], [60.0, 0.75], [59.0, 0.5],
            [59.0, 0.75], [57.0, 0.5], [59.0, 0.25],
            [59.0, 0.5], [60.0, 0.75], [62.0, 0.25], [60.0, 0.75],
            // gün-düz(~) ge-ce(~ ~ ~) -> 8 notes
            [59.0, 0.75], [57.0, 0.5], [59.0, 0.25],
            [57.0, 0.5], [57.0, 1.0], [59.0, 0.25], [57.0, 0.5], [57.0, 1.75],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
    },
];
