//! Arabic text → phoneme segments, read as Arabic is read aloud: from vowelled text (tashkīl),
//! letter by letter, with the rules of correct reading that the script leaves implicit.
//!
//! - Letters, short vowels (fatḥa, kasra, ḍamma), sukūn, shadda (a doubled consonant: one long
//!   segment, not two), tanwīn, long vowels from the madd letters (ا و ي ى and the dagger alif ٰ),
//!   the diphthongs aw/aj, hamza on every seat, maddah (آ = ʔaː).
//! - Hamzat al-waṣl (ٱ, and the bare alif of the article and of words like اسم): voiced as ʔa/ʔi/ʔu
//!   only at the start of an utterance, otherwise elided; a long vowel before it shortens (two
//!   sākin sounds never meet).
//! - The article: its lām is said before the moon letters and assimilated into the sun letters.
//! - Waqf (the pause at a phrase end): the final short vowel drops, tanwīn fatḥ becomes ā, tāʾ
//!   marbūṭa becomes h.
//! - Tafkhīm: the seven heavy letters (خ ص ض غ ط ق ظ) make their vowel heavy (the tongue pulled
//!   back: "aˤ"), the rāʾ is heavy or light by its vowel and the vowel before it, and the lām of
//!   the name Allāh is heavy after a or u.
//! - Hidden long vowels of the standard spelling (هذا, ذلك, لكن, الله, الرحمن, …).
//! - Stress (for the speaking voice's pitch accents): the last superheavy syllable, else a heavy
//!   penult, else the antepenult.
//!
//! Durations are counted in ḥarakāt (a short vowel is one count, a long vowel two, a doubled
//! nasal two), and a count is a tempo. That's the unit tajweed measures its lengthenings in, so
//! the recitation rules (madd of 4 and 6, ghunna, qalqala, idghām, ikhfāʾ, iqlāb) can be added as
//! changes to counts and segments. Each segment keeps the span of text it was read from and the
//! rules that shaped it; the tajweed-relevant contexts are tagged already (see [`rules`]).
//!
//! Text without vowel marks is read with guesses (a written madd letter gives the vowel before
//! it, otherwise fatḥa, and a word-final letter is sākin); [`ArabicPhrase::guessed`] counts them.

use super::inventory::phoneme;
use super::{Intonation, Segment};
use crate::anatomy::human::BodyMix;
use serde::{Deserialize, Serialize};

/// The rule names a segment can carry ([`ArabicPhrase::rules`]).
pub mod rules {
    /// a long vowel from a madd letter (natural madd: two counts)
    pub const MADD: &str = "madd";
    /// the madd letter is followed by a hamza in the same word (madd muttaṣil)
    pub const MADD_MUTTASIL: &str = "madd-muttasil";
    /// the madd letter ends a word and the next starts with a hamza (madd munfaṣil)
    pub const MADD_MUNFASIL: &str = "madd-munfasil";
    /// the madd letter is followed by a sākin or doubled consonant (madd lāzim)
    pub const MADD_LAZIM: &str = "madd-lazim";
    /// the madd letter stands before the last letter of a word read in pause (madd ʿāriḍ)
    pub const MADD_ARID: &str = "madd-arid";
    /// the text marks this madd as lengthened (ٓ or ۤ on a madd letter)
    pub const MADD_SIGN: &str = "madd-sign";
    /// a consonant doubled by shadda
    pub const SHADDA: &str = "shadda";
    /// a doubled nūn or mīm, held with nasal resonance (ghunna)
    pub const GHUNNA: &str = "ghunna";
    /// the n of tanwīn
    pub const TANWIN: &str = "tanwin";
    /// a nūn sākin or tanwīn, before the letter that decides its tajweed (iẓhār, idghām, iqlāb, ikhfāʾ)
    pub const NUN_SAKIN: &str = "nun-sakin";
    /// a mīm sākin (iẓhār shafawī, idghām, ikhfāʾ shafawī)
    pub const MIM_SAKIN: &str = "mim-sakin";
    /// the text marks iqlāb (ۢ or ۭ: the n becomes m before b)
    pub const IQLAB: &str = "iqlab";
    /// one of ق ط ب ج د without a vowel (qalqala)
    pub const QALQALA: &str = "qalqala";
    /// a heavy (mufakhkham) segment
    pub const TAFKHIM: &str = "tafkhim";
    /// a light rāʾ or the light lām of Allāh
    pub const TARQIQ: &str = "tarqiq";
    /// the lām of the name Allāh
    pub const ALLAH: &str = "allah";
    /// the lām of the article, said before a moon letter
    pub const QAMARIYYA: &str = "qamariyya";
    /// a sun letter, doubled by the article's assimilated lām
    pub const SHAMSIYYA: &str = "shamsiyya";
    /// the hamza and vowel of a hamzat al-waṣl, at the start of an utterance
    pub const HAMZAT_WASL: &str = "hamzat-wasl";
    /// a nūn sākin or tanwīn merged into a following lām or rāʾ (idghām without ghunna): the n is
    /// not said and the lām or rāʾ is doubled
    pub const IDGHAM: &str = "idgham";
    /// a vowel added to join a sākin word end to a hamzat al-waṣl
    pub const HELPING_VOWEL: &str = "helping-vowel";
    /// a long vowel shortened before a sākin consonant across words
    pub const SHORTENED: &str = "shortened";
    /// the form a word takes in pause
    pub const WAQF: &str = "waqf";
    /// tāʾ marbūṭa
    pub const TAA_MARBUTA: &str = "taa-marbuta";
    /// the vowel was not written and was guessed
    pub const GUESSED: &str = "guessed";
}

use rules::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArabicOptions {
    /// seconds per ḥaraka (count): a short vowel's length (default 0.1)
    #[serde(default = "default_count")]
    pub count: f64,
    /// read the last word of each phrase in pause (waqf)
    #[serde(default = "default_true")]
    pub waqf: bool,
    /// recite: apply the tajweed (madd lengths, ghunna, idghām, iqlāb, ikhfāʾ), as Ḥafṣ ʿan ʿĀṣim
    /// by way of al-Shāṭibiyya, on a reciting tone: no word accents, the pitch drifting down
    /// through the āya and falling at its end
    #[serde(default)]
    pub recite: bool,
    /// counts of a madd ʿāriḍ (a long vowel before the last letter in pause): 2, 4 or 6
    #[serde(default = "default_arid")]
    pub arid: f64,
}

fn default_arid() -> f64 {
    4.0
}

/// A ḥaraka at a careful reading pace: short vowels of 100 ms, long ones 200 ms. (MSA short vowels
/// run 60–90 ms in conversation; at 80 ms the articulated voice lost words to Whisper: training
/// CER 14.4 → 19.9 %.)
fn default_count() -> f64 {
    0.1
}
fn default_true() -> bool {
    true
}

impl Default for ArabicOptions {
    fn default() -> Self {
        ArabicOptions { count: default_count(), waqf: true, recite: false, arid: default_arid() }
    }
}

/// One phrase as the voice says it, with what each segment was read from.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArabicPhrase {
    pub segments: Vec<Segment>,
    pub intonation: Intonation,
    /// pause after the phrase (s)
    pub pause: f64,
    /// the written words, one per word end in `segments`
    pub words: Vec<String>,
    /// per segment: the characters of the text it was read from, `[start, end)` in code points
    pub sources: Vec<[usize; 2]>,
    /// per segment: its length in ḥarakāt
    pub counts: Vec<f64>,
    /// per segment: the rules that shaped it
    pub rules: Vec<Vec<&'static str>>,
    /// the reading in IPA (long and doubled segments with ː, heavy vowels with ˤ); it tokenizes
    /// back into the same segments
    pub ipa: String,
    /// letters whose vowel was not written
    pub guessed: usize,
    /// the tajweed of the written text: rules on spans of it (see [`annotate`])
    pub annotations: Vec<Annotation>,
}

/// A tajweed rule on a span of the text, `[start, end)` in code points. The rule names follow the
/// quran-tajweed data (cpfair/quran-tajweed, CC BY 4.0), the de-facto names for tajweed colouring:
/// `madd_2`, `madd_246`, `madd_muttasil`, `madd_munfasil`, `madd_6`, `ghunnah`, `idghaam_ghunnah`,
/// `idghaam_no_ghunnah`, `idghaam_shafawi`, `idghaam_mutajanisayn`, `idghaam_mutaqaribayn`,
/// `ikhfa`, `ikhfa_shafawi`, `iqlab`, `qalqalah`, `hamzat_wasl`, `lam_shamsiyyah`, `silent`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Annotation {
    pub rule: &'static str,
    pub start: usize,
    pub end: usize,
}

// ---------------------------------------------------------------- the script

