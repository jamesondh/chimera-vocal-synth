//! Polish text → phoneme segments. Polish spelling is close to phonemic, so letters map to sounds
//! by rule and no dictionary is needed (standard Warsaw pronunciation):
//! - Digraphs are one sound: sz [ʃ], cz [tʃ], rz and ż [ʒ], dż [dʒ], ch and h [x], dz [dz]. The
//!   sz series is the inventory's postalveolars, not its retroflexes: [tʂ] never quite closes (its
//!   closure stayed ~13 dB louder than [tʃ]'s) and its release is 8 dB weaker, [ʂ] hisses 8 dB
//!   below [ʃ] and [ʐ] 6 dB below [ʒ], so they sounded mushy and weak; Polish sz is a flat
//!   postalveolar, not a curled one, anyway.
//! - i before a vowel only softens the consonant before it: si, zi, ci, dzi, ni are ś ź ć dź ń
//!   ("się" [ɕɛ], "dzień" [dʑɛɲ]); labials and l are palatalized ("pies" [pʲɛs], "miasto"); k and
//!   g front to [c ɟ] ("kiedy" [cɛdɨ]); t, d and r, in loans, take a [j] ("radio" [radjɔ]). Before
//!   a consonant or at the end, i is a vowel and still softens ("nic" [ɲits]).
//! - ę and ą are a vowel and a nasal of the next consonant's place before a stop or affricate
//!   ("ręka" [rɛŋka], "ząb" [zɔmp], "pięć" [pʲɛɲtɕ]), oral before l, ł and a nasal ("wziął"),
//!   nasalized before a fricative and at the end ("mąż", "są"); a final ę is oral ("się").
//! - Voicing: obstruents devoice at a word's end ("chleb" [xlɛp]) and take the voicing of the
//!   obstruent after them ("wódka" [vutka], "prośba" [prɔʑba]); w and rz devoice after a voiceless
//!   one and don't voice what is before them ("kwiat" [kfʲat], "przy" [pʃɨ], "twój" [tfuj]).
//! - A doubled consonant letter is one long consonant ("Anna", "lekki").
//! - A word with no vowel ("w", "z") is said with the next word and assimilates to it
//!   ("w Polsce" [fpɔlstsɛ]).
//!
//! Stress falls on the penult, except: the antepenult in past-tense -liśmy/-łyście ("byliśmy"),
//! the conditional -łbym/-łaby ("zrobiłbym"; -libyśmy one further back), Greek and Latin loans in
//! -yka/-ika ("matematyka", "muzyka") and a short list ("uniwersytet"). Prepositions, pronoun
//! clitics and conjunctions of one syllable take none ("się", "na", "że").
//!
//! Approximations: r is one tap, as Russian's (the modelled trill held the tip near the ridge and
//! the voicing all but stopped). After t or d, whose closure is at the tap's place, the tap merged
//! into the stop and the voice came back only in the vowel ("tramwaj" sounded like "tamwaj"), so
//! in speech a 50 ms [ə] goes between them, the brief vowel Polish speakers make there; a nasal vowel is the oral vowel with the velum down, without the
//! [w̃] glide some speakers add. Unstressed vowels don't reduce in Polish, so there are no weak
//! forms. Numbers are not read out yet.

use super::english::{Phrase, is_phrase_punct, split_keeping};
use super::mandarin::MandarinLexicon;
use super::syllables::{FrontEnd, Lang, lyrics_in};
use super::{Intonation, Segment, Syllable};

