//! Spanish text → phoneme segments: a rule-based G2P with no dictionary. Orthographic rules
//! give the phonemes, the written accent rules give the stress, and Spanish syllabification
//! (shared code, `syllables.rs`) plus the spelling split serve singing. English keeps its own
//! front-end (`english.rs`); the two share the planner, the syllables and [`Phrase`].
//!
//! Dialect is switches on [`Dialect`] (the UI exposes presets: neutro, rioplatense,
//! castellano). The default is neutral Latin-American: seseo, yeísmo, /s/ kept in the coda.
//!
//! Not modelled (each a deliberate scope cut, revisit if the ear asks for it): number words
//! (digit runs say nothing yet, as `english.rs` spells them), verb forms like the "ría" of
//! *reír* (read as the glide [ja] of *riar* — written accents are the only hiatus signal),
//! and cross-word s–voicing beyond the same word.

use super::english::{is_phrase_punct, split_keeping};
use super::syllables::{Lang, WordSource};
use super::{Intonation, Phrase, Segment};
use serde::{Deserialize, Serialize};

/// Dialect switches (UI presets map onto these three).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Dialect {
    /// /z/ and soft /c/ say [s] (true, seseo) or [θ] (false, distinción castellana)
    pub seseo: bool,
    /// /ll/ and consonantal /y/: [ʝ] (jeísmo), [ʃ] (sheísmo), [ʒ] (zheísmo) or [ʎ] (lleísta)
    pub ll: Ll,
    /// syllable-final /s/ breathes ([ˈmihmo]) instead of staying [s] (rioplatense)
    pub aspirated_s: bool,
}

/// The /ll/ y /y/ consonantal treatment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ll {
    /// [ʝ] — standard yeísmo (most of the Spanish-speaking world)
    #[default]
    Jeismo,
    /// [ʃ] — sheísmo rioplatense
    Sheismo,
    /// [ʒ] — the zheísmo variant of the Río de la Plata
    Zheismo,
    /// [ʎ] — the palatal lateral (distinción lleísta)
    Lleista,
}

impl Default for Dialect {
    fn default() -> Dialect {
        Dialect { seseo: true, ll: Ll::Jeismo, aspirated_s: false }
    }
}

impl Ll {
    fn phone(self) -> &'static str {
        match self {
            Ll::Jeismo => "ʝ",
            Ll::Sheismo => "ʃ",
            Ll::Zheismo => "ʒ",
            Ll::Lleista => "ʎ",
        }
    }
}

/// Neutral Latin-American (the default).
pub const NEUTRO: Dialect = Dialect { seseo: true, ll: Ll::Jeismo, aspirated_s: false };
/// Rioplatense: sheísmo and aspirated coda /s/.
pub const RIOPLATENSE: Dialect = Dialect { seseo: true, ll: Ll::Sheismo, aspirated_s: true };
/// Castellano: distinción de /θ/.
pub const CASTELLANO: Dialect = Dialect { seseo: false, ll: Ll::Jeismo, aspirated_s: false };

#[derive(Clone, Debug, Default)]
pub struct SpanishG2P {
    dialect: Dialect,
}

impl SpanishG2P {
    pub fn new() -> SpanishG2P {
        SpanishG2P::default()
    }

    pub fn set_dialect(&mut self, d: Dialect) {
        self.dialect = d;
    }

    pub fn dialect(&self) -> Dialect {
        self.dialect
    }

