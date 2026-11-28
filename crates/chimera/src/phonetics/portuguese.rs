//! Portuguese (Brazil) text → phoneme segments. Brazilian Portuguese is written close to
//! phonemic, but the spelling carries the stress, so the rules below read the letters *and* the
//! accents:
//!
//! - **the three r's**: [ʁ] in the onset (at the start of a word, after n or l) and in the coda
//!   (before a consonant or at the end of a word), [ɾ] between vowels: "Português" [poʁtuˈɡeʃ],
//!   "carioca" [kaɾiˈoka], "mar" [maʁ].
//! - **the s**: [z] between vowels, [s] everywhere else, the word-final s included: "asa"
//!   [ˈazɐ], "casas" [ˈkazɐs].
//! - **the v**: [v] everywhere: "nova" [ˈnovɐ], "você" [voˈsɛ].
//! - **the l**: [w] in the coda, before a consonant or at the end of a word after a vowel: "alto"
//!   [ˈawtu], "calma" [ˈkawmɐ]. Before a vowel it is [l]: "falar" [faˈlaʁ].
//! - **the t and the d**: [tʃ] and [dʒ] before [i] — the letter "i" and the word-final
//!   unstressed "e" ("noite" [ˈnojtʃi], "tarde" [ˈtaʁdʒi]); before a sounded [e ɛ] they
//!   stay [t] and [d]: "tempo" [ˈtẽpu], "tela" [ˈtɛlɐ].
//! - **the nasal diphthongs**: ão, ãi, õe, õi are one nucleus: "pão" [pɐ̃w̃], "mãe" [mɐ̃j̃].
//! - **the plain diphthongs** are one nucleus as well, with a glide: "pai" [ˈpaj], "noite"
//!   [ˈnojtʃi], "meu" [ˈmew].
//! - **the unstressed vowels**: an unstressed /a e o/ in a closed syllable, and a word-final one,
//!   are the near-open central [ɐ] and the open mid [ɛ ɔ]: "casa" [ˈkazɐ], "mesa" [ˈmɛzɐ].
//! - **the word-final unstressed e**: [i], not [ɛ]: "fome" [ˈfomi], "dente" [ˈdẽntʃi].
//!
//! Stress is written, which makes this front end's job easier than English's: a written accent
//! wins; a word ending in a nasal vowel falls on that vowel; the endings -ei, -ou and -mente
//! carry their own stress; a word ending in a closed syllable (l, z, i, u, im, um, ns) falls on
//! it; and everything else falls on the penult: "café" [kaˈfe], "casa" [ˈkazɐ], "sapato"
//! [saˈpatu], "Brasil" [bɾaˈziw], "rapidamente" [ʁapidaˈmɛ̃ntʃi].
//!
//! The nasal vowels need no new phoneme: the engine nasalises a vowel with a combining tilde
//! (`ipa.rs`), so /ã/ is a rule and not a sound.

use super::english::{Phrase, is_phrase_punct, split_keeping};
use super::mandarin::MandarinLexicon;
use super::syllables::{FrontEnd, Lang, lyrics_in};
use super::{Intonation, Segment, Syllable};

/// The combining tilde the engine reads as "this vowel is nasalised".
const NASAL: char = '\u{0303}';

/// The vowel letters, with every accent the spelling uses.
const VOWELS: &str = "aeiouáéíóúâêôãõẽĩũàèìòùâêô";

fn is_vowel_letter(c: char) -> bool {
    VOWELS.contains(c)
}

/// The vowel letters that are nasal (the tilde on them).
fn is_nasal_letter(c: char) -> bool {
    "ãõẽĩũ".contains(c)
}

/// The vowel letters that carry an accent (the written stress mark).
fn is_accent_letter(c: char) -> bool {
    "áéíóúâêôàèìòù".contains(c)
}

