//! Lyrics → sung syllables, through language front ends (docs/language-front-ends.md). A front
//! end claims the words written in its script and turns each into syllables in IPA; the router
//! here walks the lyrics, sends each word to the front end that claims it, and handles what every
//! language shares: melisma marks, phrase punctuation and word ends.
//!
//! English words come from the pronouncing dictionary (unreduced: sung function words keep full
//! vowels), are split into syllables by the maximal-onset principle with English onset clusters,
//! and the spelling is split to match for display ("twin-kle"). Hangul words go to
//! [`super::korean`], one syllable per block, Cyrillic words to [`super::russian`]. Typed hyphens force syllable breaks; a lone "-" (or
//! "~") holds the previous vowel onto the next note (melisma), as in vocaloid editors.

use crate::anatomy::human::BodyMix;
use super::english::js_is_space;
use super::korean::Korean;
use super::russian::Russian;
use super::planner::is_vowel;
use super::{Segment, Syllable};
use serde::{Deserialize, Serialize};

// English onsets (IPA), longest first matters only for the search below
#[rustfmt::skip]
pub(crate) const ONSETS_EN: &[&str] = &[
    "p", "b", "t", "d", "k", "g", "f", "v", "θ", "ð", "s", "z", "ʃ", "ʒ", "h", "m", "n", "l", "ɹ", "w", "j", "tʃ", "dʒ",
    "pl", "bl", "kl", "gl", "fl", "sl", "pɹ", "bɹ", "tɹ", "dɹ", "kɹ", "gɹ", "fɹ", "θɹ", "ʃɹ",
    "sp", "st", "sk", "sm", "sn", "sw", "sf", "tw", "kw", "dw", "gw", "θw",
    "pj", "bj", "kj", "gj", "mj", "nj", "fj", "vj", "hj", "lj",
    "spl", "spɹ", "stɹ", "skɹ", "skw", "skj", "spj",
];

/// Spanish onsets: any consonant alone, the C+l/r clusters, consonant–glide pairs (the rising
/// diphthongs: "cie-lo", "abue-lo"), and "ks" (written x: e-ksa-men).
#[rustfmt::skip]
pub(crate) const ONSETS_ES: &[&str] = &[
    "p", "b", "t", "d", "k", "g", "f", "s", "x", "θ", "ʃ", "ʒ", "ʝ", "m", "n", "ɲ", "l", "r", "ɾ", "w", "j", "tʃ", "h", "β", "ð", "ɣ",
    "pl", "bl", "kl", "gl", "fl", "pr", "br", "tr", "dr", "kr", "gr", "fr", "ks",
    "pj", "bj", "tj", "dj", "kj", "gj", "fj", "sj", "mj", "nj", "lj", "rj", "xj",
    "pw", "bw", "tw", "dw", "kw", "gw", "fw", "sw", "mw", "nw", "lw", "rw", "xw",
    "tʃj", "tʃw",
];

/// Which language's onset clusters and spelling digraphs a word's syllables follow.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    English,
    Spanish,
}

/// The spelling digraphs that stay together on the right of a syllable break.
const DIGRAPHS_EN: &[&str] = &["th", "sh", "ch", "ph", "wh", "ck", "gh"];
const DIGRAPHS_ES: &[&str] = &["rr", "ll", "ch", "qu", "gu"];

/// A final silent e ("like") is not a syllable in English; Spanish final e always is.
fn silent_e(lang: Lang) -> bool {
    lang == Lang::English
}

/// One syllable of a word, before it gets its text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyllableParts {
    pub onset: Vec<Segment>,
    pub nucleus: Segment,
    pub coda: Vec<Segment>,
}

/// Split one word's segments into onset / nucleus / coda syllables (English onsets).
pub fn syllabify_segments(segs: &[Segment]) -> Vec<SyllableParts> {
    syllabify_segments_lang(segs, Lang::English)
}

/// A consonant cluster that can start an English syllable.
pub fn is_english_onset(cluster: &[Segment]) -> bool {
    let cluster: String = cluster.iter().map(|s| s.ipa.as_str()).collect();
    ONSETS_EN.contains(&cluster.as_str())
}