    /// Text → phrases of segments (phrase punctuation and pauses as in English).
    pub fn to_phrases(&self, text: &str) -> Vec<Phrase> {
        let mut phrases: Vec<Phrase> = Vec::new();
        let normalized = text.replace('’', "'");
        let parts = split_keeping(&normalized, is_phrase_punct);
        let mut words: Vec<String> = Vec::new();
        let mut flush = |delim: &str, words: &mut Vec<String>| {
            if words.is_empty() {
                return;
            }
            let mut segs: Vec<Segment> = Vec::new();
            // a token with no sound (digits, "--") is left out, so the n-th word ends at the
            // n-th word end
            let mut said: Vec<String> = Vec::new();
            for w in words.iter() {
                let mut ws = self.word(w, false);
                let Some(last) = ws.last_mut() else { continue };
                last.word_end = Some(true);
                segs.append(&mut ws);
                said.push(w.clone());
            }
            apply_allophones(&self.dialect, &mut segs);
            let q = delim.contains('?');
            let pause = if delim.contains(['.', '!', '?']) {
                0.35
            } else if !delim.is_empty() {
                0.18
            } else {
                0.3
            };
            words.clear();
            if !segs.is_empty() {
                phrases.push(Phrase { segments: segs, intonation: if q { Intonation::Question } else { Intonation::Statement }, pause, words: said });
            }
        };
        for part in &parts {
            if !part.is_empty() && part.chars().all(is_phrase_punct) {
                flush(part, &mut words);
            } else {
                // one word = a run of Spanish letters (lowercase here: the text was lowercased);
                // digits say nothing yet (number words are a later step)
                let lower = part.to_lowercase();
                words.extend(lower.split(|c: char| !is_word_char(c)).filter(|w| !w.is_empty()).map(str::to_string));
            }
        }
        flush("", &mut words);
        phrases
    }

    /// One word → segments with lexical stress. `reduce` is the trait's parameter: Spanish has
    /// no vowel reduction — function words simply take no stress (and keep full vowels).
    pub fn word(&self, w: &str, _reduce: bool) -> Vec<Segment> {
        let lower = w.to_lowercase();
        let clean = lower.trim_matches('\'');
        if clean.is_empty() {
            return Vec::new();
        }
        // hyphenated compounds say each part
        if clean.contains('-') {
            return clean.split('-').filter(|p| !p.is_empty()).flat_map(|p| self.word(p, false)).collect();
        }
        // "muy" is the one stressed [uj]: the weak+weak rule (below) would glide it
        if clean == "muy" {
            return vec![Segment { ipa: "m".into(), stress: Some(0), ..Default::default() }, Segment { ipa: "ui".into(), stress: Some(1), ..Default::default() }];
        }
        let mut pieces = scan(clean, &self.dialect);
        merge_vowel_runs(&mut pieces);
        let stress = pick_stress(clean, &pieces);
        let mut nucleus = 0usize;
        pieces
            .into_iter()
            .map(|p| {
                let (ipa, stress) = match p {
                    Piece::C(c) => (c.to_string(), 0),
                    Piece::V(v, accent) => {
                        let s = stress.get(nucleus).copied().unwrap_or(0);
                        nucleus += 1;
                        (v.to_string(), if accent { 1 } else { s })
                    }
                };
                Segment { ipa, stress: Some(stress), ..Default::default() }
            })
            .collect()
    }
}

impl WordSource for SpanishG2P {
    fn word(&self, w: &str, reduce: bool) -> Vec<Segment> {
        SpanishG2P::word(self, w, reduce)
    }

    fn lang(&self) -> Lang {
        Lang::Spanish
    }
}

// ---------------------------------------------------------------- scanning

/// A word piece: a consonant phoneme, or a vowel nucleus (a written accent marks stress).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Piece {
    C(&'static str),
    /// vowel, and whether the written accent marks it as the stressed one
    V(&'static str, bool),
}

/// `word.split(/[a-záéíóúüñ]+/)`: the characters one Spanish word is made of.
fn is_word_char(c: char) -> bool {
    c.is_ascii_lowercase() || matches!(c, 'á' | 'é' | 'í' | 'ó' | 'ú' | 'ü' | 'ñ')
}

fn is_weak(v: &str) -> bool {
    v == "i" || v == "u"
}

fn glide(v: &str) -> &'static str {
    if v == "i" { "j" } else { "w" }
}

/// falling diphthong of a strong vowel + a weak offglide
fn diphthong(v1: &str, v2: &str) -> Option<&'static str> {
    Some(match (v1, v2) {
        ("a", "i") => "ai",
        ("a", "u") => "au",
        ("e", "i") => "ei",
        ("e", "u") => "eu",
        ("o", "i") => "oi",
        ("o", "u") => "ou",
        _ => return None,
    })
}