/// The plain vowel under an accent or a tilde.
fn base_vowel(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ã' => 'a',
        'é' | 'è' | 'ê' | 'ẽ' => 'e',
        'í' | 'ì' | 'î' | 'ĩ' => 'i',
        'ó' | 'ò' | 'ô' | 'õ' => 'o',
        'ú' | 'ù' | 'û' | 'ũ' => 'u',
        c => c,
    }
}

fn is_letter(c: char) -> bool {
    c.is_ascii_alphabetic() || VOWELS.contains(c) || "çñ".contains(c)
}

/// One sound of a word.
struct Phone {
    /// the sound in IPA; a nucleus may hold a glide ("ɐ̃w"), a consonant is one symbol
    ipa: String,
    /// the nucleus's ordinal (None for a consonant)
    nucleus: Option<usize>,
    /// the letter the sound starts at
    letter: usize,
    /// the letter after the sound (exclusive)
    end: usize,
    /// the nucleus is nasalised
    nasal: bool,
    /// the nucleus carries a written accent
    accented: bool,
}

impl Phone {
    fn consonant(ipa: &str, letter: usize, end: usize) -> Phone {
        Phone { ipa: ipa.to_string(), nucleus: None, letter, end, nasal: false, accented: false }
    }
    fn nucleus(ipa: String, n: usize, letter: usize, end: usize, nasal: bool, accented: bool) -> Phone {
        Phone { ipa, nucleus: Some(n), letter, end, nasal, accented }
    }
}

/// Whether the letter at `i` is a vowel of the word (for the consonant rules).
fn vowel_at(w: &[char], i: usize) -> bool {
    w.get(i).is_some_and(|c| is_vowel_letter(*c))
}

/// Whether the letter at `i` is a front vowel (e or i, with or without an accent): the consonants
/// that change before them change before the accented ones too ("você" has a [s] before ê).
fn front_vowel_at(w: &[char], i: usize) -> bool {
    w.get(i).is_some_and(|c| matches!(base_vowel(*c), 'e' | 'i'))
}

/// Whether the letter at `i` is the [i] that the palatal affricates change before: the letter
/// "i", or a word-final unstressed "e" — which is read [i] ("noite", "tarde", "dente"). A sounded
/// [e ɛ] keeps the plain [t] and [d] ("tempo", "dedo", "tela").
fn palatal_before(w: &[char], i: usize) -> bool {
    let Some(&c) = w.get(i) else { return false };
    let base = base_vowel(c);
    if base == 'i' {
        return true;
    }
    base == 'e' && i + 1 == w.len() && !is_accent_letter(c)
}