/// A doubled consonant lasts this many times what the duration rules give it (as Turkish's).
const GEMINATE: f64 = 1.6;
/// Klatt's /ɨ/ is English's reduced vowel (roses, 110 ms); Polish y is a full vowel, about as long
/// as /ɪ/ (135 ms).
const Y_LENGTH: f64 = 1.25;
/// The vowel between t or d and a tap, in speech (s): at 30 ms the folds, still apart after the
/// stop, didn't voice it; at 40-50 ms the r is heard (a native listener chose 50 over 40).
const TAP_LEAD: f64 = 0.05;
/// Vowels (but y) last this many times what the duration rules give them, in speech. The rules
/// are English's: they stretch a stressed vowel, and the last one of a phrase most. A Polish
/// speaker's vowels were shorter, and stress lengthened them less (1.4 times, the rules' 1.76),
/// while the consonants matched
/// (0.84-0.98 of the rules' per phrase). Measured on one native speaker, four sentences (the
/// Speak examples and one of sz/cz/rz), aligned with MFA's polish_mfa: stressed 88 ms against the
/// rules' 144 (11 vowels), unstressed 63 against 82 (8), a phrase's last 107 against 198 (5).
const VOWEL_STRESSED: f64 = 0.6;
const VOWEL_UNSTRESSED: f64 = 0.75;
/// A phrase's last vowel, which [`to_phrases`] sets.
const VOWEL_FINAL: f64 = 0.55;

/// A vowel the length factors above apply to (y keeps [`Y_LENGTH`]).
fn shortened(ipa: &str) -> bool {
    matches!(ipa, "a" | "ɛ" | "i" | "ɔ" | "u")
}

pub fn is_letter(c: char) -> bool {
    c.is_ascii_alphabetic() || "ąćęłńóśźżĄĆĘŁŃÓŚŹŻ".contains(c)
}

fn is_vowel_letter(c: char) -> bool {
    "aeiouyąęó".contains(c)
}

fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// false for "w", "z": a word that is said with the next one
pub fn has_vowel(s: &str) -> bool {
    s.chars().map(lower).any(is_vowel_letter)
}

/// One sound of a word and the letter it is written with.
#[derive(Clone, Debug)]
struct Phone {
    ipa: &'static str,
    letter: usize,
    vowel: bool,
    /// ę or ą (before `resolve_nasals`), then: a nasalized vowel
    nasal: bool,
    palatal: bool,
    /// a doubled consonant letter
    long: bool,
    /// w or rz: devoiced after a voiceless obstruent, and doesn't voice one before it
    follows: bool,
}

impl Phone {
    fn new(ipa: &'static str, letter: usize) -> Self {
        Phone { ipa, letter, vowel: false, nasal: false, palatal: false, long: false, follows: false }
    }
}