/// Graphemes → pieces (diphthong tokens and glides are `merge_vowel_runs`'s job; here every
/// vowel letter is its own `Piece::V`).
fn scan(word: &str, d: &Dialect) -> Vec<Piece> {
    let s: Vec<char> = word.chars().collect();
    let n = s.len();
    let soft = |k: usize| matches!(s.get(k), Some('e') | Some('i'));
    let vowel = |k: usize| matches!(s.get(k), Some('a') | Some('e') | Some('i') | Some('o') | Some('u') | Some('á') | Some('é') | Some('í') | Some('ó') | Some('ú'));
    let mut out: Vec<Piece> = Vec::new();
    let mut i = 0;
    while i < n {
        let c = s[i];
        match c {
            // digraphs
            'c' if s.get(i + 1) == Some(&'h') => {
                out.push(Piece::C("tʃ"));
                i += 2;
                continue;
            }
            'l' if s.get(i + 1) == Some(&'l') => {
                out.push(Piece::C(d.ll.phone()));
                i += 2;
                continue;
            }
            'r' if s.get(i + 1) == Some(&'r') => {
                out.push(Piece::C("r"));
                i += 2;
                continue;
            }
            'q' => {
                // "qu" + e/i = [k] (u silent, accented "quí"/"qué" too); "qu" + a/o = [kw]
                // ("quota"); a lone q is [k]
                match s.get(i + 1) {
                    Some('u') => match s.get(i + 2) {
                        // the u is silent; the vowel itself is scanned next (accented ones too)
                        Some('e') | Some('i') | Some('é') | Some('í') => {
                            out.push(Piece::C("k"));
                            i += 2;
                        }
                        Some('a') | Some('o') => {
                            out.push(Piece::C("k"));
                            out.push(Piece::C("w"));
                            i += 2;
                        }
                        _ => {
                            out.push(Piece::C("k"));
                            out.push(Piece::V("u", false));
                            i += 2;
                        }
                    },
                    _ => {
                        out.push(Piece::C("k"));
                        i += 1;
                    }
                }
                continue;
            }
            'g' if s.get(i + 1) == Some(&'u') => {
                // "gu" + e/i = [g] (u silent, accented too); "gü" = [gw]; "gu" + a/o/u = [gw]
                match s.get(i + 2) {
                    // the u is silent; the vowel itself is scanned next (accented ones too)
                    Some('e') | Some('i') | Some('é') | Some('í') => {
                        out.push(Piece::C("g"));
                        i += 2;
                    }
                    Some('ü') => {
                        out.push(Piece::C("g"));
                        out.push(Piece::C("w"));
                        i += 3;
                    }
                    Some('a') | Some('o') | Some('u') => {
                        out.push(Piece::C("g"));
                        out.push(Piece::C("w"));
                        i += 2;
                    }
                    _ => {
                        out.push(Piece::C("g"));
                        out.push(Piece::V("u", false));
                        i += 2;
                    }
                }
                continue;
            }
            // single letters
            'b' | 'v' => out.push(Piece::C("b")),
            'f' => out.push(Piece::C("f")),
            'm' => out.push(Piece::C("m")),
            'n' => out.push(Piece::C("n")),
            'p' => out.push(Piece::C("p")),
            't' => out.push(Piece::C("t")),
            'd' => out.push(Piece::C("d")),
            'l' => out.push(Piece::C("l")),
            's' => out.push(Piece::C("s")),
            'k' | 'w' => out.push(Piece::C(if c == 'k' { "k" } else { "w" })),
            'z' => out.push(Piece::C(if d.seseo { "s" } else { "θ" })),
            'h' => {} // silent
            'j' => out.push(Piece::C("x")),
            'x' => {
                // x = [ks]: e-xa-men (the cluster is an onset, see `syllables::ONSETS_ES`)
                out.push(Piece::C("k"));
                out.push(Piece::C("s"));
            }
            'ñ' => out.push(Piece::C("ɲ")),
            'c' => out.push(Piece::C(if soft(i + 1) { if d.seseo { "s" } else { "θ" } } else { "k" })),
            'g' => out.push(Piece::C(if soft(i + 1) { "x" } else { "g" })),
            'r' => {
                // trill at word start and after n, l, s; a tap elsewhere
                let prev = if i == 0 { None } else { Some(s[i - 1]) };
                let strong_ctx = i == 0 || matches!(prev, Some('n') | Some('l') | Some('s'));
                out.push(Piece::C(if strong_ctx { "r" } else { "ɾ" }));
            }
            'y' => {
                // before a vowel it is the consonant of a rising diphthong; at the end or
                // before a consonant it is the vowel [i]
                if vowel(i + 1) {
                    out.push(Piece::C(d.ll.phone()));
                } else {
                    out.push(Piece::V("i", false));
                }
            }
            'a' => out.push(Piece::V("a", false)),
            'e' => out.push(Piece::V("e", false)),
            'o' => out.push(Piece::V("o", false)),
            'i' => out.push(Piece::V("i", false)),
            'u' => out.push(Piece::V("u", false)),
            'á' => out.push(Piece::V("a", true)),
            'é' => out.push(Piece::V("e", true)),
            'í' => out.push(Piece::V("i", true)),
            'ó' => out.push(Piece::V("o", true)),
            'ú' => out.push(Piece::V("u", true)),
            'ü' => out.push(Piece::C("w")), // not after g (there "gü" is scanned): a glide
            _ => {}                         // unknown characters say nothing
        }
        i += 1;
    }
    out
}

