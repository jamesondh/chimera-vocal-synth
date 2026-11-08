//! Turkish lyrics → sung syllables. Turkish spelling is close to one letter per sound, so rules
//! stand in for the English dictionary: each letter maps to a phoneme (k, g and l take their front
//! forms beside front vowels, soft ğ is silent or a glide), and a syllable holds one vowel with at
//! most one consonant before it inside a word ("kar-deş", "Üs-kü-dar"). Hyphens, lone "-"/"~"
//! melisma marks and phrase punctuation work as in English lyrics.

use super::english::js_is_space;
use super::syllables::is_lyric_punct;
use super::{Segment, Syllable};

/// Turkish lowercase: dotless I → ı, dotted İ → i (Unicode's default maps I → i).
fn lower(c: char) -> char {
    match c {
        'I' => 'ı',
        'İ' => 'i',
        _ => c.to_lowercase().next().unwrap_or(c),
    }
}

fn vowel_ipa(c: char) -> Option<&'static str> {
    Some(match c {
        'a' | 'â' => "a",
        'e' => "e",
        'ı' => "ɯ",
        'i' | 'î' => "i",
        'o' => "o",
        'ö' => "ø",
        'u' | 'û' => "u",
        'ü' => "y",
        _ => return None,
    })
}

/// Front vowels; a circumflex on a or u marks the consonant before it as front ("kâr", "lâle").
fn is_front(c: char) -> bool {
    matches!(c, 'e' | 'i' | 'î' | 'ö' | 'ü' | 'â' | 'û')
}

/// Whether k, g or l at index i takes its front form: from the next letter when it is a vowel,
/// otherwise from the vowel before ("renk", "gel").
fn front_context(w: &[char], i: usize) -> bool {
    if let Some(&n) = w.get(i + 1) {
        if vowel_ipa(n).is_some() {
            return is_front(n);
        }
    }
    w[..i].iter().rev().find(|c| vowel_ipa(**c).is_some()).is_some_and(|c| is_front(*c))
}

/// One sound and the index of the letter it starts at.
struct Unit {
    ipa: &'static str,
    a: usize,
}

/// A word's sounds. Letters that make no sound of their own (ğ after a vowel, an apostrophe,
/// anything outside the alphabet) are written with the sound before them: a syllable's text runs
/// up to the next syllable's first sound.
fn units(w: &[char]) -> Vec<Unit> {
    let mut out: Vec<Unit> = Vec::new();
    for (i, &c) in w.iter().enumerate() {
        let ipa: &[&'static str] = if let Some(v) = vowel_ipa(c) {
            &[v][..]
        } else {
            match c {
                'b' => &["b"],
                'c' => &["dʒ"],
                'ç' => &["tʃ"],
                'd' => &["d"],
                'f' => &["f"],
                'g' => if front_context(w, i) { &["ɟ"] } else { &["g"] },
                // soft g: a glide between front vowels ("değil"), otherwise it lengthens the vowel
                'ğ' => {
                    let prev = i.checked_sub(1).and_then(|p| w.get(p)).copied();
                    let next = w.get(i + 1).copied();
                    if prev.is_some_and(|p| is_front(p) && vowel_ipa(p).is_some()) && next.is_some_and(|n| is_front(n) && vowel_ipa(n).is_some()) { &["j"] } else { &[] }
                }
                'h' => &["h"],
                'j' => &["ʒ"],
                'k' | 'q' => if front_context(w, i) { &["c"] } else { &["k"] },
                'l' => if front_context(w, i) { &["l"] } else { &["ɫ"] },
                'm' => &["m"],
                'n' => &["n"],
                'p' => &["p"],
                'r' => &["ɾ"],
                's' => &["s"],
                'ş' => &["ʃ"],
                't' => &["t"],
                'v' | 'w' => &["v"],
                'x' => &["k", "s"],
                'y' => &["j"],
                'z' => &["z"],
                _ => &[],
            }
        };
        for (k, p) in ipa.iter().enumerate() {
            // a letter written as two sounds ("x") shows on the first
            out.push(Unit { ipa: p, a: if k == 0 { i } else { i + 1 } });
        }
    }
    out
}

/// One written word (no hyphens) → its syllables, spelling split to match.
fn word_syllables(w: &[char]) -> Vec<Syllable> {
    let lw: Vec<char> = w.iter().map(|&c| lower(c)).collect();
    let us = units(&lw);
    let vi: Vec<usize> = us.iter().enumerate().filter(|(_, u)| vowel_ipa_of(u.ipa)).map(|(i, _)| i).collect();
    if vi.is_empty() {
        return Vec::new();
    }
    // unit index where each syllable starts: one consonant before a vowel opens its syllable
    let mut starts = vec![0];
    for pair in vi.windows(2) {
        let (v, next) = (pair[0], pair[1]);
        starts.push(if next - v > 1 { next - 1 } else { next });
    }
    let seg = |u: &Unit| Segment::new(u.ipa);
    let mut out = Vec::new();
    for (n, &v) in vi.iter().enumerate() {
        let (s, e) = (starts[n], starts.get(n + 1).copied().unwrap_or(us.len()));
        let a = if n == 0 { 0 } else { us[s].a };
        let b = if n + 1 == vi.len() { w.len() } else { us[e].a };
        out.push(Syllable {
            text: w[a..b].iter().collect(),
            onset: us[s..v].iter().map(seg).collect(),
            nucleus: seg(&us[v]),
            coda: us[v + 1..e].iter().map(seg).collect(),
            melisma: None,
            word_end: false,
            phrase_end: false,
        });
    }
    out
}

fn vowel_ipa_of(ipa: &str) -> bool {
    matches!(ipa, "a" | "e" | "ɯ" | "i" | "o" | "ø" | "u" | "y")
}

fn is_word_char(c: char) -> bool {
    c.is_alphabetic() || c == '\''
}

/// Turkish lyrics text → syllables.
pub fn lyrics_to_syllables(text: &str) -> Vec<Syllable> {
    let s: Vec<char> = text.replace('’', "'").chars().collect();
    let mut out: Vec<Syllable> = Vec::new();
    let mut i = 0;
    while i < s.len() {
        let c = s[i];
        if is_word_char(c) {
            let start = i;
            while i < s.len() && (is_word_char(s[i]) || s[i] == '-') {
                i += 1;
            }
            let mut syls: Vec<Syllable> = s[start..i].split(|&c| c == '-').filter(|p| !p.is_empty()).flat_map(word_syllables).collect();
            if let Some(last) = syls.last_mut() {
                last.word_end = true;
            }
            out.append(&mut syls);
            continue;
        }
        let melisma = (c == '~' || c == '-') && (i == 0 || js_is_space(s[i - 1])) && (i + 1 == s.len() || js_is_space(s[i + 1]));
        if melisma {
            if let Some(prev) = out.last() {
                let held = Syllable { text: "~".into(), onset: vec![], nucleus: prev.nucleus.clone(), coda: vec![], melisma: Some(true), word_end: prev.word_end, phrase_end: false };
                out.push(held);
            }
        } else if is_lyric_punct(c) {
            if let Some(last) = out.last_mut() {
                last.phrase_end = true;
            }
        }
        i += 1;
    }
    out
}