const FATHATAN: char = '\u{64B}';
const DAMMATAN: char = '\u{64C}';
const KASRATAN: char = '\u{64D}';
const FATHA: char = '\u{64E}';
const DAMMA: char = '\u{64F}';
const KASRA: char = '\u{650}';
const SHADDA_MARK: char = '\u{651}';
const SUKUN: char = '\u{652}';
const MADDAH: char = '\u{653}';
const HAMZA_ABOVE: char = '\u{654}';
const HAMZA_BELOW: char = '\u{655}';
const SUBSCRIPT_ALEF: char = '\u{656}';
const INVERTED_DAMMA: char = '\u{657}';
const DAGGER_ALEF: char = '\u{670}';
const WASLA: char = 'ٱ';
const TATWEEL: char = 'ـ';
const SMALL_HIGH_ZERO: char = '\u{6DF}';
const SMALL_HIGH_RECT_ZERO: char = '\u{6E0}';
const SMALL_DOTLESS_KHAH: char = '\u{6E1}';
const SMALL_HIGH_MEEM: char = '\u{6E2}';
const SMALL_HIGH_MADDA: char = '\u{6E4}';
const SMALL_WAW: char = '\u{6E5}';
const SMALL_YEH: char = '\u{6E6}';
const SMALL_LOW_MEEM: char = '\u{6ED}';
const OPEN_FATHATAN: char = '\u{8F0}';
const OPEN_DAMMATAN: char = '\u{8F1}';
const OPEN_KASRATAN: char = '\u{8F2}';

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum V {
    A,
    I,
    U,
}

impl V {
    fn ipa(self) -> &'static str {
        match self {
            V::A => "a",
            V::I => "i",
            V::U => "u",
        }
    }
}

/// A written letter and its marks.
#[derive(Clone, Debug)]
struct Letter {
    ch: char,
    vowel: Option<V>,
    tanwin: Option<V>,
    shadda: bool,
    sukun: bool,
    /// superscript (dagger) alif: a long ā on this letter
    dagger: bool,
    maddah: bool,
    /// not pronounced (ۛ۟, or found silent by the spelling repairs)
    silent: bool,
    /// pronounced only in pause (۠: the alif of أنا)
    pause_only: bool,
    /// ۤ: a lengthened madd
    long_madd: bool,
    iqlab: bool,
    /// a long vowel follows this letter's vowel (small wāw/yāʾ of the pronoun, subscript alif)
    long_after: Option<V>,
    /// the lām of the name Allāh
    allah: bool,
    /// its long ā is not written (added by the spelling repairs)
    hidden: bool,
    src: [usize; 2],
    /// where the tanwīn mark, the dagger alif and a small wāw/yāʾ are (code points)
    tanwin_at: usize,
    dagger_at: usize,
    long_after_at: usize,
}

impl Letter {
    fn new(ch: char, at: usize) -> Letter {
        Letter { ch, vowel: None, tanwin: None, shadda: false, sukun: false, dagger: false, maddah: false, silent: false, pause_only: false, long_madd: false, iqlab: false, long_after: None, allah: false, hidden: false, src: [at, at + 1], tanwin_at: at, dagger_at: at, long_after_at: at }
    }
    /// carries no vowel sign, no sukūn and no shadda
    fn bare(&self) -> bool {
        self.vowel.is_none() && self.tanwin.is_none() && !self.sukun && !self.shadda && !self.dagger
    }
}

#[derive(Clone, Debug)]
struct Word {
    letters: Vec<Letter>,
    text: String,
}

impl Word {
    /// The disjoint letters opening some sūras (الٓمٓ, كٓهيعٓصٓ, طه, يسٓ, …): read as letter names. Their
    /// letters carry the maddah without a vowel.
    fn muqattaat(&self) -> bool {
        self.letters.iter().any(|l| l.maddah && l.vowel.is_none() && l.long_after.is_none() && !matches!(l.ch, 'ا' | 'و' | 'ي' | 'ى' | 'آ') && !l.dagger && consonant(l.ch).is_some())
    }

    /// Written with vowel marks: then a bare consonant is sākin (Uthmani script leaves the sukūn
    /// off a letter that merges into or hides before the next), not a vowel to guess.
    fn vowelled(&self) -> bool {
        self.letters.iter().any(|l| l.vowel.is_some() || l.tanwin.is_some() || l.sukun || l.shadda)
    }
}

fn is_mark(c: char) -> bool {
    matches!(c, '\u{610}'..='\u{61A}' | '\u{64B}'..='\u{65F}' | '\u{670}' | '\u{6D6}'..='\u{6DC}' | '\u{6DF}'..='\u{6E8}' | '\u{6EA}'..='\u{6ED}' | '\u{8D3}'..='\u{8FF}')
}

fn is_letter(c: char) -> bool {
    matches!(c, '\u{621}'..='\u{63A}' | '\u{641}'..='\u{64A}' | 'ٱ' | 'ک' | 'ی' | 'پ' | 'چ' | 'ژ' | 'گ' | 'ڤ' | 'ڨ' | 'ے')
}

/// Letters that end a phrase (punctuation, the end of an āya, the stop signs that call for a pause).
fn is_phrase_break(c: char) -> bool {
    matches!(c, '.' | '!' | '?' | '؟' | '،' | ',' | '؛' | ';' | ':' | '\n' | '۝' | '\u{6D8}' | '\u{6D7}' | '۞' | '\u{FD3E}' | '\u{FD3F}' | '«' | '»')
}

fn is_question(c: char) -> bool {
    c == '?' || c == '؟'
}

/// One written phrase: its words and how it ends.
struct RawPhrase {
    words: Vec<Word>,
    question: bool,
}

fn parse(text: &str) -> Vec<RawPhrase> {
    let mut phrases = vec![];
    let mut words: Vec<Word> = vec![];
    let mut cur: Option<Word> = None;
    let flush_word = |cur: &mut Option<Word>, words: &mut Vec<Word>| {
        if let Some(mut w) = cur.take() {
            settle(&mut w);
            if !w.letters.is_empty() {
                words.push(w);
            }
        }
    };
    for (at, c) in text.chars().enumerate() {
        let c = match c {
            'ک' => 'ك',
            'ی' | 'ے' => 'ي',
            'ڨ' => 'ق',
            _ => c,
        };
        if is_letter(c) {
            let w = cur.get_or_insert_with(|| Word { letters: vec![], text: String::new() });
            // آ is an alif with maddah
            let mut l = Letter::new(if c == 'آ' { 'ا' } else { c }, at);
            l.maddah = c == 'آ';
            w.letters.push(l);
            w.text.push(c);
            continue;
        }
        if c == TATWEEL || c == '\u{200C}' || c == '\u{200D}' {
            if let Some(w) = cur.as_mut() {
                w.text.push(c);
                // a tatweel can carry a hamza (ـٔ) or a dagger alif (ـٰ): hold it as a letter until
                // its marks say which (see `settle`)
                if c == TATWEEL {
                    w.letters.push(Letter::new(TATWEEL, at));
                }
            }
            continue;
        }
        if is_mark(c) || c == SMALL_WAW || c == SMALL_YEH {
            let Some(w) = cur.as_mut() else { continue };
            w.text.push(c);
            let Some(l) = w.letters.last_mut() else { continue };
            l.src[1] = at + 1;
            match c {
                FATHA => l.vowel = Some(V::A),
                KASRA => l.vowel = Some(V::I),
                DAMMA => l.vowel = Some(V::U),
                FATHATAN | OPEN_FATHATAN => (l.tanwin, l.tanwin_at) = (Some(V::A), at),
                KASRATAN | OPEN_KASRATAN => (l.tanwin, l.tanwin_at) = (Some(V::I), at),
                DAMMATAN | OPEN_DAMMATAN => (l.tanwin, l.tanwin_at) = (Some(V::U), at),
                SHADDA_MARK => l.shadda = true,
                SUKUN | SMALL_DOTLESS_KHAH => l.sukun = true,
                MADDAH => l.maddah = true,
                SMALL_HIGH_MADDA => l.long_madd = true,
                DAGGER_ALEF => (l.dagger, l.dagger_at) = (true, at),
                SMALL_HIGH_ZERO => l.silent = true,
                SMALL_HIGH_RECT_ZERO => l.pause_only = true,
                SMALL_HIGH_MEEM | SMALL_LOW_MEEM => l.iqlab = true,
                SMALL_WAW | INVERTED_DAMMA => (l.long_after, l.long_after_at) = (Some(V::U), at),
                SMALL_YEH | SUBSCRIPT_ALEF => (l.long_after, l.long_after_at) = (Some(V::I), at),
                HAMZA_ABOVE => {
                    l.ch = match l.ch {
                        'ا' => 'أ',
                        'و' => 'ؤ',
                        'ي' | 'ى' => 'ئ',
                        TATWEEL => 'ء',
                        x => x,
                    }
                }
                HAMZA_BELOW => {
                    if l.ch == 'ا' {
                        l.ch = 'إ'
                    }
                }
                _ => {}
            }
            continue;
        }
        if c == HAMZA_ABOVE {
            continue;
        }
        // a hamza carried by a tatweel ("ـٔ") is a hamza on its own
        flush_word(&mut cur, &mut words);
        if is_phrase_break(c) && !words.is_empty() {
            phrases.push(RawPhrase { words: std::mem::take(&mut words), question: is_question(c) });
        }
    }
    flush_word(&mut cur, &mut words);
    if !words.is_empty() {
        phrases.push(RawPhrase { words, question: false });
    }
    phrases
}