/// Adjacent vowels become diphthong tokens and glides (one pass, left to right):
/// a weak unstressed vowel before another vowel is a rising glide ([ˈaβɾja], [kwi-do]); a
/// strong vowel + weak unstressed vowel is a falling diphthong token ([ˈaj-ɾe]); accented
/// weak vowels and strong+strong stay separate nuclei in hiatus ([pa-ís], [le-er]).
fn merge_vowel_runs(pieces: &mut Vec<Piece>) {
    let mut out: Vec<Piece> = Vec::with_capacity(pieces.len());
    let mut k = 0;
    while k < pieces.len() {
        let p = pieces[k];
        if !matches!(p, Piece::V(..)) {
            out.push(p);
            k += 1;
            continue;
        }
        // the maximal run of adjacent vowel pieces
        let mut m = k;
        while m < pieces.len() && matches!(pieces[m], Piece::V(..)) {
            m += 1;
        }
        let mut idx = k;
        while idx < m {
            if let Piece::V(v1, acc1) = pieces[idx] {
                if let Some(Piece::V(v2, acc2)) = pieces.get(idx + 1) {
                    // a weak unstressed first vowel is a glide before anything ("día" keeps
                    // its accented í out of this rule by `acc1`)
                    if is_weak(v1) && !acc1 {
                        out.push(Piece::C(glide(v1)));
                        idx += 1;
                        continue;
                    }
                    if is_weak(v2) && !acc2 {
                        // strong + weak → falling diphthong (the accent lives on the strong vowel)
                        if let Some(t) = diphthong(v1, v2) {
                            out.push(Piece::V(t, acc1));
                            idx += 2;
                            continue;
                        }
                    }
                }
                out.push(pieces[idx]);
                idx += 1;
            } else {
                idx += 1;
            }
        }
        k = m;
    }
    *pieces = out;
}

// ---------------------------------------------------------------- stress

/// monosyllabic function words take no stress (and keep their full vowel); written-accented
/// forms ("mí", "sí", "qué") are not in the list, so they stay stressed
const FUNCTION_WORDS: &str = "a o u y e el al del la las le les lo los me mi se su te tu tu tus sus nos os un una uno unas unos que si yo en con de por para sin sobre entre tras desde hasta según tan ni fue fui dio vio vi di es";

fn is_function_word(w: &str) -> bool {
    FUNCTION_WORDS.split(' ').any(|f| f == w)
}