/// The sounds of a consonant letter or digraph at `i`, and how many letters they use.
fn consonants(w: &[char], i: usize) -> (Vec<&'static str>, usize) {
    let c = w[i];
    let next = w.get(i + 1).copied();
    match c {
        // "qu" is a single sound; the u is only there to mark it
        'q' => (vec!["k"], if next == Some('u') { 2 } else { 1 }),
        // "gu" before e or i is a bare g ("guerra"); before a it is a glide ("água");
        // elsewhere the u is a vowel of its own ("guarita")
        'g' => {
            if next == Some('u') && front_vowel_at(w, i + 2) {
                (vec!["g"], 2)
            } else if next == Some('u') && matches!(w.get(i + 2), Some('a')) {
                (vec!["g", "w"], 2)
            } else if front_vowel_at(w, i + 1) {
                (vec!["ʒ"], 1)
            } else {
                (vec!["g"], 1)
            }
        }
        // "ch", "lh", "nh" and "ll" are one sound each
        'c' if next == Some('h') => (vec!["ʃ"], 2),
        'l' if next == Some('h') => (vec!["ʎ"], 2),
        'n' if next == Some('h') => (vec!["ɲ"], 2),
        'l' if next == Some('l') => (vec!["ʎ"], 2),
        'c' => (if front_vowel_at(w, i + 1) { vec!["s"] } else { vec!["k"] }, 1),
        'd' => (if palatal_before(w, i + 1) { vec!["dʒ"] } else { vec!["d"] }, 1),
        // "ti" and "di" are the palatal affricates of most of Brazil
        't' => {
            if palatal_before(w, i + 1) {
                (vec!["tʃ"], 1)
            } else if i >= 2 && is_mente_tail(w, i - 2) {
                // the "te" of "-mente" is read [tʃi]
                (vec!["tʃ"], 1)
            } else {
                (vec!["t"], 1)
            }
        }
        'x' => (vec!["ʃ"], 1),
        'y' => (vec!["j"], 1),
        'b' => (vec!["b"], 1),
        'ç' => (vec!["s"], 1),
        'f' => (vec!["f"], 1),
        'h' => (Vec::new(), 1),
        'j' => (vec!["ʒ"], 1),
        'm' => (vec!["m"], 1),
        'n' => (vec!["n"], 1),
        'p' => (vec!["p"], 1),
        's' => {
            // [z] between vowels, [s] everywhere else — the word-final s included: before a pause
            // it is [s] ("casas" [ˈkazɐs]); [z] belongs before a vowel or a voiced consonant
            // across a word boundary, which a word-by-word reader cannot see.
            let between = vowel_at(w, i + 1) && i > 0 && is_vowel_letter(w[i - 1]);
            if between {
                (vec!["z"], 1)
            } else {
                (vec!["s"], 1)
            }
        }
        'v' => {
            // Portuguese keeps [v] everywhere — the [β] of the Spanish b/v is not ours
            (vec!["v"], 1)
        }
        'w' => (vec!["w"], 1),
        'z' => (vec!["z"], 1),
        'l' => {
            // a coda l is [w]: before a consonant ("alto" [ˈawtu]) or at the end of a word after a
            // vowel ("calma" [ˈkawmɐ]); before a vowel it is [l] ("falar")
            let after_vowel = i > 0 && is_vowel_letter(w[i - 1]);
            if i + 1 < w.len() && !vowel_at(w, i + 1) {
                (vec!["w"], 1)
            } else if i + 1 == w.len() && after_vowel {
                (vec!["w"], 1)
            } else {
                (vec!["l"], 1)
            }
        }
        'r' => {
            // the doubled r is one sound
            if next == Some('r') {
                return (vec!["ʁ"], 2);
            }
            // [ɾ] in the onset before a vowel, unless the word starts with it or it follows n or l;
            // everywhere else (the coda) it is the uvular [ʁ]
            let before_vowel = vowel_at(w, i + 1);
            let after_nl = i > 0 && matches!(w[i - 1], 'n' | 'l');
            if before_vowel && i > 0 && !after_nl {
                (vec!["ɾ"], 1)
            } else {
                (vec!["ʁ"], 1)
            }
        }
        _ => (Vec::new(), 1),
    }
}

/// Whether the letters from `i` spell the ending "-mente", whose "te" is read [tʃi] ("mente").
fn is_mente_tail(w: &[char], i: usize) -> bool {
    let tail: String = w[i.min(w.len())..].iter().collect();
    tail == "mente"
}