/// A tatweel that carried no hamza is no letter: its dagger alif (ـٰ) belongs to the letter before.
fn settle(w: &mut Word) {
    let mut i = 0;
    while i < w.letters.len() {
        if w.letters[i].ch != TATWEEL {
            i += 1;
            continue;
        }
        let t = w.letters.remove(i);
        if i > 0 {
            let p = &mut w.letters[i - 1];
            if t.dagger {
                (p.dagger, p.dagger_at) = (true, t.dagger_at);
            }
            p.maddah |= t.maddah;
            p.src[1] = t.src[1];
        }
    }
}

// ---------------------------------------------------------------- spelling repairs

/// The letters of a word without marks, the wasla read as alif.
fn skeleton(w: &Word) -> String {
    w.letters.iter().map(|l| if l.ch == WASLA { 'ا' } else { l.ch }).collect()
}

/// Words whose standard spelling leaves out a long ā (or writes a letter that isn't said):
/// (skeleton, letters that carry a hidden ā, letters that are silent). Matched with or without one
/// proclitic (و ف ب ك ل).
const HIDDEN: [(&str, &[usize], &[usize]); 22] = [
    ("هذا", &[0], &[]),
    ("هذه", &[0], &[]),
    ("هذان", &[0], &[]),
    ("هذين", &[0], &[]),
    ("هؤلاء", &[0], &[]),
    ("هكذا", &[0], &[]),
    ("ذلك", &[0], &[]),
    ("ذلكم", &[0], &[]),
    ("ذلكما", &[0], &[]),
    ("لكن", &[0], &[]),
    ("لكنه", &[0], &[]),
    ("لكنهم", &[0], &[]),
    ("الله", &[2], &[]),
    ("لله", &[1], &[]),
    ("اللهم", &[2], &[]),
    ("إله", &[1], &[]),
    ("الإله", &[3], &[]),
    ("الرحمن", &[4], &[]),
    ("رحمن", &[2], &[]),
    ("أولئك", &[2], &[1]),
    ("أولو", &[], &[1]),
    ("أولي", &[], &[1]),
];

const PROCLITICS: [char; 5] = ['و', 'ف', 'ب', 'ك', 'ل'];

fn repair(w: &mut Word) {
    let sk: Vec<char> = skeleton(w).chars().collect();
    for (form, hidden, silent) in HIDDEN {
        let f: Vec<char> = form.chars().collect();
        // with or without one proclitic (and the oath's تَ before الله)
        let prefixed = sk.len() == f.len() + 1 && sk[1..] == f[..] && (PROCLITICS.contains(&sk[0]) || sk[0] == 'ت' && form == "الله");
        let off = if sk == f {
            0
        } else if prefixed {
            1
        } else {
            continue;
        };
        // أُولُو "possessors of" (ʔu…), not أَوَلَوْ "or if"
        if form.starts_with('أ') && w.letters[off].vowel.is_some_and(|v| v != V::U) {
            continue;
        }
        for &k in hidden {
            let l = &mut w.letters[off + k];
            if !l.dagger {
                l.hidden = true;
            }
            l.dagger = true;
        }
        for &k in silent {
            w.letters[off + k].silent = true;
        }
        if form.contains("لله") {
            w.letters[off + hidden[0]].allah = true;
        }
        return;
    }
}

// ---------------------------------------------------------------- reading

/// A sound as read, before timing.
#[derive(Clone, Debug)]
struct Phone {
    /// consonant IPA, or the vowel's quality
    ipa: &'static str,
    vowel: Option<V>,
    long: bool,
    /// doubled consonant
    gem: bool,
    /// heavy (tafkhīm): a pharyngealized vowel
    heavy: bool,
    /// the letter a consonant was read from
    letter: char,
    /// the vowel of a hamzat al-waṣl
    wasl: bool,
    allah: bool,
    src: [usize; 2],
    rules: Vec<&'static str>,
    word: usize,
    /// recited length in ḥarakāt, when tajweed sets it (a madd of 4 or 6, a ghunna of 2)
    counts: Option<f64>,
    nasalized: bool,
    /// merged into the next letter (idghām): not said
    drop: bool,
}

impl Phone {
    fn cons(ipa: &'static str, letter: char, src: [usize; 2], word: usize) -> Phone {
        Phone { ipa, vowel: None, long: false, gem: false, heavy: false, letter, wasl: false, allah: false, src, rules: vec![], word, counts: None, nasalized: false, drop: false }
    }
    fn vowel(v: V, src: [usize; 2], word: usize) -> Phone {
        Phone { ipa: v.ipa(), vowel: Some(v), long: false, gem: false, heavy: false, letter: '\0', wasl: false, allah: false, src, rules: vec![], word, counts: None, nasalized: false, drop: false }
    }
    fn is_vowel(&self) -> bool {
        self.vowel.is_some()
    }
    fn tag(mut self, r: &'static str) -> Self {
        self.rules.push(r);
        self
    }
}

/// The consonant a letter stands for (None for the madd letters and the seats handled apart).
fn consonant(c: char) -> Option<&'static str> {
    Some(match c {
        'ء' | 'أ' | 'إ' | 'ؤ' | 'ئ' | 'آ' => "ʔ",
        'ب' => "b",
        'ت' => "t",
        'ث' => "θ",
        'ج' => "dʒ",
        'ح' => "ħ",
        'خ' => "χ",
        'د' => "d",
        'ذ' => "ð",
        'ر' => "ɾ",
        'ز' => "z",
        'س' => "s",
        'ش' => "ʃ",
        'ص' => "sˤ",
        'ض' => "dˤ",
        'ط' => "tˤ",
        'ظ' => "ðˤ",
        'ع' => "ʕ",
        'غ' => "ʁ",
        'ف' => "f",
        'ق' => "q",
        'ك' => "k",
        'ل' => "l",
        'م' => "m",
        'ن' => "n",
        'ه' => "h",
        'و' => "w",
        'ي' => "j",
        'ة' => "t",
        'پ' => "p",
        'چ' => "tʃ",
        'ژ' => "ʒ",
        'گ' => "g",
        'ڤ' => "v",
        _ => return None,
    })
}

/// The heavy letters (al-mustaʿliya): their vowels are always heavy.
fn heavy_letter(c: char) -> bool {
    matches!(c, 'خ' | 'ص' | 'ض' | 'غ' | 'ط' | 'ق' | 'ظ')
}

/// The sun letters: the article's lām assimilates into them.
fn sun_letter(c: char) -> bool {
    matches!(c, 'ت' | 'ث' | 'د' | 'ذ' | 'ر' | 'ز' | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ل' | 'ن')
}

fn qalqala_letter(c: char) -> bool {
    matches!(c, 'ق' | 'ط' | 'ب' | 'ج' | 'د')
}

fn hamza_letter(c: char) -> bool {
    matches!(c, 'ء' | 'أ' | 'إ' | 'ؤ' | 'ئ' | 'آ')
}

/// Small words said without stress in running speech.
const PARTICLES: [&str; 18] = ["في", "من", "إلى", "على", "عن", "أن", "إن", "لا", "ما", "لم", "لن", "قد", "ثم", "أو", "بل", "هل", "يا", "مع"];

/// Whether letter `i` of a word is the article's lām: ال, after a proclitic (وَال, بِال, وَبِال),
/// or after li- (لِلْ, whose alif the spelling drops).
fn article_lam(w: &Word, i: usize) -> bool {
    let l = &w.letters;
    if i + 1 >= l.len() || l[i].ch != 'ل' || !(l[i].bare() || l[i].sukun) || l[i].allah {
        return false;
    }
    let alif = |k: usize| matches!(l[k].ch, 'ا' | WASLA) && l[k].vowel.is_none() && !l[k].maddah;
    // li- (doubled when a nūn or tanwīn before it merges in: هُدًى لِّلنَّاسِ)
    let li = |k: usize| l[k].ch == 'ل' && matches!(l[k].vowel, Some(V::I) | None) && !l[k].sukun;
    let pro = |k: usize| PROCLITICS.contains(&l[k].ch) && !l[k].sukun && !l[k].shadda;
    match i {
        1 => alif(0) || li(0),
        2 => alif(1) && pro(0) || li(1) && matches!(l[0].ch, 'و' | 'ف'),
        3 => alif(2) && pro(1) && pro(0),
        _ => false,
    }
}

struct Reader<'a> {
    phones: Vec<Phone>,
    guessed: usize,
    words: &'a [Word],
}