/// What a following i does to a consonant.
#[derive(Clone, Copy)]
enum Soft {
    /// nothing (sz, cz, ż, ł, j, ...)
    None,
    /// becomes this sound (s → ɕ, n → ɲ, k → c)
    To(&'static str),
    /// palatalized (p b m f w l ch)
    Palatal,
    /// a [j] after it before a vowel (t d r, in loans)
    Glide,
}

/// The consonant grapheme at `i`: its sound, its letters, and what an i after it does.
fn grapheme(w: &[char], i: usize, split_rz: bool) -> (&'static str, usize, Soft) {
    let next = w.get(i + 1).copied();
    match (w[i], next) {
        ('d', Some('ź')) => ("dʑ", 2, Soft::None),
        ('d', Some('ż')) => ("dʒ", 2, Soft::None),
        ('d', Some('z')) => ("dz", 2, Soft::To("dʑ")),
        ('d', _) => ("d", 1, Soft::Glide),
        ('c', Some('z')) => ("tʃ", 2, Soft::None),
        ('c', Some('h')) => ("x", 2, Soft::Palatal),
        ('c', _) => ("ts", 1, Soft::To("tɕ")),
        ('s', Some('z')) => ("ʃ", 2, Soft::None),
        ('s', _) => ("s", 1, Soft::To("ɕ")),
        ('r', Some('z')) if !split_rz => ("ʒ", 2, Soft::None),
        ('r', _) => ("ɾ", 1, Soft::Glide),
        ('z', _) => ("z", 1, Soft::To("ʑ")),
        ('n', _) => ("n", 1, Soft::To("ɲ")),
        ('k' | 'q', _) => ("k", 1, Soft::To("c")),
        ('g', _) => ("g", 1, Soft::To("ɟ")),
        ('t', _) => ("t", 1, Soft::Glide),
        ('h', _) => ("x", 1, Soft::Palatal),
        ('p', _) => ("p", 1, Soft::Palatal),
        ('b', _) => ("b", 1, Soft::Palatal),
        ('m', _) => ("m", 1, Soft::Palatal),
        ('f', _) => ("f", 1, Soft::Palatal),
        ('w' | 'v', _) => ("v", 1, Soft::Palatal),
        ('l', _) => ("l", 1, Soft::Palatal),
        ('ż', _) => ("ʒ", 1, Soft::None),
        ('ź', _) => ("ʑ", 1, Soft::None),
        ('ś', _) => ("ɕ", 1, Soft::None),
        ('ć', _) => ("tɕ", 1, Soft::None),
        ('ń', _) => ("ɲ", 1, Soft::None),
        ('ł', _) => ("w", 1, Soft::None),
        ('j', _) => ("j", 1, Soft::None),
        _ => ("", 1, Soft::None),
    }
}

/// Words where rz is r and z, not [ʒ] ("marznąć", "zamarzać").
const RZ_SPLIT: &[&str] = &["marzn", "marzł", "zamarza", "tarzan"];

/// The indices of the r's in `w` that are r and z, not rz: only the rz inside a listed stem, so
/// "przemarznąć" keeps prze- [pʃɛ].
fn rz_splits(w: &[char]) -> Vec<usize> {
    let mut out = Vec::new();
    for stem in RZ_SPLIT {
        let s: Vec<char> = stem.chars().collect();
        let r = s.windows(2).position(|x| x == ['r', 'z']).unwrap_or(0);
        for (k, win) in w.windows(s.len()).enumerate() {
            if win == s.as_slice() {
                out.push(k + r);
            }
        }
    }
    out
}

fn letters_to_phones(w: &[char]) -> Vec<Phone> {
    let splits = rz_splits(w);
    let mut out = Vec::with_capacity(w.len() + 2);
    let mut long = false;
    let mut i = 0;
    while i < w.len() {
        let c = w[i];
        if is_vowel_letter(c) {
            let ipa = match c {
                'a' => "a",
                'e' | 'ę' => "ɛ",
                'i' => "i",
                'o' | 'ą' => "ɔ",
                'y' => "ɨ",
                _ => "u", // u, ó
            };
            out.push(Phone { vowel: true, nasal: c == 'ę' || c == 'ą', ..Phone::new(ipa, i) });
            // au and eu starting a word are a falling diphthong in loans ("auto", "Europa");
            // elsewhere they are two vowels ("nauka")
            if i == 0 && (c == 'a' || c == 'e') && w.get(1) == Some(&'u') {
                out.push(Phone::new("w", 1));
                i += 1;
            }
            i += 1;
            continue;
        }
        if c == 'x' {
            out.push(Phone::new("k", i));
            out.push(Phone::new("s", i));
            i += 1;
            continue;
        }
        // a doubled consonant letter: the second one, long
        if w.get(i + 1) == Some(&c) {
            long = true;
            i += 1;
            continue;
        }
        let (ipa, len, soft) = grapheme(w, i, splits.contains(&i));
        if ipa.is_empty() {
            i += 1;
            continue;
        }
        let mut p = Phone { long, follows: c == 'w' || (ipa == "ʒ" && c == 'r'), ..Phone::new(ipa, i) };
        long = false;
        let mut j = i + len;
        let softened = w.get(j) == Some(&'i');
        // an i before another vowel is only the softening; elsewhere it is also a vowel. After
        // sz, cz, ż... it is always a vowel.
        let marker = softened && !matches!(soft, Soft::None) && w.get(j + 1).is_some_and(|&n| is_vowel_letter(n));
        let mut glide = None;
        if softened {
            match soft {
                Soft::To(s) => p.ipa = s,
                Soft::Palatal => p.palatal = true,
                Soft::Glide if marker => glide = Some(Phone::new("j", j)),
                Soft::Glide | Soft::None => {}
            }
        }
        if marker {
            j += 1;
        }
        out.push(p);
        out.extend(glide);
        i = j;
    }
    resolve_nasals(&mut out);
    out
}

/// ę and ą by what follows (module doc).
fn resolve_nasals(p: &mut Vec<Phone>) {
    let mut k = 0;
    while k < p.len() {
        if p[k].vowel && p[k].nasal {
            let next = p.get(k + 1).map(|x| x.ipa);
            let nasal = match next {
                Some("p" | "b") => Some("m"),
                Some("t" | "d" | "ts" | "dz" | "tʃ" | "dʒ") => Some("n"),
                Some("tɕ" | "dʑ") => Some("ɲ"),
                Some("k" | "g" | "c" | "ɟ") => Some("ŋ"),
                _ => None,
            };
            let final_e = next.is_none() && p[k].ipa == "ɛ";
            p[k].nasal = !(nasal.is_some() || final_e || matches!(next, Some("l" | "w" | "m" | "n" | "ɲ")));
            if let Some(n) = nasal {
                p.insert(k + 1, Phone::new(n, p[k].letter));
                k += 1;
            }
        }
        k += 1;
    }
}

fn devoiced(s: &str) -> Option<&'static str> {
    Some(match s {
        "b" => "p",
        "d" => "t",
        "g" => "k",
        "ɟ" => "c",
        "v" => "f",
        "z" => "s",
        "ʒ" => "ʃ",
        "ʑ" => "ɕ",
        "dz" => "ts",
        "dʒ" => "tʃ",
        "dʑ" => "tɕ",
        "ɣ" => "x",
        _ => return None,
    })
}