/// The sounds of the word, and its nuclei.
fn scan(w: &[char]) -> Vec<Phone> {
    let mut out: Vec<Phone> = Vec::with_capacity(w.len() + 2);
    let mut n = 0usize;
    let mut i = 0usize;
    while i < w.len() {
        let c = w[i];
        if is_vowel_letter(c) {
            let nasal = is_nasal_letter(c);
            let base = base_vowel(c);
            // the nasal diphthongs: ão, ãi, õe, õi are one nucleus
            if nasal {
                let glide = match w.get(i + 1) {
                    Some('o') => Some("w"),
                    Some('e') | Some('i') => Some("j"),
                    _ => None,
                };
                if let Some(glide) = glide {
                    // the nasal diphthong: a closed nasal vowel and its glide ("pão", "mãe")
                    let vowel: &str = match base {
                        'a' => "ɐ",
                        'e' => "e",
                        'i' => "i",
                        'o' => "o",
                        _ => "u",
                    };
                    out.push(Phone::nucleus(format!("{vowel}{NASAL}{glide}"), n, i, i + 2, true, false));
                    n += 1;
                    i += 2;
                    continue;
                }
            }
            // the plain diphthongs are one nucleus too: ai, ei, oi take a [j] glide, and au, eu,
            // ou a [w] ("pai" [ˈpaj], "noite" [ˈnojtʃi], "meu" [ˈmew]). An accented vowel is
            // never part of one ("saída" [saˈidɐ]) — it carries the stress of its own.
            if !nasal && !is_accent_letter(c) {
                let next_vowel = w.get(i + 1).copied().filter(|v| is_vowel_letter(*v) && !is_accent_letter(*v));
                let glide = match (base, next_vowel) {
                    ('a' | 'e' | 'o', Some('i')) => Some("j"),
                    ('a' | 'e' | 'o', Some('u')) => Some("w"),
                    ('u', Some('i')) => Some("j"),
                    _ => None,
                };
                if let Some(glide) = glide {
                    out.push(Phone::nucleus(format!("{base}{glide}"), n, i, i + 2, false, false));
                    n += 1;
                    i += 2;
                    continue;
                }
            }
            // the "te" of "-mente" is read [tʃi], the vowel a [i]
            let mente = i >= 4 && w[i - 4] == 'm' && is_mente_tail(w, i - 4);
            let ipa = if nasal {
                format!("{base}{NASAL}")
            } else if mente {
                "i".to_string()
            } else {
                base.to_string()
            };
            out.push(Phone::nucleus(ipa, n, i, i + 1, nasal, is_accent_letter(c)));
            n += 1;
            i += 1;
            continue;
        }
        let (sounds, used) = consonants(w, i);
        for (k, s) in sounds.iter().enumerate() {
            out.push(Phone::consonant(s, i + k, i + k + used));
        }
        i += used;
    }
    out
}

/// The onsets a syllable can start with: a single consonant, or one of the clusters Brazilian
/// Portuguese allows ("Português" has no "rt" onset, so the r opens the next syllable alone).
#[rustfmt::skip]
const ONSETS: &[&str] = &[
    "b", "c", "ç", "d", "f", "g", "j", "l", "m", "n", "p", "q", "r", "s", "t", "v", "w", "x", "z",
    "br", "cr", "dr", "fr", "gr", "pr", "tr", "vr", "bl", "cl", "fl", "gl", "kl", "pl", "sl",
    "ch", "lh", "nh", "gu", "qu", "ge", "gi",
];

/// How many of the consonants between two nuclei open the next syllable: the longest valid onset,
/// counted in sounds ("lh" is one sound and one onset; "br" is two).
fn onset_len(w: &[char], cluster: &[Phone]) -> usize {
    if cluster.is_empty() {
        return 0;
    }
    for k in (1..=cluster.len().min(2)).rev() {
        let spelling: String = w[cluster[0].letter..cluster[k - 1].end].iter().collect();
        if ONSETS.contains(&spelling.as_str()) {
            return k;
        }
    }
    0
}

/// The syllables of a word: the nuclei and the consonants around them. Each syllable is (the
/// first phone, the nucleus phone, the end of its coda): the longest valid onset opens the next
/// syllable, and the rest of a cluster closes this one.
fn syllables(w: &[char], phones: &[Phone]) -> Vec<(usize, usize, usize)> {
    let nuclei: Vec<usize> = phones.iter().enumerate().filter(|(_, p)| p.nucleus.is_some()).map(|(i, _)| i).collect();
    let mut out = Vec::with_capacity(nuclei.len());
    let mut first = 0usize;
    for (k, &v) in nuclei.iter().enumerate() {
        let coda_end = match nuclei.get(k + 1).copied() {
            Some(nv) => {
                // the consonants between the nuclei: the valid onset opens the next syllable
                nv - onset_len(w, &phones[v + 1..nv])
            }
            None => phones.len(),
        };
        out.push((first, v, coda_end));
        first = coda_end;
    }
    out
}