impl Reader<'_> {
    fn read_word(&mut self, wi: usize, pause: bool) {
        let w = &self.words[wi];
        let l = &w.letters;
        let n = l.len();
        let start = self.phones.len();
        let utterance_start = start == 0;
        // the last letter that is said (silent letters and a final seat alif don't count)
        let mut i = 0;
        while i < n {
            let x = &l[i];
            // the word's last sounding letter: what follows is silent, or the seat of this letter's
            // tanwīn fatḥ (ـًا, ـًى), or the alif after a plural wāw
            let rest = &l[i + 1..];
            let seat = |y: &Letter| matches!(y.ch, 'ا' | 'ى') && y.bare() && !y.maddah;
            let last = rest.iter().all(|y| y.silent) || rest.len() == 1 && seat(&rest[0]) && (x.tanwin == Some(V::A) || x.ch == 'و' && rest[0].ch == 'ا');
            if x.silent || (x.pause_only && !pause) {
                i += 1;
                continue;
            }
            match x.ch {
                // ---- hamzat al-waṣl
                WASLA | 'ا' if i == 0 && x.vowel.is_none() && x.tanwin.is_none() && !x.maddah && (x.ch == WASLA || article_lam(w, 1) || l.get(1).is_some_and(|y| y.sukun || y.ch == 'ل' && y.shadda)) => {
                    if utterance_start {
                        // the article (and الَّذِي, اللَّه) takes a; otherwise u before a ḍamma, else i
                        let v = if l.get(1).is_some_and(|y| y.ch == 'ل') {
                            V::A
                        } else if l.get(2).and_then(|y| y.vowel) == Some(V::U) {
                            V::U
                        } else {
                            V::I
                        };
                        self.phones.push(Phone::cons("ʔ", 'ء', x.src, wi).tag(HAMZAT_WASL));
                        let mut p = Phone::vowel(v, x.src, wi).tag(HAMZAT_WASL);
                        p.wasl = true;
                        self.phones.push(p);
                    } else {
                        self.join_wasl(wi);
                    }
                }
                WASLA => {
                    // after a proclitic (وَٱلْ…): silent
                }
                // the article's alif after a proclitic (وَالْ, بِالْ): silent
                'ا' if i > 0 && x.bare() && !x.maddah && article_lam(w, i + 1) => {}
                // an alif written with a vowel stands for a hamza (اِقْرَأْ)
                'ا' if x.vowel.is_some() || x.tanwin.is_some() => {
                    self.phones.push(Phone::cons("ʔ", 'أ', x.src, wi));
                    self.vowel_of(x, wi, last && pause, i + 1 < n);
                }
                // an alif maqṣūra with tanwīn (هدىً): the vowel and its n
                'ى' if x.tanwin.is_some() => {
                    if !self.after_vowel(start) {
                        self.vowel_of(x, wi, last && pause, false);
                    }
                }
                // ---- alif: a madd letter, a seat, or (written bare at a word start) a hamza
                'ا' | 'ى' if !x.shadda && x.vowel.is_none() && !(x.ch == 'ى' && x.tanwin.is_some()) => {
                    let prev_letter = if i > 0 { Some(&l[i - 1]) } else { None };
                    // a maddah after fatḥa lengthens the madd before a hamza, a sākin or a word end
                    // (the Qurʾān's ـَآ, جَآءَ); otherwise آ is a hamza and long ā (مَآرِب)
                    let madd_sign = self.after_short(V::A, start) && l.get(i + 1).is_none_or(|y| hamza_letter(y.ch) || y.shadda || y.sukun);
                    if x.maddah && !madd_sign {
                        // آ: hamza and long ā
                        self.phones.push(Phone::cons("ʔ", 'آ', x.src, wi));
                        let mut v = Phone::vowel(V::A, x.src, wi).tag(MADD);
                        v.long = true;
                        self.phones.push(v);
                    } else if prev_letter.is_some_and(|p| p.tanwin == Some(V::A)) {
                        // the seat of tanwīn fatḥ: said only in pause (handled with the tanwīn)
                    } else if x.ch == 'ا' && prev_letter.is_some_and(|p| p.ch == 'و') && last {
                        // the alif after a plural wāw (كَتَبُوا): silent
                    } else if i == 0 {
                        // a bare alif starting a word stands for a hamza
                        self.phones.push(Phone::cons("ʔ", 'أ', x.src, wi));
                        let v = x.vowel.unwrap_or(V::A);
                        self.phones.push(Phone::vowel(v, x.src, wi).tag(GUESSED));
                        self.guessed += 1;
                    } else if x.ch == 'ى' && self.after_short(V::I, start) {
                        // a dotless final yāʾ after kasra (فى): ī
                        self.lengthen(x, wi, l.get(i + 1));
                    } else {
                        if !self.after_vowel(start) {
                            // a consonant without a vowel before the alif: its vowel is fatḥa
                            self.phones.push(Phone::vowel(V::A, x.src, wi).tag(GUESSED));
                            self.guessed += 1;
                        }
                        self.lengthen(x, wi, l.get(i + 1));
                    }
                }
                // ---- wāw and yāʾ: consonants, or madd letters after their vowel, or diphthongs
                'و' | 'ي' if x.vowel.is_none() && x.tanwin.is_none() && !x.shadda && i > 0 => {
                    let v = if x.ch == 'و' { V::U } else { V::I };
                    if x.dagger {
                        // a wāw/yāʾ carrying the dagger alif is its seat (ٱلصَّلَوٰة): ā
                        self.lengthen(x, wi, l.get(i + 1));
                    } else if self.after_short(v, start) {
                        self.lengthen(x, wi, l.get(i + 1));
                    } else if !self.after_vowel(start) && !x.sukun && (i + 1 == n || !l[i + 1].is_vowel_bearing()) {
                        // a consonant with no vowel, then a bare wāw/yāʾ before a consonant: the
                        // madd letter gives the vowel
                        self.phones.push(Phone::vowel(v, x.src, wi).tag(GUESSED));
                        self.guessed += 1;
                        self.lengthen(x, wi, l.get(i + 1));
                    } else {
                        // a diphthong (aw, aj) or a consonant; unvowelled after fatḥa, a diphthong
                        let ipa = if x.ch == 'و' { "w" } else { "j" };
                        self.phones.push(Phone::cons(ipa, x.ch, x.src, wi));
                        if !(x.bare() && self.phones.len() >= 2 && self.phones[self.phones.len() - 2].vowel == Some(V::A)) {
                            self.vowel_of(x, wi, last && pause, i + 1 < n);
                        }
                    }
                }
                // ---- tāʾ marbūṭa: t joined, h in pause
                'ة' if last && pause => {
                    self.phones.push(Phone::cons("h", 'ة', x.src, wi).tag(TAA_MARBUTA).tag(WAQF));
                }
                // ---- the article's lām
                'ل' if article_lam(w, i) => {
                    let next = &l[i + 1];
                    if x.sukun || (!next.shadda && !sun_letter(next.ch)) {
                        self.phones.push(Phone::cons("l", 'ل', x.src, wi).tag(QAMARIYYA));
                    }
                    // before a sun letter the lām is silent and the letter doubled (its shadda, or
                    // added here for unvowelled text)
                }
                _ => {
                    let Some(ipa) = consonant(x.ch) else {
                        i += 1;
                        continue;
                    };
                    let mut c = Phone::cons(ipa, x.ch, x.src, wi);
                    if x.ch == 'ة' {
                        c = c.tag(TAA_MARBUTA);
                    }
                    let prev_article = i > 0 && article_lam(w, i - 1) && !l[i - 1].sukun;
                    if x.shadda || (prev_article && sun_letter(x.ch)) {
                        c.gem = true;
                        c = c.tag(SHADDA);
                        if prev_article {
                            c = c.tag(SHAMSIYYA);
                            c.src[0] = l[i - 1].src[0];
                        }
                        if matches!(ipa, "n" | "m") {
                            c = c.tag(GHUNNA);
                        }
                    }
                    if x.allah {
                        c.allah = true;
                        c = c.tag(ALLAH);
                    }
                    if x.iqlab {
                        c = c.tag(IQLAB);
                    }
                    self.phones.push(c);
                    if x.maddah && hamza_letter(x.ch) {
                        // a hamza with maddah (ءَا written as one): ʔaː
                        let mut v = Phone::vowel(V::A, x.src, wi).tag(MADD);
                        v.long = true;
                        self.phones.push(v);
                    } else {
                        self.vowel_of(x, wi, last && pause, i + 1 < n);
                    }
                }
            }
            i += 1;
        }
        // a word's sounds end at its last segment
        let _ = start;
    }

    /// The vowel (and tanwīn) a consonant letter carries.
    fn vowel_of(&mut self, x: &Letter, wi: usize, pause_final: bool, more: bool) {
        let w = &self.words[wi];
        let at = w.letters.iter().position(|y| y.src == x.src).unwrap_or(0);
        let next = w.letters.get(at + 1);
        if let Some(t) = x.tanwin {
            if pause_final {
                // waqf: tanwīn fatḥ becomes ā, the others drop (on tāʾ marbūṭa: h, handled above)
                if t == V::A && x.ch != 'ة' {
                    let mut v = Phone::vowel(V::A, x.src, wi).tag(WAQF);
                    v.long = true;
                    if let Some(s) = next.filter(|y| matches!(y.ch, 'ا' | 'ى')) {
                        v.src[1] = s.src[1];
                    }
                    self.phones.push(v);
                }
                return;
            }
            self.phones.push(Phone::vowel(t, x.src, wi));
            self.phones.push(Phone::cons("n", 'ن', x.src, wi).tag(TANWIN).tag(NUN_SAKIN));
            return;
        }
        if let Some(v) = x.vowel {
            if pause_final && x.long_after.is_none() && !x.dagger {
                // waqf: the final short vowel drops
                if let Some(c) = self.phones.last_mut() {
                    c.rules.push(WAQF);
                }
                return;
            }
            let mut p = Phone::vowel(v, x.src, wi);
            if x.dagger || x.long_after.is_some() {
                if pause_final && x.long_after.is_some() {
                    // the pronoun's ṣila is not said in pause
                    if let Some(c) = self.phones.last_mut() {
                        c.rules.push(WAQF);
                    }
                    return;
                }
                p.long = true;
                p.rules.push(MADD);
                if x.long_madd || x.maddah {
                    p.rules.push(MADD_SIGN);
                }
            }
            self.phones.push(p);
            return;
        }
        if x.dagger {
            let mut p = Phone::vowel(V::A, x.src, wi).tag(MADD);
            p.long = true;
            self.phones.push(p);
            return;
        }
        if x.sukun || x.shadda && !more {
            return;
        }
        // no vowel written: a following madd letter tells it; a word's last letter is sākin
        if !more || pause_final {
            return;
        }
        let vowelled = w.vowelled();
        if let Some(n) = next {
            let v = match n.ch {
                'ا' | 'ى' if n.vowel.is_none() => Some(V::A),
                'و' if n.bare() => Some(V::U),
                'ي' if n.bare() => Some(V::I),
                _ => None,
            };
            if v.is_some() {
                // the madd letter adds the vowel and lengthens it
                return;
            }
        }
        if vowelled {
            return;
        }
        self.phones.push(Phone::vowel(V::A, x.src, wi).tag(GUESSED));
        self.guessed += 1;
    }

    fn after_vowel(&self, start: usize) -> bool {
        self.phones.len() > start && self.phones.last().is_some_and(|p| p.is_vowel())
    }

    fn after_short(&self, v: V, start: usize) -> bool {
        self.phones.len() > start && self.phones.last().is_some_and(|p| p.vowel == Some(v) && !p.long)
    }

    /// A madd letter: lengthen the vowel before it (adding it when the consonant had none).
    fn lengthen(&mut self, x: &Letter, wi: usize, next: Option<&Letter>) {
        let v = match self.phones.last() {
            Some(p) if p.is_vowel() => p.vowel.unwrap(),
            _ => {
                let v = if x.ch == 'و' { V::U } else if x.ch == 'ي' { V::I } else { V::A };
                self.phones.push(Phone::vowel(v, x.src, wi).tag(GUESSED));
                self.guessed += 1;
                v
            }
        };
        let _ = v;
        let p = self.phones.last_mut().unwrap();
        p.long = true;
        p.src[1] = p.src[1].max(x.src[1]);
        p.rules.push(MADD);
        if x.maddah || x.long_madd {
            p.rules.push(MADD_SIGN);
        }
        if let Some(n) = next {
            if hamza_letter(n.ch) {
                p.rules.push(MADD_MUTTASIL);
            } else if n.shadda || n.sukun {
                p.rules.push(MADD_LAZIM);
            }
        }
    }

    /// A hamzat al-waṣl inside an utterance: elided; the word before joins to the sākin after it.
    fn join_wasl(&mut self, wi: usize) {
        let Some(last) = self.phones.last_mut() else { return };
        if last.word + 1 != wi {
            return;
        }
        if last.is_vowel() {
            if last.long {
                last.long = false;
                last.rules.retain(|r| *r != MADD);
                last.rules.push(SHORTENED);
            }
            return;
        }
        // a sākin end meets the sākin after the waṣl: a vowel joins them (مِنَ, هُمُ, -ini)
        let prev_word = skeleton(&self.words[last.word]);
        let v = if prev_word == "من" {
            V::A
        } else if last.ipa == "m" && (prev_word.ends_with("هم") || prev_word.ends_with("كم") || prev_word.ends_with("تم")) {
            V::U
        } else {
            V::I
        };
        let (src, word) = (last.src, last.word);
        self.phones.push(Phone::vowel(v, src, word).tag(HELPING_VOWEL));
    }
}