/// A consonant cluster that can start a Spanish syllable.
pub fn is_spanish_onset(cluster: &[Segment]) -> bool {
    let cluster: String = cluster.iter().map(|s| s.ipa.as_str()).collect();
    ONSETS_ES.contains(&cluster.as_str())
}

/// Split one word's segments with the selected language's legal onset clusters.
pub fn syllabify_segments_lang(segs: &[Segment], lang: Lang) -> Vec<SyllableParts> {
    let is_onset: fn(&[Segment]) -> bool = match lang {
        Lang::English => is_english_onset,
        Lang::Spanish => is_spanish_onset,
    };
    syllabify_with(segs, is_onset)
}

/// Split segments by assigning the longest legal onset between two vowel nuclei.
pub fn syllabify_with(segs: &[Segment], is_onset: impl Fn(&[Segment]) -> bool) -> Vec<SyllableParts> {
    let vi: Vec<usize> = segs.iter().enumerate().filter(|(_, s)| is_vowel(&s.ipa, BodyMix::MALE)).map(|(i, _)| i).collect();
    if vi.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut start = 0;
    for (n, &v) in vi.iter().enumerate() {
        let mut end = segs.len(); // exclusive end of this syllable
        if let Some(&next) = vi.get(n + 1) {
            // consonants between this vowel and the next: the longest legal onset goes right
            let between = &segs[v + 1..next];
            let mut k = 0;
            for len in (1..=between.len()).rev() {
                if is_onset(&between[between.len() - len..]) {
                    k = len;
                    break;
                }
            }
            end = next - k;
        }
        out.push(SyllableParts { onset: segs[start..v].to_vec(), nucleus: segs[v].clone(), coda: segs[v + 1..end].to_vec() });
        start = end;
    }
    out
}

// The spelling is indexed in UTF-16 code units, as the JS string was.
/// Spanish vowels include their written accents ("camión" splits at the ió group).
fn is_spell_vowel(c: u16, lang: Lang) -> bool {
    "aeiouy".bytes().any(|v| v as u16 == c)
        || (lang == Lang::Spanish && matches!(c, 0xE1 | 0xE9 | 0xED | 0xF3 | 0xFA | 0xFC)) // á é í ó ú ü
}
fn is_spell_vowel_ci(c: u16) -> bool {
    "aeiouyAEIOUY".bytes().any(|v| v as u16 == c)
}

/// A Spanish vowel run breaks where the pronunciation has two nuclei — mirroring the G2P's
/// `merge_vowel_runs`: a written accent starts a nucleus ([pa-ís], [dí-a]) and so do two strong
/// vowels ([ca-e], [le-er]); a weak unstressed vowel glides into the next one and a strong + weak
/// pair is one diphthong ([pia-no], [ai-re], [ciu-dad]).
fn split_es_vowel_runs(lower: &[u16], runs: &[[usize; 2]]) -> Vec<[usize; 2]> {
    let acc = |c: u16| matches!(c, 0xE1 | 0xE9 | 0xED | 0xF3 | 0xFA); // á é í ó ú
    let weak = |c: u16| matches!(c, 0x69 | 0x75 | 0xED | 0xFA); // i u í ú
    let strong = |c: u16| matches!(c, 0x61 | 0x65 | 0x6F | 0xE1 | 0xE9 | 0xF3); // a e o á é ó
    let mut out = Vec::new();
    for &[s, e] in runs {
        let mut start = s;
        let mut i = s;
        while i < e {
            let v1 = lower[i];
            if i + 1 < e {
                let v2 = lower[i + 1];
                // one nucleus: the pair stays together (a glide, or a falling diphthong token)
                if weak(v1) && !acc(v1)
                    || (strong(v1) && weak(v2) && !acc(v2)
                        && matches!((v1, v2), (0x61, 0x69) | (0x61, 0x75) | (0x65, 0x69) | (0x65, 0x75) | (0x6F, 0x69) | (0x6F, 0x75)))
                {
                    i += 1;
                    continue;
                }
                // two nuclei: the run breaks here
                out.push([start, i + 1]);
                start = i + 1;
            }
            i += 1;
        }
        out.push([start, e]);
    }
    out
}