/// One stress flag per nucleus: an accent wins; a word ending in a vowel letter, n or s
/// (orthographic — "raíz" ends in z, so it stays aguda) stresses the penult, the rest the last
/// syllable; monosyllabic content words stress their only vowel; function words none.
fn pick_stress(clean: &str, pieces: &[Piece]) -> Vec<u8> {
    let n = pieces.iter().filter(|p| matches!(p, Piece::V(..))).count();
    let mut out = vec![0u8; n];
    if n == 0 {
        return out;
    }
    // a written accent marks its syllable ("país", "púa", "mía", "qué")
    let mut idx = 0;
    for p in pieces {
        if let Piece::V(_, accent) = p {
            if *accent {
                out[idx] = 1;
                return out;
            }
            idx += 1;
        }
    }
    if is_function_word(clean) {
        return out;
    }
    if n == 1 {
        out[0] = 1;
        return out;
    }
    let last = clean.chars().last().unwrap_or('a');
    let open = matches!(last, 'a' | 'e' | 'i' | 'o' | 'u' | 'ü' | 'n' | 's');
    let which = if open { n - 2 } else { n - 1 };
    out[which] = 1;
    out
}

// ---------------------------------------------------------------- allophones

/// the vowels and diphthong tokens the G2P emits
fn is_es_vowel(ipa: &str) -> bool {
    matches!(ipa, "a" | "e" | "i" | "o" | "u" | "ai" | "au" | "ei" | "oi" | "eu" | "ou" | "ui")
}

/// consonants whose voicing assimilates a preceding /s/
fn voiced_consonant(ipa: &str) -> bool {
    matches!(ipa, "b" | "d" | "g" | "β" | "ð" | "ɣ" | "m" | "n" | "ɲ" | "l" | "r" | "ɾ" | "j" | "w" | "ʝ" | "ʃ" | "ʒ")
}

/// continuants after which a stop becomes a fricative ([ˈaɣwa], [ˈbeɾðe], [ˈalβa], [ˈdezðe])
fn fricative_ctx(ipa: &str) -> bool {
    is_es_vowel(ipa) || matches!(ipa, "l" | "r" | "ɾ" | "z")
}

/// Panhispanic allophones, in place:
/// - /b d g/ are fricatives after a vowel or a continuant ([la βoka], [eɾ ðedo], [aɣwa]), stops
///   after a pause or a nasal ([en boca], [tengo]);
/// - /s/ voices before a voiced consonant ([ˈmizmo]); with `aspirated_s` the coda /s/ breathes;
/// - /n/ velarises before k, g, x ([ˈbaŋko], [ˈaŋtʃo]).
fn apply_allophones(d: &Dialect, segs: &mut [Segment]) {
    for i in 0..segs.len() {
        let (prev, next) = (
            if i == 0 { None } else { Some(segs[i - 1].ipa.clone()) },
            segs.get(i + 1).map(|s| s.ipa.clone()),
        );
        let s = &mut segs[i];
        match s.ipa.as_str() {
            "b" | "d" | "g" => {
                if prev.as_deref().is_some_and(fricative_ctx) {
                    s.ipa = match s.ipa.as_str() {
                        "b" => "β".into(),
                        "d" => "ð".into(),
                        _ => "ɣ".into(),
                    };
                }
            }
            "n" if next.as_deref().is_some_and(|n| matches!(n, "k" | "g" | "x")) => s.ipa = "ŋ".into(),
            "s" => {
                // coda: a consonant follows (a glide before a vowel is an onset, not a coda),
                // or the word ends here
                let coda = next
                    .as_deref()
                    .is_some_and(|n| !is_es_vowel(n) && n != "j" && n != "w")
                    || s.word_end == Some(true);
                if coda {
                    if d.aspirated_s {
                        s.ipa = "h".into();
                    } else if next.as_deref().is_some_and(voiced_consonant) && s.word_end != Some(true) {
                        s.ipa = "z".into();
                    }
                }
            }
            _ => {}
        }
    }
}