impl Letter {
    fn is_vowel_bearing(&self) -> bool {
        self.vowel.is_some() || self.tanwin.is_some() || self.shadda
    }
}

/// Idghām without ghunna: a nūn sākin or tanwīn at a word's end merges into a lām or rāʾ starting
/// the next word (يَكُن لَّهُۥ → yakul-lahū). The n goes; the lām or rāʾ is doubled (the vowelled text
/// marks it with shadda already; unvowelled text gets it here). Ḥafṣ makes no exception before
/// these two letters. (Idghām with ghunna, iqlāb and ikhfāʾ come with the tajweed step.)
fn idgham_without_ghunna(ph: &mut Vec<Phone>) {
    let mut i = 0;
    while i + 1 < ph.len() {
        let n = &ph[i];
        let merges = n.letter == 'ن' && !n.is_vowel() && !n.gem && ph[i + 1].word != n.word && matches!(ph[i + 1].letter, 'ل' | 'ر') && !ph[i + 1].is_vowel();
        if merges {
            let src0 = ph[i].src[0].min(ph[i + 1].src[0]);
            let next = &mut ph[i + 1];
            next.gem = true;
            next.rules.push(IDGHAM);
            if !next.rules.contains(&SHADDA) {
                next.rules.push(SHADDA);
            }
            // the merged consonant is said from the n's letter on (the whole span lights)
            let _ = src0;
            ph.remove(i);
            continue;
        }
        i += 1;
    }
}

// ---------------------------------------------------------------- tajweed annotations

fn is_hamza_or_throat(c: char) -> bool {
    matches!(c, 'ء' | 'أ' | 'إ' | 'ؤ' | 'ئ' | 'آ' | 'ه' | 'ع' | 'ح' | 'غ' | 'خ')
}

/// The pairs that merge when the first is sākin: same place (mutajānisayn) and near places
/// (mutaqāribayn).
fn merging_pair(a: char, b: char) -> Option<&'static str> {
    match (a, b) {
        ('د', 'ت') | ('ت', 'د') | ('ت', 'ط') | ('ط', 'ت') | ('ث', 'ذ') | ('ذ', 'ظ') | ('ب', 'م') => Some("idghaam_mutajanisayn"),
        ('ل', 'ر') | ('ق', 'ك') => Some("idghaam_mutaqaribayn"),
        _ => None,
    }
}