/// The endings that carry their own stress, and how far from the end of the word the stressed
/// nucleus is (1 = the last one). A word ending in a final r falls on it ("falar", "andar"), and
/// so does a word ending in a closed syllable that Portuguese writes oxytone ("Brasil",
/// "feliz", "aqui", "comum"); "-mente" and "-ia" are rules of their own, above.
#[rustfmt::skip]
const ENDINGS: &[(&str, usize)] = &[
    ("ei", 1), ("ou", 1), ("r", 1), ("ns", 1), ("im", 1), ("um", 1),
    ("l", 1), ("z", 1), ("i", 1), ("u", 1),
];

/// The word's spelling without its ending, as a string (for the -mente rule).
fn word_string(w: &[char]) -> String {
    w.iter().collect()
}

/// Where the stress falls: the nucleus's ordinal.
fn stress(w: &[char], phones: &[Phone]) -> Option<usize> {
    let nuclei: Vec<usize> = phones.iter().filter_map(|p| p.nucleus).collect();
    if nuclei.is_empty() {
        return None;
    }
    let last = nuclei.len() - 1;
    let text = word_string(w);

    // a word ending in a nasal vowel falls on it ("irmão", "manhã", "coração")
    let ends_nasal = phones.last().is_some_and(|p| p.nucleus.is_some() && p.nasal);

    // -mente carries the stress: the "men" of the suffix is the stressed syllable
    // ("rapidamente" [ʁapidaˈmɛ̃ntʃi]); a written accent in the stem still wins.
    if let Some(stem) = text.strip_suffix("mente") {
        let stem_phones = scan(&stem.chars().collect::<Vec<char>>());
        let accent = stem_phones.iter().find(|p| p.accented).and_then(|p| p.nucleus);
        let circumflex = stem.chars().any(|c| matches!(c, 'â' | 'ê' | 'ô'));
        if let Some(n) = accent {
            if !(circumflex && ends_nasal) {
                return Some(n);
            }
        }
        return Some(last.saturating_sub(1));
    }

    // a written accent wins ("café", "único") — but a circumflex before a nasal ending marks the
    // nasalisation and not the stress ("bênção")
    if let Some(a) = phones.iter().find(|p| p.accented) {
        let circumflex = matches!(w.get(a.letter), Some('â') | Some('ê') | Some('ô'));
        if !(circumflex && ends_nasal) {
            return a.nucleus;
        }
    }

    if ends_nasal {
        return Some(last);
    }

    // the imperfect -ia, -iam: the stress falls on the antepenult, or on the penult in the
    // short words ("Maria" [maˈɾiɐ], "falava" [faˈlavɐ], "academia" [akaˈdɛmiɐ])
    if text.ends_with("ia") || text.ends_with("iam") {
        let back = if nuclei.len() == 3 { 2 } else { 3 };
        return Some(last.saturating_sub(back - 1));
    }

    // the endings that carry their own stress
    for (ending, back) in ENDINGS {
        if text.ends_with(ending) {
            return Some(last.saturating_sub(back - 1));
        }
    }

    // everything else: the penult
    Some(last.saturating_sub(1))
}

/// A word read: its sounds, its stressed nucleus, and the syllables they make.
struct Parsed {
    phones: Vec<Phone>,
    syllables: Vec<(usize, usize, usize)>,
    stressed: Option<usize>,
}

