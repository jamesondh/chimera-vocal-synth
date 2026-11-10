//! Demo melodies for Sing mode: [MIDI note, beats] per syllable, in the adult female register.
//! Notes may sit between the keys (a makam's koma pitches).

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
    /// the language of the lyrics ("en" or "tr"), which says how they become syllables
    #[serde(skip_serializing_if = "is_english")]
    pub lang: &'static str,
}

fn is_english(lang: &&str) -> bool {
    *lang == "en"
}

/// Uşşak/Hüseyni pitches (MIDI) of "Gesi Bağları": karar La; Si is segah, 8 komas (1.81
/// semitones) above La in the Arel–Ezgi–Uzdilek system; the F♯ of the opening "Of" is eviç,
/// a koma under F♯.
mod gesi {
    pub const LA: f64 = 69.0;
    pub const SEGAH: f64 = 70.8;
    pub const DO: f64 = 72.0;
    pub const RE: f64 = 74.0;
    pub const MI: f64 = 76.0;
    pub const FA: f64 = 77.0;
    pub const EVIC: f64 = 77.8;
    pub const SOL: f64 = 79.0;
}

/// Pitches (MIDI) of "Atma Beni Yabana": karar Sol; the score's Si is B♭², two komas (2 × 22.6
/// cents) under Si.
mod atma {
    pub const SOL: f64 = 67.0;
    pub const LA: f64 = 69.0;
    pub const SI: f64 = 70.55;
    pub const DO: f64 = 72.0;
    pub const RE: f64 = 74.0;
    pub const MI: f64 = 76.0;
    pub const FA: f64 = 77.0;
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
        lang: "en",
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
        lang: "en",
    },
    DemoSong {
        // Kayseri türküsü, anonymous (public domain). The first verse of TRT THM repertoire no. 631
        // (from Ahmet Gazi Ayhan, notated by Muzaffer Sarısözen), its 32nd-note ornaments cut to
        // two to four notes a syllable and the saz interludes left out; the rests are breaths at
        // the ends of the notes before them. A hyphen ending a piece of a word says the word goes
        // on; each lone "-" after it is a held note ("dola- - nıyorum": la, held, nı).
        name: "Gesi Bağları",
        bpm: 72.0,
        lyrics: "Of - - - Gesi bağlarında dola- - nıyo- - - rum - - - , \
                 Yi- - tir- - - dim - - - yâ- - - ri- - - - mi, aman ara- - nı- - - - - - - yorum. \
                 Yi- - tir- - - dim - - - yâ- - - ri- - - - mi, aman ara- - nı- - - - - - - yorum.",
        notes: &[
            // (2/4) Of… | (6/4) …f Ge-si bağ-la-rın-da do-la-nı-yo-rum
            [gesi::SOL, 1.0], [gesi::EVIC, 0.75], [gesi::MI, 0.25],
            [gesi::MI, 0.5], [gesi::MI, 0.25], [gesi::MI, 0.25], [gesi::MI, 0.5], [gesi::MI, 0.5], [gesi::MI, 0.5], [gesi::MI, 0.5],
            [gesi::MI, 0.5], [gesi::MI, 0.25], [gesi::RE, 0.25], [gesi::RE, 0.25], [gesi::MI, 0.25], [gesi::FA, 0.25], [gesi::RE, 0.25],
            [gesi::MI, 0.25], [gesi::RE, 0.25], [gesi::DO, 0.25], [gesi::RE, 0.25],
            // (6/4) yi-tir-dim yâ-ri-mi (and the eighth rest)
            [gesi::DO, 0.25], [gesi::RE, 0.25], [gesi::DO, 0.25], [gesi::RE, 1.0], [gesi::DO, 0.25],
            [gesi::DO, 0.25], [gesi::RE, 0.25], [gesi::MI, 0.25], [gesi::RE, 0.25],
            [gesi::MI, 0.25], [gesi::DO, 0.25], [gesi::RE, 0.5],
            [gesi::SEGAH, 0.25], [gesi::DO, 0.25], [gesi::RE, 0.25], [gesi::SEGAH, 0.25], [gesi::LA, 1.5],
            // (6/4) a-man a-ra-nı-yo-rum
            [gesi::SEGAH, 0.25], [gesi::DO, 0.25], [gesi::DO, 0.25], [gesi::RE, 0.25], [gesi::SEGAH, 0.5],
            [gesi::DO, 0.25], [gesi::RE, 0.25], [gesi::DO, 0.25], [gesi::SEGAH, 0.25], [gesi::LA, 0.25], [gesi::SEGAH, 0.125], [gesi::LA, 0.125],
            [gesi::LA, 0.5], [gesi::LA, 2.0],
            // the second half again
            [gesi::DO, 0.25], [gesi::RE, 0.25], [gesi::DO, 0.25], [gesi::RE, 1.0], [gesi::DO, 0.25],
            [gesi::DO, 0.25], [gesi::RE, 0.25], [gesi::MI, 0.25], [gesi::RE, 0.25],
            [gesi::MI, 0.25], [gesi::DO, 0.25], [gesi::RE, 0.5],
            [gesi::SEGAH, 0.25], [gesi::DO, 0.25], [gesi::RE, 0.25], [gesi::SEGAH, 0.25], [gesi::LA, 1.5],
            [gesi::SEGAH, 0.25], [gesi::DO, 0.25], [gesi::DO, 0.25], [gesi::RE, 0.25], [gesi::SEGAH, 0.5],
            [gesi::DO, 0.25], [gesi::RE, 0.25], [gesi::DO, 0.25], [gesi::SEGAH, 0.25], [gesi::LA, 0.25], [gesi::SEGAH, 0.125], [gesi::LA, 0.125],
            [gesi::LA, 0.5], [gesi::LA, 2.0],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
        lang: "tr",
    },
    DemoSong {
        // Trabzon (Maçka, Hamsiköy) türküsü, anonymous (public domain): TRT THM repertoire no. 2907,
        // from Salih Zeki Bahçekapılı, compiled by Volkan Konak, notated by Yücel Paşmakçı. Its
        // sung verse, 10/8: two lines, each twice (the second time "Atma" rises out of Do and
        // the line ends up on Re, into "Al beni"). Ornaments cut to sixteenths, the saz left out.
        name: "Atma Beni Yabana",
        bpm: 92.0,
        lyrics: "Atma - - - be- - - ni yabana - - ben da - bu - - - - - dereliyim - - , \
                 At- - - ma - - - - be- - - ni yabana - - ben da - bu - - - - dereliyim - - - - , \
                 Al - - - - be- - - - ni yar ya- - - nına - hiç sor- - ma - - - - nereliyim - - , \
                 Al - - - - be- - - - - ni - yar - cidelim - - - hiç sor- - ma - - - - nereliyim - .",
        notes: &[
            // At-ma be-ni ya-ba-na
            [atma::SOL, 1.0], [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.25], [atma::RE, 0.25],
            [atma::DO, 0.25], [atma::SI, 0.25], [atma::DO, 0.5], [atma::DO, 0.75],
            [atma::RE, 0.25], [atma::MI, 0.25], [atma::MI, 0.25], [atma::FA, 0.25], [atma::RE, 0.25],
            // ben da bu de-re-li-yim
            [atma::RE, 0.5], [atma::RE, 0.25], [atma::SI, 0.25],
            [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.25], [atma::SI, 0.75],
            [atma::LA, 0.5], [atma::LA, 0.5], [atma::SI, 0.25], [atma::SOL, 0.25], [atma::LA, 0.25], [atma::SOL, 0.25],
            // At-ma be-ni ya-ba-na, again
            [atma::DO, 0.25], [atma::RE, 0.5], [atma::DO, 0.25],
            [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.25],
            [atma::DO, 0.25], [atma::SI, 0.25], [atma::DO, 0.25], [atma::DO, 0.75],
            [atma::RE, 0.25], [atma::MI, 0.25], [atma::MI, 0.25], [atma::FA, 0.25], [atma::RE, 0.25],
            // ben da bu de-re-li-yim, up to Re
            [atma::RE, 0.5], [atma::RE, 0.25], [atma::DO, 0.25],
            [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.5],
            [atma::SI, 0.25], [atma::SI, 0.25], [atma::LA, 0.5],
            [atma::SI, 0.25], [atma::LA, 0.25], [atma::RE, 0.25], [atma::DO, 0.25], [atma::RE, 0.5],
            // Al be-ni yar ya-nı-na
            [atma::RE, 0.25], [atma::MI, 0.25], [atma::RE, 0.25], [atma::MI, 0.25], [atma::RE, 0.5],
            [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.25], [atma::RE, 0.25], [atma::SI, 0.5],
            [atma::SI, 0.5], [atma::SI, 0.25], [atma::DO, 0.25], [atma::LA, 0.25], [atma::SOL, 0.25],
            [atma::DO, 0.25], [atma::RE, 0.25],
            // hiç sor-ma ne-re-li-yim
            [atma::RE, 0.5], [atma::RE, 0.25], [atma::SI, 0.25],
            [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.25],
            [atma::DO, 0.5], [atma::SI, 0.5], [atma::SI, 0.75], [atma::SOL, 0.25], [atma::LA, 0.25], [atma::SOL, 0.5],
            // Al be-ni yar ci-de-lim
            [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.25],
            [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.25],
            [atma::DO, 0.25], [atma::SI, 0.25], [atma::SI, 0.25], [atma::LA, 0.25], [atma::SI, 0.25], [atma::SI, 0.25],
            [atma::RE, 0.25], [atma::MI, 0.25], [atma::RE, 0.25], [atma::MI, 0.25],
            // hiç sor-ma ne-re-li-yim, down to the karar
            [atma::RE, 0.5], [atma::RE, 0.25], [atma::SI, 0.25],
            [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.25], [atma::RE, 0.25], [atma::DO, 0.25],
            [atma::DO, 0.5], [atma::SI, 0.5], [atma::SI, 0.5], [atma::LA, 0.5], [atma::SOL, 0.75],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
        lang: "tr",
    },
];