/// The tajweed of a written phrase (its last word read in pause when `pause`), letter by letter as
/// tajweed is taught. The spans follow the quran-tajweed data: a madd on its madd letter, ghunna on
/// the doubled nūn or mīm, qalqala on its letter, a nūn or mīm sākin (or the tanwīn mark) through
/// the letter after it, an idghām of like or near letters on the letter that merges, and the silent
/// letters (hamzat al-waṣl, the sun lām, the marked ones) on themselves.
fn annotate_words(words: &[Word], pause: bool) -> Vec<Annotation> {
    let mut out: Vec<Annotation> = vec![];
    let flat: Vec<(usize, usize)> = words.iter().enumerate().flat_map(|(wi, w)| (0..w.letters.len()).map(move |li| (wi, li))).collect();
    let at = |k: usize| &words[flat[k].0].letters[flat[k].1];
    let nw = words.len();
    // a letter that is not said: marked silent, the seat of a tanwīn fatḥ, the alif after a plural wāw
    let unsaid = |k: usize| -> bool {
        let (wi, li) = flat[k];
        let l = at(k);
        let w = &words[wi];
        let prev = if li > 0 { Some(&w.letters[li - 1]) } else { None };
        l.silent
            || (l.pause_only && !(pause && wi + 1 == nw))
            // the seat of a tanwīn fatḥ, unless the word ends in pause (then it is the ā said)
            || matches!(l.ch, 'ا' | 'ى') && l.bare() && !l.maddah && prev.is_some_and(|p| p.tanwin.is_some()) && !(pause && wi + 1 == nw)
            || l.ch == 'ا' && l.bare() && li + 1 == w.letters.len() && prev.is_some_and(|p| p.ch == 'و' && (p.bare() || p.sukun))
    };
    // the next letter that is said, across words
    let next_said = |k: usize| -> Option<usize> { (k + 1..flat.len()).find(|&j| !unsaid(j)) };
    // whether letter k is the last said letter of its word
    let last_in_word = |k: usize| -> bool { next_said(k).is_none_or(|j| flat[j].0 != flat[k].0) };
    let in_pause = |k: usize| -> bool { pause && flat[k].0 + 1 == nw && last_in_word(k) };
    let push = |out: &mut Vec<Annotation>, rule: &'static str, start: usize, end: usize| out.push(Annotation { rule, start, end });
    // Uthmani script (the muṣḥaf's spelling) marks its lengthened madds
    let uthmani = words.iter().flat_map(|w| &w.letters).any(|l| l.ch == WASLA || l.dagger && !l.hidden || l.maddah && l.ch != 'ا');
    for k in 0..flat.len() {
        let (wi, li) = flat[k];
        let w = &words[wi];
        let l = at(k);
        let vowelled = w.vowelled();
        let prev = if li > 0 { Some(&w.letters[li - 1]) } else { None };
        // ---- silent letters and the waṣl
        if l.ch == WASLA {
            // said (ʔa, ʔi) at the start of the phrase; elsewhere not
            if k > 0 {
                push(&mut out, "hamzat_wasl", l.src[0], l.src[1]);
            }
            continue;
        }
        if unsaid(k) && (l.silent || l.pause_only || l.ch == 'ا' && prev.is_some_and(|p| p.ch == 'و' && (p.bare() || p.sukun))) {
            push(&mut out, "silent", l.src[0], l.src[1]);
            continue;
        }
        if !w.muqattaat() && article_lam(w, li) && !l.sukun && w.letters.get(li + 1).is_some_and(|n| (n.shadda || sun_letter(n.ch)) && !n.allah) {
            push(&mut out, "lam_shamsiyyah", l.src[0], l.src[1]);
            continue;
        }
        // ---- madd
        // a full madd letter: alif or dotless yāʾ after fatḥa (or a dagger), dotless yāʾ after
        // kasra, wāw after ḍamma, yāʾ after kasra
        let after = |v: V| prev.is_some_and(|p| p.vowel == Some(v));
        let full_madd = match l.ch {
            'ا' | 'ى' => l.vowel.is_none() && !l.shadda && !l.sukun && l.tanwin.is_none() && !unsaid(k) && (after(V::A) || prev.is_some_and(|p| p.dagger) || l.ch == 'ى' && after(V::I)) && prev.is_some_and(|p| p.tanwin.is_none()),
            'و' => l.bare() && !l.dagger && after(V::U),
            'ي' => l.bare() && !l.dagger && after(V::I),
            _ => false,
        };
        let madd_span: Option<[usize; 2]> = if full_madd {
            Some(l.src)
        } else if l.dagger && !l.hidden && l.ch != 'ى' {
            if matches!(l.ch, 'و' | 'ي') && l.vowel.is_none() {
                // a wāw or yāʾ carrying the dagger alif is its silent seat
                push(&mut out, "silent", l.src[0], l.dagger_at);
            }
            Some([l.dagger_at, l.src[1]])
        } else {
            None
        };
        if let Some(span) = madd_span {
            let nxt = next_said(k);
            // the vocative yā is a word of its own joined in writing (يَٰٓأَيُّهَا): a hamza after it
            // starts the next word (the hā of attention, هَٰٓؤُلَآءِ, counts as one word)
            let joined_particle = li == 0 && l.dagger && l.ch == 'ي';
            let same_word_next = nxt.filter(|&j| flat[j].0 == wi && !joined_particle);
            let rule = if let Some(j) = same_word_next {
                let n = at(j);
                // the article's silent lām before a sun letter: the doubled letter follows (ءَآلذَّكَرَ)
                let doubled_after = n.ch == 'ل' && n.bare() && j + 1 < flat.len() && flat[j + 1].0 == wi && at(j + 1).shadda;
                if hamza_letter(n.ch) {
                    "madd_muttasil"
                } else if n.shadda || n.sukun || doubled_after {
                    "madd_6"
                } else if pause && wi + 1 == nw && last_in_word(j) {
                    "madd_246"
                } else {
                    "madd_2"
                }
            } else {
                // the word's last letter: before a hamza in the next word, or (shortened) a waṣl
                match nxt.map(at) {
                    Some(n) if hamza_letter(n.ch) => "madd_munfasil",
                    Some(n) if n.ch == WASLA => "",
                    _ => "madd_2",
                }
            };
            // the muṣḥaf's colouring: a written madd letter of two counts isn't marked (only the
            // dagger alif and the small wāw and yāʾ are), and in Uthmani text the lengthened madds
            // are the ones written with the maddah
            let marked = l.maddah || l.long_madd;
            let rule = match rule {
                "madd_muttasil" | "madd_munfasil" | "madd_6" if uthmani && !marked => "madd_2",
                "madd_2" if !l.dagger || l.ch == 'ى' && last_in_word(k) => "",
                r => r,
            };
            // a natural madd on a dotless yāʾ is coloured on its dagger alif, inside a word (عَلَىٰ
            // at a word's end is not coloured)
            let span = if rule == "madd_2" && l.ch == 'ى' { [l.dagger_at, l.src[1]] } else { span };
            if !rule.is_empty() {
                push(&mut out, rule, span[0], span[1]);
            }
        }
        // the pronoun's ṣila (ۥ ۦ): a madd of its own, not said in pause
        if l.long_after.is_some() && !in_pause(k) {
            let rule = match next_said(k).map(at) {
                Some(n) if hamza_letter(n.ch) && (!uthmani || l.maddah) => "madd_munfasil",
                Some(n) if n.ch == WASLA => "",
                _ => "madd_2",
            };
            if !rule.is_empty() {
                push(&mut out, rule, l.long_after_at, l.src[1]);
            }
        }
        // a consonant with the maddah: a letter-name of the openings (الٓمٓ): six counts
        if l.maddah && l.vowel.is_none() && l.long_after.is_none() && !matches!(l.ch, 'ا' | 'و' | 'ي' | 'ى') && !l.dagger && consonant(l.ch).is_some() && !hamza_letter(l.ch) {
            push(&mut out, "madd_6", l.src[0], l.src[1]);
        }
        // līn in pause: wāw or yāʾ sākin after fatḥa before the last letter
        if pause && wi + 1 == nw && matches!(l.ch, 'و' | 'ي') && l.sukun && prev.is_some_and(|p| p.vowel == Some(V::A)) && next_said(k).is_some_and(|j| flat[j].0 == wi && last_in_word(j)) {
            push(&mut out, "madd_246", l.src[0], l.src[1]);
        }
        // ---- ghunna and qalqala
        // a doubled nūn or mīm holds its ghunna, unless a nūn, tanwīn or mīm before it merges into
        // it (then the idghām is the rule)
        let merged_into = k > 0 && {
            let p = at(k - 1);
            let pw = &words[flat[k - 1].0];
            p.tanwin.is_some() || matches!(p.ch, 'ن' | 'م') && (p.sukun || p.bare() && pw.vowelled()) || matches!(p.ch, 'ا' | 'ى') && p.bare() && k > 1 && at(k - 2).tanwin.is_some()
        };
        if matches!(l.ch, 'ن' | 'م') && l.shadda && !merged_into {
            push(&mut out, "ghunnah", l.src[0], l.src[1]);
        }
        let sakin = l.sukun || (l.bare() && vowelled && consonant(l.ch).is_some() && !matches!(l.ch, 'ا' | 'ى'));
        if qalqala_letter(l.ch) && (l.sukun || in_pause(k)) {
            push(&mut out, "qalqalah", l.src[0], l.src[1]);
        }
        // ---- nūn sākin, tanwīn, mīm sākin, and merging letters
        let pausing = in_pause(k);
        let next = next_said(k);
        if !pausing && let Some(j) = next {
            let n = at(j);
            let nun = l.ch == 'ن' && sakin && !l.shadda;
            if (nun || l.tanwin.is_some()) && n.ch != WASLA {
                let across = flat[j].0 != wi;
                // iẓhār before the throat letters; idghām before يرملون (with ghunna into ينمو, across
                // words only); iqlāb before ب; ikhfāʾ before the other fifteen
                let rule = match n.ch {
                    c if is_hamza_or_throat(c) => "",
                    'ي' | 'ن' | 'م' | 'و' if across => "idghaam_ghunnah",
                    'ي' | 'ن' | 'م' | 'و' => "",
                    'ل' | 'ر' => "idghaam_no_ghunnah",
                    'ب' => "iqlab",
                    c if consonant(c).is_some() => "ikhfa",
                    _ => "",
                };
                if !rule.is_empty() {
                    let start = if l.tanwin.is_some() { l.tanwin_at } else { l.src[0] };
                    push(&mut out, rule, start, n.src[1]);
                }
            } else if l.ch == 'م' && sakin && !l.shadda {
                if n.ch == 'ب' {
                    push(&mut out, "ikhfa_shafawi", l.src[0], n.src[1]);
                } else if n.ch == 'م' {
                    push(&mut out, "idghaam_shafawi", l.src[0], n.src[1]);
                }
            } else if l.bare() && vowelled && n.shadda && !article_lam(w, li) && let Some(rule) = merging_pair(l.ch, n.ch) {
                push(&mut out, rule, l.src[0], l.src[1]);
            }
        }
    }
    out
}