fn eq_ci(c: u16, lower: u8) -> bool {
    c == lower as u16 || c == lower.to_ascii_uppercase() as u16
}
/// `word.slice(a, b)` (indices clamped to the word)
fn slice(w: &[u16], a: usize, b: usize) -> String {
    let (a, b) = (a.min(w.len()), b.min(w.len()));
    if a >= b { String::new() } else { String::from_utf16_lossy(&w[a..b]) }
}

/// Split a written word into n chunks at its vowel groups (approximate hyphenation, English).
pub fn split_spelling(word: &str, n: usize) -> Vec<String> {
    split_spelling_lang(word, n, Lang::English)
}

/// Split a written word into n chunks at its vowel groups (approximate hyphenation).
pub fn split_spelling_lang(word: &str, n: usize, lang: Lang) -> Vec<String> {
    if n <= 1 {
        return vec![word.to_string()];
    }
    let w: Vec<u16> = word.encode_utf16().collect();
    let lower: Vec<u16> = word.to_lowercase().encode_utf16().collect();
    // /[aeiouyáéíóúü]+/g on the lowercased word
    let mut runs: Vec<[usize; 2]> = Vec::new();
    let mut i = 0;
    while i < lower.len() {
        if is_spell_vowel(lower[i], lang) {
            let s = i;
            while i < lower.len() && is_spell_vowel(lower[i], lang) {
                i += 1;
            }
            runs.push([s, i]);
        } else {
            i += 1;
        }
    }
    // Spanish: a run breaks where the pronunciation has two nuclei (mirroring
    // `merge_vowel_runs`: [pa-ís], [ca-e], [dí-a] split; [pia-no], [ai-re], [ciu-dad] don't)
    let mut groups: Vec<[usize; 2]> = match lang {
        Lang::English => runs,
        Lang::Spanish => split_es_vowel_runs(&lower, &runs),
    };
    let len = w.len();
    // /[^aeiouy]e$/i and /[^aeiouyl]le$/i (English: a final silent e is not a syllable; the
    // Spanish final e always is)
    let silent_e = silent_e(lang) && len >= 2 && eq_ci(w[len - 1], b'e') && !is_spell_vowel_ci(w[len - 2]);
    let le = silent_e && len >= 3 && eq_ci(w[len - 1], b'e') && eq_ci(w[len - 2], b'l') && !is_spell_vowel_ci(w[len - 3]) && !eq_ci(w[len - 3], b'l');
    // a final silent e ("like") is not a syllable, but consonant + "le" is ("twin-kle", "lit-tle")
    if groups.len() > 1 && silent_e && !le && groups[groups.len() - 1][0] + 1 == len {
        groups.pop();
    }
    if groups.len() < n {
        // fewer written vowels than sung syllables (e.g. "fire"): hold the word, then continue marks
        let mut out = vec![word.to_string()];
        out.extend(std::iter::repeat_n("~".to_string(), n - 1));
        return out;
    }
    let digraphs = match lang {
        Lang::English => DIGRAPHS_EN,
        Lang::Spanish => DIGRAPHS_ES,
    };
    while groups.len() > n {
        let last = groups.pop().unwrap();
        let k = groups.len() - 1;
        groups[k][1] = last[1];
    }
    let mut cuts: Vec<usize> = vec![0];
    for i in 0..groups.len() - 1 {
        let (a, b) = (groups[i][1], groups[i + 1][0]); // consonant letters a..b between vowel groups
        // a digraph stays together on the right ("any-where", "fa-ther", "gue-rra"); otherwise
        // one consonant goes right and longer runs split in the middle ("lit-tle", "won-der")
        let gap = b - a;
        let dig = gap >= 2 && {
            let d = [w.get(b - 2).copied(), w.get(b - 1).copied()];
            digraphs.iter().any(|p| {
                let p = p.as_bytes();
                d[0].is_some_and(|c| eq_ci(c, p[0])) && d[1].is_some_and(|c| eq_ci(c, p[1]))
            })
        };
        cuts.push(if dig { b - 2 } else { a + if gap <= 1 { 0 } else { gap / 2 } });
    }
    cuts.push(len);
    (0..cuts.len() - 1).map(|i| slice(&w, cuts[i], cuts[i + 1])).collect()
}