impl Parsed {
    /// The sound of a phone, with the vowel quality the rules give it.
    fn ipa(&self, p: &Phone) -> String {
        let Some(n) = p.nucleus else { return p.ipa.clone() };
        let stressed = self.stressed == Some(n);
        // a nucleus with a glide (a diphthong) keeps the vowel it was written with
        if p.ipa.contains('w') || p.ipa.contains('j') {
            return p.ipa.clone();
        }
        // a vowel in a closed syllable is short and central; a word-final one closes too, but
        // only when it is not stressed ("casa" [ˈkazɐ], "café" [kaˈfe])
        let closed = self.has_coda(n) || (self.word_final(n) && !stressed);
        let base = p.ipa.trim_end_matches(NASAL);
        let final_vowel = self.last_sound(n);
        let mut out = match (base, stressed, closed) {
            // an unstressed /o/ at the end of a word rises to [u] ("copo", "sapato")
            ("o", false, true) => "u".to_string(),
            ("a", false, true) => "ɐ".to_string(),
            // a word-final unstressed /e/ is [i], not [ɛ] ("fome" [ˈfomi], "dente" [ˈdẽntʃi])
            ("e", false, true) if final_vowel => "i".to_string(),
            ("e", false, true) => "ɛ".to_string(),
            // a stressed /a/ stays open ("mar", "calma"); in a monosyllable it closes
            // ("já" [ʒˈɐ], "lá" [ˈlɐ]); a stressed /e o/ closes ("porta")
            ("a", true, _) => {
                if final_vowel && self.syllables.len() == 1 {
                    "ɐ".to_string()
                } else {
                    "a".to_string()
                }
            }
            ("e", true, true) => "ɛ".to_string(),
            ("o", true, true) => "ɔ".to_string(),
            (b, _, _) => b.to_string(),
        };
        if self.nasalised(p) {
            out.push(NASAL);
        }
        out
    }

    /// Whether the nucleus is nasalised: written with a tilde, or before a nasal in its coda
    /// ("homem" [ˈomẽm], "falaram" [faˈlaɾɐ̃m]).
    fn nasalised(&self, p: &Phone) -> bool {
        if p.nasal {
            return true;
        }
        let Some(n) = p.nucleus else { return false };
        let Some(&(_, v, coda_end)) = self.syllables.iter().find(|(_, v, _)| self.phones[*v].nucleus == Some(n)) else {
            return false;
        };
        self.phones[v + 1..coda_end].iter().any(|ph| ph.ipa == "m" || ph.ipa == "n")
    }

    /// Whether the nucleus's syllable has a coda ("admissão", "cor").
    fn has_coda(&self, n: usize) -> bool {
        let Some(&(_, v, coda_end)) = self.syllables.iter().find(|(_, v, _)| self.phones[*v].nucleus == Some(n)) else {
            return false;
        };
        coda_end > v + 1
    }

    /// Whether the nucleus is the last one of the word.
    fn word_final(&self, n: usize) -> bool {
        self.syllables.last().and_then(|(_, v, _)| self.phones[*v].nucleus) == Some(n)
    }

    /// Whether the nucleus is the last *sound* of the word — the vowel itself, with no consonant
    /// after it ("já" [ʒˈɐ], but "mar" [ˈmaʁ] keeps its open [a]).
    fn last_sound(&self, n: usize) -> bool {
        let Some(i) = self.phones.iter().position(|p| p.nucleus == Some(n)) else { return false };
        i + 1 == self.phones.len()
    }

    fn segment(&self, p: &Phone) -> Segment {
        Segment {
            ipa: self.ipa(p),
            dur: None,
            stress: Some(u8::from(p.nucleus.is_some() && p.nucleus == self.stressed)),
            nasalized: self.nasalised(p).then_some(true),
            ..Default::default()
        }
    }
}

/// The syllables of a word, with the sounds of each.
fn parse(w: &[char]) -> Parsed {
    let phones = scan(w);
    let syllables = syllables(w, &phones);
    let stressed = stress(w, &phones);
    Parsed { phones, syllables, stressed }
}

/// One word, as typed, to segments with stress.
pub fn word(typed: &str) -> Vec<Segment> {
    let letters: Vec<char> = typed.chars().filter(|c| is_letter(*c)).map(|c| c.to_lowercase().next().unwrap_or(c)).collect();
    let p = parse(&letters);
    p.phones.iter().map(|ph| p.segment(ph)).collect()
}

// ---------------------------------------------------------------- singing