/// The tajweed annotations of each phrase of a text (as [`to_phrases`] splits it).
fn annotate(words: &[Word], pause: bool) -> Vec<Annotation> {
    annotate_words(words, pause)
}

/// The nasal a hidden nūn (ikhfāʾ) takes toward the next consonant: made at its place, held with
/// the velum open. (In ikhfāʾ the tongue only approaches that place; a full closure there is the
/// nearest this articulator set comes.)
fn hidden_nasal(next: &str) -> &'static str {
    match next.trim_end_matches('ˤ') {
        "k" | "g" => "ŋ",
        "q" => "ɴ",
        "f" => "ɱ",
        "ʃ" | "dʒ" | "j" => "ɲ",
        _ => "n",
    }
}

/// Tajweed applied to the sounds, from the annotations of the text (Ḥafṣ by way of al-Shāṭibiyya):
/// madd muttaṣil and munfaṣil 4 counts, lāzim 6, ʿāriḍ `o.arid`; a ghunna held 2 counts; idghām
/// with ghunna (the nūn merges into a doubled ي ن م و, nasalized), iqlāb (n → m), ikhfāʾ (a nasal
/// at the next letter's place), ikhfāʾ and idghām shafawī; like and near letters merged. (Qalqala
/// is annotated; its echo is not made yet, see below.)
fn recite(ph: &mut [Phone], ann: &[Annotation], o: &ArabicOptions) {
    let over = |p: &Phone, a: &Annotation| p.src[0] < a.end && a.start < p.src[1];
    let n = ph.len();
    for a in ann {
        // the first consonant in the span, and the first consonant after index i
        let first_cons = |ph: &[Phone], letter: Option<char>| (0..n).find(|&i| !ph[i].is_vowel() && !ph[i].drop && over(&ph[i], a) && letter.is_none_or(|c| ph[i].letter == c));
        let next_cons = |ph: &[Phone], i: usize| (i + 1..n).find(|&j| !ph[j].is_vowel() && !ph[j].drop);
        match a.rule {
            "madd_muttasil" | "madd_munfasil" | "madd_6" | "madd_246" => {
                let c = match a.rule {
                    "madd_6" => 6.0,
                    "madd_246" => o.arid,
                    _ => 4.0,
                };
                for p in ph.iter_mut().filter(|p| p.is_vowel() && p.long && over(p, a)) {
                    p.counts = Some(c);
                }
            }
            "ghunnah" => {
                if let Some(i) = first_cons(ph, None) {
                    ph[i].counts = Some(2.0);
                }
            }
            "idghaam_ghunnah" | "idghaam_shafawi" => {
                let letter = if a.rule == "idghaam_shafawi" { 'م' } else { 'ن' };
                if let Some(i) = first_cons(ph, Some(letter))
                    && let Some(j) = next_cons(ph, i)
                {
                    ph[i].drop = true;
                    let t = &mut ph[j];
                    t.gem = true;
                    t.counts = Some(2.0);
                    t.nasalized = matches!(t.ipa, "j" | "w");
                    t.rules.push(GHUNNA);
                }
            }
            "iqlab" | "ikhfa" => {
                if let Some(i) = first_cons(ph, Some('ن')) {
                    let next = next_cons(ph, i).map_or("n", |j| ph[j].ipa);
                    let p = &mut ph[i];
                    p.ipa = if a.rule == "iqlab" { "m" } else { hidden_nasal(next) };
                    p.counts = Some(2.0);
                    p.rules.push(GHUNNA);
                }
            }
            "ikhfa_shafawi" => {
                if let Some(i) = first_cons(ph, Some('م')) {
                    ph[i].counts = Some(2.0);
                    ph[i].rules.push(GHUNNA);
                }
            }
            "idghaam_mutajanisayn" | "idghaam_mutaqaribayn" => {
                if let Some(i) = first_cons(ph, None) {
                    ph[i].drop = true;
                }
            }
            // qalqala: the stop is released (as every stop before a pause is). Its echo, an added
            // short vowel even 15 ms long, made Whisper hear another syllable (أَحَدْ → "أحوال",
            // لَمْ يَلِدْ → garbage): a bounce in the release itself needs a better source model.
            _ => {}
        }
    }
}

/// Heaviness: the heavy letters, rāʾ by its vowels, the lām of Allāh by the vowel before it.
fn tafkhim(ph: &mut [Phone]) {
    let n = ph.len();
    for i in 0..n {
        if ph[i].is_vowel() {
            continue;
        }
        let next_vowel = if i + 1 < n && ph[i + 1].is_vowel() { Some(i + 1) } else { None };
        let heavy = if heavy_letter(ph[i].letter) {
            true
        } else if ph[i].letter == 'ر' {
            match next_vowel.map(|k| ph[k].vowel.unwrap()) {
                Some(V::A) | Some(V::U) => true,
                Some(V::I) => false,
                None => {
                    // a sākin rāʾ: by the vowel before it (past one sākin that isn't yāʾ)
                    let mut k = i;
                    let mut light = false;
                    let mut found = None;
                    while k > 0 {
                        k -= 1;
                        if ph[k].is_vowel() {
                            found = Some(k);
                            break;
                        }
                        if ph[k].ipa == "j" {
                            light = true;
                            break;
                        }
                    }
                    if light {
                        false
                    } else {
                        match found {
                            Some(k) if ph[k].vowel == Some(V::I) && !ph[k].wasl => {
                                // after kasra: light, unless a heavy letter follows in the word
                                ph[i + 1..].iter().take_while(|p| p.word == ph[i].word).find(|p| !p.is_vowel()).is_some_and(|p| heavy_letter(p.letter))
                            }
                            _ => true,
                        }
                    }
                }
            }
        } else if ph[i].allah {
            ph[..i].iter().rev().find(|p| p.is_vowel()).is_none_or(|p| p.vowel != Some(V::I))
        } else {
            continue;
        };
        if ph[i].letter == 'ر' || ph[i].allah {
            ph[i].heavy = heavy;
            ph[i].rules.push(if heavy { TAFKHIM } else { TARQIQ });
        } else {
            ph[i].rules.push(TAFKHIM);
        }
        if heavy && let Some(k) = next_vowel {
            ph[k].heavy = true;
            ph[k].rules.push(TAFKHIM);
        }
    }
}