/// Where lyrics get their pronunciations: `EnglishG2P`, or any `Fn(&str, bool) -> Vec<Segment>`.
pub trait WordSource {
    fn word(&self, w: &str, reduce: bool) -> Vec<Segment>;
    /// which language's syllables and spelling the lyrics follow
    fn lang(&self) -> Lang {
        Lang::English
    }
}

impl<F: Fn(&str, bool) -> Vec<Segment>> WordSource for F {
    fn word(&self, w: &str, reduce: bool) -> Vec<Segment> {
        self(w, reduce)
    }
}

fn is_lyric_punct(c: char) -> bool {
    matches!(c, '.' | ',' | ';' | ':' | '!' | '?' | '\n')
}
/// A language front end: claims the words written in its script and turns each into sung syllables
/// (IPA onset, nucleus and coda; each syllable's text is its part of the word's spelling).
pub trait FrontEnd {
    /// A word this front end reads starts with `c`.
    fn starts(&self, c: char) -> bool;
    /// `c` continues a word this front end started.
    fn continues(&self, c: char) -> bool {
        self.starts(c)
    }
    /// One word → its syllables (the router marks the last as the word's end).
    fn word(&self, word: &str) -> Vec<Syllable>;
    /// The same, knowing the word of this front end just before it in the phrase, for a language
    /// whose words lean on each other (a Russian preposition with no vowel has no note of its
    /// own: it returns no syllables and starts the next word's first one).
    fn word_after(&self, word: &str, _prev: Option<&str>) -> Vec<Syllable> {
        self.word(word)
    }
}

/// English, through a [`WordSource`] (the pronouncing dictionary, or letter-to-sound rules).
pub struct English<'a, W: WordSource>(pub &'a W);

impl<W: WordSource> FrontEnd for English<'_, W> {
    fn starts(&self, c: char) -> bool {
        c.is_ascii_alphabetic() || c == '\''
    }
    fn continues(&self, c: char) -> bool {
        self.starts(c) || c == '-'
    }
    fn word(&self, tok: &str) -> Vec<Syllable> {
        // typed hyphens split syllables explicitly ("ti-ger"); otherwise the dictionary decides
        let parts: Vec<&str> = tok.split('-').filter(|p| !p.is_empty()).collect();
        let mut syls: Vec<Syllable> = Vec::new();
        for part in &parts {
            let sy = syllabify_segments(&self.0.word(&part.to_lowercase(), false));
            let spell: Vec<String> = if parts.len() > 1 {
                let mut v = vec![part.to_string()];
                v.extend(std::iter::repeat_n("~".to_string(), sy.len().saturating_sub(1)));
                v
            } else {
                split_spelling(part, sy.len())
            };
            for (i, s) in sy.into_iter().enumerate() {
                syls.push(Syllable { text: spell.get(i).cloned().unwrap_or_else(|| "~".into()), onset: s.onset, nucleus: s.nucleus, coda: s.coda, melisma: None, word_end: false, phrase_end: false, ..Default::default() });
            }
        }
        syls
    }
}

/// Spanish, through a [`WordSource`] (the Spanish G2P and its dialects). Words are made of ASCII
/// letters and the Spanish written accents; syllables follow the Spanish onsets and the written
/// accents split the spelling ([pa-ís], [dí-a]).
pub struct Spanish<'a, W: WordSource>(pub &'a W);