fn voiced(s: &str) -> Option<&'static str> {
    Some(match s {
        "p" => "b",
        "t" => "d",
        "k" => "g",
        "c" => "ɟ",
        "f" => "v",
        "s" => "z",
        "ʃ" => "ʒ",
        "ɕ" => "ʑ",
        "ts" => "dz",
        "tʃ" => "dʒ",
        "tɕ" => "dʑ",
        "x" => "ɣ",
        _ => return None,
    })
}

/// a voiceless obstruent (one with a voiced partner)
fn is_voiceless(s: &str) -> bool {
    voiced(s).is_some()
}

/// a voiced obstruent (one with a voiceless partner)
fn is_voiced(s: &str) -> bool {
    devoiced(s).is_some()
}

/// Voicing through a word, with the vowelless word before it (module doc): w and rz devoice after
/// a voiceless obstruent, then each obstruent takes the voicing of the one after it, and the last
/// devoices, also before a word-final sonorant (bóbr [bupɾ], jaźń [jaɕɲ]).
fn assimilate(p: &mut [Phone]) {
    for k in 1..p.len() {
        if p[k].follows
            && is_voiceless(p[k - 1].ipa)
            && let Some(d) = devoiced(p[k].ipa)
        {
            p[k].ipa = d;
        }
    }
    for k in (0..p.len()).rev() {
        let r = match p.get(k + 1) {
            _ if p[k + 1..].iter().all(|n| is_sonorant(n.ipa)) => devoiced(p[k].ipa),
            Some(n) if is_voiceless(n.ipa) => devoiced(p[k].ipa),
            Some(n) if is_voiced(n.ipa) && !n.follows => voiced(p[k].ipa),
            _ => None,
        };
        if let Some(r) = r {
            p[k].ipa = r;
        }
    }
}

// ---------------------------------------------------------------- stress

/// Words of one syllable that lean on the next or the previous word and take no stress:
/// prepositions, pronoun clitics, conjunctions and particles.
#[rustfmt::skip]
const CLITICS: &[&str] = &[
    "się", "mi", "ci", "go", "mu", "cię", "by", "bym", "byś", "że", "i", "a", "o", "u", "w", "z", "we", "ze",
    "na", "do", "od", "po", "za", "dla", "bez", "pod", "nad", "przy", "przez", "przed", "ku", "czy",
];

/// Words stressed on the antepenult.
#[rustfmt::skip]
const ANTEPENULT: &[&str] = &[
    "uniwersytet", "opera", "rzeczpospolita", "czterysta", "siedemset", "osiemset", "dziewięćset",
];