/// Tajweed contexts that depend on the next sound: nūn and mīm sākin, qalqala, madd before
/// hamza across words and madd ʿāriḍ.
fn contexts(ph: &mut [Phone], pause: bool) {
    let n = ph.len();
    for i in 0..n {
        let next = ph.get(i + 1).map(|p| (p.is_vowel(), p.letter, p.word));
        let sakin = !ph[i].is_vowel() && next.is_none_or(|(v, _, _)| !v);
        // nūn and mīm sākin take their tajweed from the consonant after them
        if sakin && next.is_some() && ph[i].letter == 'ن' && !ph[i].gem && !ph[i].rules.contains(&NUN_SAKIN) {
            ph[i].rules.push(NUN_SAKIN);
        }
        if sakin && next.is_some() && ph[i].letter == 'م' && !ph[i].gem {
            ph[i].rules.push(MIM_SAKIN);
        }
        if sakin && qalqala_letter(ph[i].letter) {
            ph[i].rules.push(QALQALA);
        }
        if ph[i].long && ph[i].rules.contains(&MADD) {
            // a madd at a word end, the next word starting with a hamza
            if let Some((_, l, w)) = next
                && w != ph[i].word
                && hamza_letter(l)
            {
                ph[i].rules.push(MADD_MUNFASIL);
            }
            // in pause, a madd before the word's last (now sākin) consonant
            let last_word = ph[i].word == ph[n - 1].word;
            if pause && last_word && i + 2 == n && !ph[n - 1].is_vowel() {
                ph[i].rules.push(MADD_ARID);
            }
        }
    }
}

/// Stress per word: the last superheavy syllable, else a heavy penult, else the antepenult.
fn stress(ph: &[Phone], words: &[Word]) -> Vec<u8> {
    let mut out = vec![0u8; ph.len()];
    let mut i = 0;
    while i < ph.len() {
        let w = ph[i].word;
        let mut j = i;
        while j < ph.len() && ph[j].word == w {
            j += 1;
        }
        let vs: Vec<usize> = (i..j).filter(|&k| ph[k].is_vowel()).collect();
        if !vs.is_empty() && !PARTICLES.contains(&skeleton(&words[w]).as_str()) {
            // consonant slots after each vowel up to the next vowel (a doubled one counts twice)
            let weight = |n: usize| -> u8 {
                let k = vs[n];
                let end = if n + 1 < vs.len() { vs[n + 1] } else { j };
                let mut slots: usize = ph[k + 1..end].iter().map(|p| if p.gem { 2 } else { 1 }).sum();
                if n + 1 < vs.len() {
                    slots = slots.saturating_sub(1); // the next syllable's onset
                }
                let long = ph[k].long as usize;
                (long + slots).min(2) as u8
            };
            let s = vs.len();
            let pick = if s == 1 {
                0
            } else if weight(s - 1) >= 2 {
                s - 1
            } else if weight(s - 2) >= 1 || s == 2 {
                s - 2
            } else {
                s - 3
            };
            out[vs[pick]] = 1;
        }
        i = j;
    }
    out
}

/// Seconds a consonant lasts (its inventory length at a 0.1 s count, scaled with the tempo), doubled
/// ones longer: an obstruent 1.6 times (Arabic geminates are 1.5–2 times their singletons), a
/// nasal two counts.
fn consonant_dur(p: &Phone, count: f64) -> (f64, f64) {
    let base = phoneme(p.ipa, BodyMix::default()).map_or(0.08, |d| d.dur) * count / 0.1;
    if !p.gem {
        return (base, 1.0);
    }
    if matches!(p.ipa, "n" | "m") {
        // a doubled nasal is held two counts (its ghunna)
        return (2.0 * count, 2.0);
    }
    (1.6 * base, 2.0)
}

fn time_phrase(ph: Vec<Phone>, words: &[Word], raw_words: &[usize], o: &ArabicOptions, question: bool, guessed: usize) -> ArabicPhrase {
    // letters merged into the next (idghām) are not said
    let ph: Vec<Phone> = ph.into_iter().filter(|p| !p.drop).collect();
    // a recitation has no speech stress accents
    let st = if o.recite { vec![0; ph.len()] } else { stress(&ph, words) };
    let last_vowel = ph.iter().rposition(|p| p.is_vowel());
    let n = ph.len();
    let mut segments = Vec::with_capacity(n);
    let (mut sources, mut counts, mut rules) = (vec![], vec![], vec![]);
    // the reading in IPA, a stress mark before the stressed syllable's onset consonant
    let mut parts: Vec<String> = Vec::with_capacity(n);
    let mut out_words = vec![];
    for (i, p) in ph.iter().enumerate() {
        let word_end = i + 1 == n || ph[i + 1].word != p.word;
        let mut seg_ipa = p.ipa.to_string();
        // rāʾ is a tap between the vowels of a word and a short trill elsewhere (starting a word,
        // ending one, beside a consonant, doubled): a lone tap there is heard as a d
        let between_vowels = i > 0 && ph[i - 1].is_vowel() && ph[i - 1].word == p.word && i + 1 < n && ph[i + 1].is_vowel();
        let trill = p.ipa == "ɾ" && (p.gem || !between_vowels);
        if p.ipa == "ɾ" {
            seg_ipa = match (trill, p.heavy) {
                (true, true) => "rˤ",
                (true, false) => "r",
                (false, true) => "ɾˤ",
                (false, false) => "ɾ",
            }
            .to_string();
        } else if p.heavy && p.ipa == "l" {
            // the heavy lām of Allāh
            seg_ipa = "ɫ".to_string();
        }
        if p.heavy && p.is_vowel() {
            seg_ipa.push('ˤ');
        }
        let (dur, c) = if let Some(c) = p.counts {
            // a length tajweed sets (a madd of 4 or 6, a ghunna of 2)
            (c * o.count, c)
        } else if p.is_vowel() {
            let c = if p.long { 2.0 } else { 1.0 };
            // the phrase's last vowel lengthens (phrase-final lengthening), stressed ones a little
            let mut d = c * o.count;
            if Some(i) == last_vowel {
                d *= 1.3;
            } else if st[i] == 1 {
                d *= 1.1;
            }
            (d, c)
        } else if trill {
            // a short trill of two contacts, three when doubled: not a roll (tajweed's takrīr is
            // a quality to keep small)
            if p.gem { (1.1 * o.count, 2.0) } else { (0.75 * o.count, 1.0) }
        } else {
            consonant_dur(p, o.count)
        };
        segments.push(Segment { ipa: seg_ipa.clone(), dur: Some(dur), stress: Some(if p.is_vowel() { st[i] } else { 0 }), f0: None, word_end: if word_end { Some(true) } else { None }, nasalized: if p.nasalized { Some(true) } else { None } });
        sources.push(p.src);
        counts.push(c);
        rules.push(p.rules.clone());
        let mut part = seg_ipa.clone();
        if let Some(c) = p.counts.filter(|c| *c > 2.0) {
            // a lengthened madd: one more ː per two counts
            part.push_str(&"ː".repeat((c / 2.0).round() as usize));
        } else if p.long || p.gem || p.counts.is_some() {
            part.push('ː');
        }

        if st[i] == 1 {
            let onset = i > 0 && !ph[i - 1].is_vowel() && ph[i - 1].word == p.word;
            let k = if onset { parts.len() - 1 } else { parts.len() };
            if k < parts.len() {
                parts[k].insert(0, 'ˈ');
            } else {
                part.insert(0, 'ˈ');
            }
        }
        if word_end {
            part.push(' ');
            out_words.push(words[raw_words[p.word]].text.clone());
        }
        parts.push(part);
    }
    let ipa = parts.concat().trim_end().to_string();
    // recited, the āya drifts down and falls at its end (a level tone lost words to Whisper:
    // قُلْ هُوَ → "قره"); spoken, a question rises
    let intonation = if question && !o.recite { Intonation::Question } else { Intonation::Statement };
    ArabicPhrase { segments, intonation, pause: if o.recite { 0.6 } else { 0.35 }, words: out_words, sources, counts, rules, ipa, guessed, annotations: vec![] }
}

/// Arabic text → phrases of segments (see the module docs).
pub fn to_phrases(text: &str, o: &ArabicOptions) -> Vec<ArabicPhrase> {
    let mut out = vec![];
    for rp in parse(text) {
        let mut words = rp.words;
        for w in words.iter_mut() {
            repair(w);
        }
        let mut r = Reader { phones: vec![], guessed: 0, words: &words };
        let nw = words.len();
        for wi in 0..nw {
            r.read_word(wi, o.waqf && wi + 1 == nw);
        }
        let mut ph = r.phones;
        if ph.is_empty() {
            continue;
        }
        idgham_without_ghunna(&mut ph);
        tafkhim(&mut ph);
        contexts(&mut ph, o.waqf);
        let annotations = annotate(&words, o.waqf);
        if o.recite {
            recite(&mut ph, &annotations, o);
        }
        let idx: Vec<usize> = (0..nw).collect();
        let mut phrase = time_phrase(ph, &words, &idx, o, rp.question, r.guessed);
        phrase.annotations = annotations;
        out.push(phrase);
    }
    out
}

/// The reading of a text as IPA, phrases joined by " | " (for tests and the CLI).
pub fn to_ipa(text: &str, o: &ArabicOptions) -> String {
    to_phrases(text, o).iter().map(|p| p.ipa.clone()).collect::<Vec<_>>().join(" | ")
}