impl<W: WordSource> FrontEnd for Spanish<'_, W> {
    fn starts(&self, c: char) -> bool {
        c.is_ascii_alphabetic() || c == '\'' || matches!(c, 'á' | 'é' | 'í' | 'ó' | 'ú' | 'ü' | 'ñ' | 'Á' | 'É' | 'Í' | 'Ó' | 'Ú' | 'Ü' | 'Ñ')
    }
    fn continues(&self, c: char) -> bool {
        self.starts(c) || c == '-'
    }
    fn word(&self, tok: &str) -> Vec<Syllable> {
        // typed hyphens split syllables explicitly ("gra-cias"); otherwise the G2P decides
        let parts: Vec<&str> = tok.split('-').filter(|p| !p.is_empty()).collect();
        let mut syls: Vec<Syllable> = Vec::new();
        for part in &parts {
            let sy = syllabify_segments_lang(&self.0.word(&part.to_lowercase(), false), Lang::Spanish);
            let spell: Vec<String> = if parts.len() > 1 {
                let mut v = vec![part.to_string()];
                v.extend(std::iter::repeat_n("~".to_string(), sy.len().saturating_sub(1)));
                v
            } else {
                split_spelling_lang(part, sy.len(), Lang::Spanish)
            };
            for (i, s) in sy.into_iter().enumerate() {
                syls.push(Syllable { text: spell.get(i).cloned().unwrap_or_else(|| "~".into()), onset: s.onset, nucleus: s.nucleus, coda: s.coda, melisma: None, word_end: false, phrase_end: false, ..Default::default() });
            }
        }
        syls
    }
}

enum Token {
    Word(usize, String),
    Mark(char),
}

/// Words (with the index of the front end that claims them), melisma marks and phrase punctuation,
/// in order. For English and Hangul this is
/// `text.match(/[A-Za-z'][A-Za-z'\-]*|[가-힣]+|(?<=\s|^)[~-](?=\s|$)|[.,;:!?\n]/g)`.
fn lyric_tokens(text: &str, front_ends: &[&dyn FrontEnd]) -> Vec<Token> {
    let s: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < s.len() {
        let c = s[i];
        if let Some(k) = front_ends.iter().position(|f| f.starts(c)) {
            let start = i;
            i += 1;
            while i < s.len() && front_ends[k].continues(s[i]) {
                i += 1;
            }
            out.push(Token::Word(k, s[start..i].iter().collect()));
            continue;
        }
        let melisma = (c == '~' || c == '-') && (i == 0 || js_is_space(s[i - 1])) && (i + 1 == s.len() || js_is_space(s[i + 1]));
        if melisma || is_lyric_punct(c) {
            out.push(Token::Mark(c));
        }
        i += 1;
    }
    out
}

/// Lyrics text → syllables, each word read by the first front end that claims it. Characters no
/// front end claims (other scripts, symbols) are skipped.
pub fn read_lyrics(text: &str, front_ends: &[&dyn FrontEnd]) -> Vec<Syllable> {
    let mut out: Vec<Syllable> = Vec::new();
    // the word before this one and its front end, within a phrase
    let mut prev: Option<(usize, String)> = None;
    for tok in lyric_tokens(&text.replace('’', "'"), front_ends) {
        match tok {
            Token::Mark(c) if is_lyric_punct(c) => {
                prev = None;
                if let Some(last) = out.last_mut() {
                    last.phrase_end = true;
                }
            }
            Token::Mark(_) => {
                if let Some(prev) = out.last() {
                    let s = Syllable { text: "~".into(), onset: vec![], nucleus: prev.nucleus.clone(), coda: vec![], melisma: Some(true), word_end: prev.word_end, phrase_end: false, ..Default::default() };
                    out.push(s);
                }
            }
            Token::Word(k, w) => {
                let before = prev.as_ref().filter(|(pk, _)| *pk == k).map(|(_, p)| p.as_str());
                let mut syls = front_ends[k].word_after(&w, before);
                if let Some(last) = syls.last_mut() {
                    last.word_end = true;
                }
                out.append(&mut syls);
                prev = Some((k, w));
            }
        }
    }
    out
}

/// Lyrics text to syllables through the built-in front ends. Latin text follows the selected G2P;
/// Hangul and Cyrillic are routed to their script-specific front ends.
pub fn lyrics_to_syllables(text: &str, g2p: &impl WordSource) -> Vec<Syllable> {
    match g2p.lang() {
        Lang::Spanish => read_lyrics(text, &[&Spanish(g2p), &Korean, &Russian]),
        Lang::English => read_lyrics(text, &[&English(g2p), &Korean, &Russian]),
    }
}