/// One word's sung syllables, split where Portuguese spelling splits them: a single consonant
/// between two vowels opens the next syllable ("a-mi-go"), the rest of a cluster closes this one
/// ("ad-mis-são").
fn sung_syllables(typed: &str) -> Vec<Syllable> {
    let letters: Vec<char> = typed.chars().filter(|c| is_letter(*c)).map(|c| c.to_lowercase().next().unwrap_or(c)).collect();
    let p = parse(&letters);
    if p.syllables.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<Syllable> = Vec::with_capacity(p.syllables.len());
    for &(first, v, coda_end) in &p.syllables {
        // the syllable's text is its whole spelling, the coda included ("ad", "mis")
        let end = if coda_end < p.phones.len() { p.phones[coda_end - 1].end } else { p.phones[v].end };
        let text: String = letters[p.phones[first].letter..end].iter().collect();
        let mut syl = Syllable { text, ..Default::default() };
        for ph in &p.phones[first..v] {
            syl.onset.push(p.segment(ph));
        }
        syl.nucleus = p.segment(&p.phones[v]);
        for ph in &p.phones[v + 1..coda_end] {
            syl.coda.push(p.segment(ph));
        }
        out.push(syl);
    }
    out
}

/// The Portuguese front end for the lyrics router ([`super::syllables::read_lyrics`]). Portuguese
/// is written in Latin letters, so the router can't tell it from English or Turkish by script:
/// the player chooses it ([`Lang`]).
pub struct Portuguese;

impl FrontEnd for Portuguese {
    fn starts(&self, c: char) -> bool {
        is_letter(c)
    }
    fn continues(&self, c: char) -> bool {
        self.starts(c) || c == '-'
    }
    fn word(&self, tok: &str) -> Vec<Syllable> {
        let mut out = Vec::new();
        for part in tok.split('-').filter(|p| p.chars().any(is_letter)) {
            out.append(&mut sung_syllables(part));
        }
        out
    }
}

/// Lyrics → sung syllables with Latin script read as Portuguese ([`Lang::Pt`]); other scripts are
/// routed as usual, and no dictionary is needed.
pub fn lyrics_to_syllables(text: &str) -> Vec<Syllable> {
    lyrics_in(text, &|_: &str, _: bool| Vec::<Segment>::new(), &MandarinLexicon::new(), Lang::Pt)
}

// ---------------------------------------------------------------- speech

/// Portuguese text → phrases of segments. Phrase punctuation breaks phrases as in English;
/// numbers are read out ("1.923,5" → "mil novecentos e vinte e três vírgula cinco").
pub fn to_phrases(text: &str) -> Vec<Phrase> {
    let normalized = expand_numbers(&text.replace('’', "'"));
    let mut phrases = Vec::new();
    let mut words: Vec<String> = Vec::new();
    let mut flush = |delim: &str, words: &mut Vec<String>| {
        let mut segs: Vec<Segment> = Vec::new();
        let mut said: Vec<String> = Vec::new();
        for w in words.drain(..) {
            let mut ws = word(&w);
            let Some(last) = ws.last_mut() else { continue };
            last.word_end = Some(true);
            segs.append(&mut ws);
            said.push(w);
        }
        if segs.is_empty() {
            return;
        }
        let pause = if delim.contains(['.', '!', '?']) {
            0.35
        } else if !delim.is_empty() {
            0.18
        } else {
            0.3
        };
        phrases.push(Phrase { segments: segs, intonation: Intonation::Statement, pause, words: said });
    };
    for part in split_keeping(&normalized, is_phrase_punct) {
        if !part.is_empty() && part.chars().all(is_phrase_punct) {
            flush(&part, &mut words);
        } else {
            words.extend(part.split(|c: char| !is_letter(c)).filter(|w| w.chars().any(is_letter)).map(str::to_string));
        }
    }
    flush("", &mut words);
    phrases
}

// ---------------------------------------------------------------- numbers