/// Loans in -yka/-ika and their cases (-ki, -kę, -ką, -ce), stressed on the antepenult: these
/// stems before one of those endings ("matematyka", "muzyce").
#[rustfmt::skip]
const YKA_STEMS: &[&str] = &[
    "matematy", "gramaty", "polity", "fizy", "muzy", "techni", "republi", "prakty", "kryty",
    "informaty", "logi", "afry", "amery", "mechani", "botani", "klini", "pani", "grafi", "fonety",
];
const YKA_ENDINGS: &[&str] = &["ka", "ki", "kę", "ką", "ce"];

/// The l-participle a conditional or past-tense ending sits on ("zrobił-", "byli-").
fn l_participle(stem: &str) -> bool {
    stem.ends_with('ł') || ["ła", "ło", "li", "ły"].iter().any(|e| stem.ends_with(e))
}

/// The stressed vowel (0 = the first), or None for a clitic or a word without vowels.
fn stressed_vowel(word: &str, vowels: usize) -> Option<usize> {
    if vowels == 0 || CLITICS.contains(&word) {
        return None;
    }
    let back = |k: usize| Some(vowels.saturating_sub(k));
    if ANTEPENULT.contains(&word) || YKA_ENDINGS.iter().any(|e| word.strip_suffix(e).is_some_and(|s| YKA_STEMS.contains(&s))) {
        return back(3);
    }
    let stem = |ends: &[&str]| ends.iter().find_map(|e| word.strip_suffix(e)).is_some_and(l_participle);
    if stem(&["byśmy", "byście"]) {
        return back(4);
    }
    if stem(&["bym", "byś", "by"]) || stem(&["śmy", "ście"]) {
        return back(3);
    }
    back(2)
}

// ---------------------------------------------------------------- words

/// A word read: its letters as typed, its sounds after voicing, and its stressed vowel. `before`
/// is a word with no vowel said with it ("w", "z"): its sounds come first.
struct Parsed {
    typed: Vec<char>,
    phones: Vec<Phone>,
    stressed: Option<usize>,
}

fn parse(typed: &str, before: Option<&str>) -> Parsed {
    let kept: Vec<char> = typed.chars().filter(|c| is_letter(*c)).collect();
    let letters: Vec<char> = kept.iter().map(|c| lower(*c)).collect();
    let mut phones: Vec<Phone> = match before {
        Some(b) => {
            let b: Vec<char> = b.chars().filter(|c| is_letter(*c)).map(lower).collect();
            // its letters aren't this word's: they all stand before its first letter
            letters_to_phones(&b).into_iter().map(|p| Phone { letter: 0, ..p }).collect()
        }
        None => Vec::new(),
    };
    phones.extend(letters_to_phones(&letters));
    assimilate(&mut phones);
    let vowels = phones.iter().filter(|p| p.vowel).count();
    let word: String = letters.iter().collect();
    Parsed { typed: kept, phones, stressed: stressed_vowel(&word, vowels) }
}

impl Parsed {
    /// Each sound's segment, one per phone. `speech` gives long sounds and vowels their length (a
    /// sung note sets its own) and puts [ə] between t or d and a tap ([`TAP_LEAD`]).
    fn segments(&self, speech: bool) -> Vec<Segment> {
        let mut nv = 0;
        let mut out = Vec::with_capacity(self.phones.len() + 1);
        for (i, p) in self.phones.iter().enumerate() {
            let stressed = p.vowel && Some(nv) == self.stressed;
            nv += usize::from(p.vowel);
            let length = if !speech {
                None
            } else if p.long {
                Some(GEMINATE)
            } else if p.ipa == "ɨ" {
                Some(Y_LENGTH)
            } else if shortened(p.ipa) {
                Some(if stressed { VOWEL_STRESSED } else { VOWEL_UNSTRESSED })
            } else {
                None
            };
            if speech && p.ipa == "ɾ" && i > 0 && matches!(self.phones[i - 1].ipa, "t" | "d") {
                out.push(Segment { ipa: "ə".into(), dur: Some(TAP_LEAD), stress: Some(0), ..Default::default() });
            }
            out.push(Segment {
                ipa: p.ipa.to_string(),
                stress: Some(u8::from(stressed)),
                nasalized: p.nasal.then_some(true),
                palatalized: p.palatal.then_some(true),
                length,
                ..Default::default()
            });
        }
        out
    }
}