const ONES: [&str; 10] = ["zero", "um", "dois", "três", "quatro", "cinco", "seis", "sete", "oito", "nove"];
const TEENS: [&str; 10] = ["dez", "onze", "doze", "treze", "quatorze", "quinze", "dezesseis", "dezessete", "dezoito", "dezenove"];
const TENS: [&str; 10] = ["", "", "vinte", "trinta", "quarenta", "cinquenta", "sessenta", "setenta", "oitenta", "noventa"];
const HUNDREDS: [&str; 10] = ["", "cento", "duzentos", "trezentos", "quatrocentos", "quinhentos", "seiscentos", "setecentos", "oitocentos", "novecentos"];
const SCALES: [(u64, &str); 4] = [(1_000_000_000_000, "trilhão"), (1_000_000_000, "bilhão"), (1_000_000, "milhão"), (1_000, "mil")];

fn below_thousand(n: u64, out: &mut Vec<String>) {
    let (h, t, o) = ((n / 100) as usize, (n / 10 % 10) as usize, (n % 10) as usize);
    if h > 0 {
        if h > 1 {
            // "duzentos", "novecentos"
            out.push(HUNDREDS[h].to_string());
        } else if t > 0 || o > 0 {
            // "cem" alone, "cento" when something follows ("cento e vinte e três")
            out.push(HUNDREDS[1].to_string());
        } else {
            out.push("cem".to_string());
        }
    }
    if t > 0 {
        if h > 0 {
            out.push("e".to_string());
        }
        if t == 1 && o > 0 {
            // 11 to 19 are one word ("quinze")
            out.push(TEENS[o].to_string());
        } else if t == 1 {
            out.push(TEENS[0].to_string());
        } else {
            out.push(TENS[t].to_string());
        }
    }
    if o > 0 && !(t == 1) {
        if t > 0 || h > 0 {
            out.push("e".to_string());
        }
        out.push(ONES[o].to_string());
    }
}

/// Digits → Portuguese number words.
fn number_words(digits: &str) -> String {
    let d = digits.trim_start_matches('0');
    if d.is_empty() {
        return ONES[0].into();
    }
    let Some(mut n) = d.parse::<u64>().ok().filter(|n| *n < 1_000_000_000_000_000) else {
        return digits.bytes().map(|b| ONES[(b - b'0') as usize]).collect::<Vec<_>>().join(" ");
    };
    let mut out: Vec<String> = Vec::new();
    for (scale, name) in SCALES {
        let q = n / scale;
        if q > 0 {
            if !(scale == 1_000 && q == 1) {
                below_thousand(q, &mut out);
            }
            out.push(name.to_string());
            n %= scale;
        }
    }
    below_thousand(n, &mut out);
    out.join(" ")
}

/// Numbers in text → words. A dot before three digits groups thousands ("1.000"), a comma
/// between digits is the decimal comma ("3,05" → "três vírgula zero cinco").
fn expand_numbers(t: &str) -> String {
    let c: Vec<char> = t.chars().collect();
    let digit = |i: usize| c.get(i).is_some_and(char::is_ascii_digit);
    let mut out = String::with_capacity(t.len());
    let mut i = 0;
    while i < c.len() {
        if !digit(i) {
            out.push(c[i]);
            i += 1;
            continue;
        }
        let mut int = String::new();
        loop {
            while digit(i) {
                int.push(c[i]);
                i += 1;
            }
            if c.get(i) == Some(&'.') && digit(i + 1) && digit(i + 2) && digit(i + 3) && !digit(i + 4) {
                i += 1;
                continue;
            }
            break;
        }
        out.push(' ');
        out.push_str(&number_words(&int));
        if c.get(i) == Some(&',') && digit(i + 1) {
            i += 1;
            let mut frac = String::new();
            while digit(i) {
                frac.push(c[i]);
                i += 1;
            }
            out.push_str(" vírgula");
            let zeros = frac.len() - frac.trim_start_matches('0').len();
            for _ in 0..zeros {
                out.push_str(" zero");
            }
            if zeros < frac.len() {
                out.push(' ');
                out.push_str(&number_words(&frac[zeros..]));
            }
        }
        out.push(' ');
    }
    out
}