/// One word, as typed, to segments with stress.
pub fn word(typed: &str) -> Vec<Segment> {
    parse(typed, None).segments(true)
}

/// One word with the vowelless word said before it ("w", "z"), or none.
pub fn word_after(typed: &str, before: Option<&str>) -> Vec<Segment> {
    parse(typed, before).segments(true)
}

fn is_sonorant(ipa: &str) -> bool {
    matches!(ipa, "ɾ" | "l" | "w" | "m" | "n" | "ɲ" | "ŋ" | "j")
}

/// One word's sung syllables. Polish singing keeps syllables open: the next syllable takes the
/// consonants between two vowels, except that a sonorant before another consonant stays with the
/// vowel before it ("Pol-ska", "ręn-ka"), and a long consonant is sung on both sides ("An-na").
/// `before`: a vowelless word sung with this one, at the start of its first syllable ("w do-mu").
/// `cuts` are letter indices where the singer's hyphens start a syllable; between two vowels with
/// no cut, the usual rule decides.
fn sung_syllables(typed: &str, before: Option<&str>, cuts: &[usize]) -> Vec<Syllable> {
    let p = parse(typed, before);
    let segs = p.segments(false);
    let vi: Vec<usize> = p.phones.iter().enumerate().filter(|(_, x)| x.vowel).map(|(i, _)| i).collect();
    if vi.is_empty() {
        return Vec::new();
    }
    // where each syllable starts (a phone index), and a long consonant's copy closing the one before
    let mut starts = vec![0];
    let mut doubled = vec![None; vi.len()];
    for (k, w) in vi.windows(2).enumerate() {
        let run = &p.phones[w[0] + 1..w[1]];
        let (from, to) = (p.phones[w[0]].letter, p.phones[w[1]].letter);
        let cut = if let Some(&c) = cuts.iter().find(|&&c| from < c && c <= to) {
            let g = run.iter().position(|x| x.letter >= c).unwrap_or(run.len());
            if run.get(g).is_some_and(|x| x.long) {
                doubled[k] = Some(w[0] + 1 + g);
            }
            g
        } else if let Some(g) = run.iter().position(|x| x.long) {
            doubled[k] = Some(w[0] + 1 + g);
            g
        } else if run.len() >= 2 && is_sonorant(run[0].ipa) {
            1
        } else {
            0
        };
        starts.push(w[0] + 1 + cut);
    }
    let letter = |k: usize| if k == 0 { 0 } else { p.phones[starts[k]].letter };
    (0..vi.len())
        .map(|k| {
            let end = starts.get(k + 1).copied().unwrap_or(p.phones.len());
            let mut coda = segs[vi[k] + 1..end].to_vec();
            if let Some(d) = doubled[k] {
                coda.push(segs[d].clone());
            }
            let a = letter(k).min(p.typed.len());
            let b = if k + 1 < vi.len() { letter(k + 1).max(a).min(p.typed.len()) } else { p.typed.len() };
            let mut text: String = p.typed[a..b].iter().collect();
            if let (0, Some(w)) = (k, before) {
                text = format!("{w} {text}");
            }
            Syllable { text, onset: segs[starts[k]..vi[k]].to_vec(), nucleus: segs[vi[k]].clone(), coda, ..Default::default() }
        })
        .collect()
}

/// The Polish front end for the lyrics router ([`super::syllables::read_lyrics`]). Polish is
/// written in Latin letters, so the router can't tell it from English by script: the player
/// chooses it ([`Lang`]). Typed hyphens split a word your way, as in English.
pub struct Polish;

impl FrontEnd for Polish {
    fn starts(&self, c: char) -> bool {
        is_letter(c)
    }
    fn continues(&self, c: char) -> bool {
        is_letter(c) || c == '-'
    }
    fn word(&self, tok: &str) -> Vec<Syllable> {
        self.word_after(tok, None)
    }
    /// A word with no vowel before this one ("w", "z") has no note of its own (it returned no
    /// syllables): it starts this word's first one, voiced by this word.
    fn word_after(&self, tok: &str, prev: Option<&str>) -> Vec<Syllable> {
        // hyphens only place the breaks: the word is read whole, so "rę-ka" keeps ręka's [ŋ] and
        // "proś-ba" its [ʑ]
        let mut cuts = Vec::new();
        let mut letters = 0;
        for c in tok.chars() {
            if is_letter(c) {
                letters += 1;
            } else if c == '-' && letters > 0 {
                cuts.push(letters);
            }
        }
        sung_syllables(tok, prev.filter(|p| !has_vowel(p)), &cuts)
    }
}

/// Lyrics → sung syllables with Latin script read as Polish ([`Lang::Pl`]); other scripts are
/// routed as usual, and no dictionary is needed.
pub fn lyrics_to_syllables(text: &str) -> Vec<Syllable> {
    lyrics_in(text, &|_: &str, _: bool| Vec::<Segment>::new(), &MandarinLexicon::new(), Lang::Pl)
}

/// Question words: a question that starts with one falls, as a statement does; any other
/// question (a yes/no question, "czy ...?") rises.
#[rustfmt::skip]
const QUESTION_WORDS: &[&str] = &[
    "co", "czego", "czemu", "czym", "kto", "kogo", "komu", "kim", "gdzie", "kiedy", "jak", "jaki",
    "jaka", "jakie", "jakim", "dlaczego", "ile", "który", "która", "które", "skąd", "dokąd", "czyj",
];

/// Conjunctions that can come before a question word.
const LEADING: &[&str] = &["a", "i", "no", "ale", "więc", "to"];

/// Polish text → phrases of segments. Phrase punctuation breaks phrases as in English; a word
/// with no vowel is said with the next and shown with it ("w domu").
pub fn to_phrases(text: &str) -> Vec<Phrase> {
    let mut phrases = Vec::new();
    let mut words: Vec<String> = Vec::new();
    let mut flush = |delim: &str, words: &mut Vec<String>| {
        let mut segs: Vec<Segment> = Vec::new();
        let mut said: Vec<String> = Vec::new();
        let mut before: Option<String> = None;
        let n = words.len();
        for (k, w) in words.drain(..).enumerate() {
            if !has_vowel(&w) && k + 1 < n {
                before = Some(w);
                continue;
            }
            let b = before.take();
            let mut ws = word_after(&w, b.as_deref());
            let Some(last) = ws.last_mut() else { continue };
            last.word_end = Some(true);
            segs.append(&mut ws);
            let w: String = w.chars().map(lower).collect();
            said.push(match b {
                Some(b) => format!("{} {w}", b.to_lowercase()),
                None => w,
            });
        }
        if segs.is_empty() {
            return;
        }
        if let Some(v) = segs.iter_mut().rev().find(|s| s.ipa == "ɨ" || shortened(&s.ipa))
            && shortened(&v.ipa)
        {
            v.length = Some(VOWEL_FINAL);
        }
        // a conjunction before the question word doesn't count ("A gdzie jesteś?")
        let first = said.iter().map(|w| w.rsplit(' ').next().unwrap_or(w)).find(|w| !LEADING.contains(w));
        let question = delim.contains('?') && !first.is_some_and(|w| QUESTION_WORDS.contains(&w));
        let pause = if delim.contains(['.', '!', '?']) {
            0.35
        } else if !delim.is_empty() {
            0.18
        } else {
            0.3
        };
        phrases.push(Phrase { segments: segs, intonation: if question { Intonation::Question } else { Intonation::Statement }, pause, words: said });
    };
    for part in split_keeping(text, is_phrase_punct) {
        if !part.is_empty() && part.chars().all(is_phrase_punct) {
            flush(&part, &mut words);
        } else {
            words.extend(part.split(|c: char| !is_letter(c)).filter(|w| !w.is_empty()).map(str::to_string));
        }
    }
    flush("", &mut words);
    phrases
}
