//! Hebrew lyrics. Everyday Hebrew is written without its vowels, and which vowels a word has
//! depends on the sentence, so no spelling rule can read it. The reading comes from outside the
//! engine: ReNikud (a small neural model the web app runs, `src/song/hebrew.ts`) says each
//! letter's sounds, and this front end turns them into sung syllables.
//!
//! A reading is one IPA chunk per Hebrew letter (א–ת), in the order the letters appear in the
//! lyrics: a consonant, a stress mark and a vowel, any of them missing ("ʃa", "lˈo", "", "m" for
//! שלום). Without readings (the model not loaded, or no model: the CLI) Hebrew words have no
//! sound and are skipped, as a script no front end reads is.
//!
//! Syllables: between two vowels one consonant starts the next syllable and the rest close the
//! one before (mid·bar); a word's first consonants all start its first syllable (ʃtaim). Each
//! syllable's text is the letters whose sounds start in it, and a silent letter stays with the
//! letter before it (שָׁ·לוֹם: ש, לום).
use super::ipa::tokenize_ipa;
use super::syllables::{syllabify_with, FrontEnd};
use super::{Segment, Syllable};
use std::cell::Cell;

pub fn is_letter(c: char) -> bool {
    ('א'..='ת').contains(&c)
}
pub fn contains_hebrew(s: &str) -> bool {
    s.chars().any(is_letter)
}

/// Marks written inside a word: geresh and gershayim (and the apostrophe typed for a geresh),
/// and the vowel points, apart from the maqaf, which joins two words.
fn is_mark(c: char) -> bool {
    matches!(c, '\'' | '׳' | '״') || (('\u{591}'..='\u{5C7}').contains(&c) && c != '־')
}

/// One word and its letters' chunks → syllables. No vowel among the chunks: no syllables.
pub fn word_syllables(word: &str, chunks: &[String]) -> Vec<Syllable> {
    // the word's sounds, and the letter (its index among the word's letters) each belongs to
    let mut segs: Vec<Segment> = Vec::new();
    let mut owner: Vec<usize> = Vec::new();
    for (k, chunk) in chunks.iter().enumerate() {
        for s in tokenize_ipa(chunk) {
            if s.ipa != "_" {
                segs.push(s);
                owner.push(k);
            }
        }
    }
    let parts = syllabify_with(&segs, |cluster| cluster.len() == 1);
    // the syllable each letter's first sound is in
    let mut starts_in: Vec<Option<usize>> = vec![None; chunks.len()];
    let mut i = 0;
    for (n, p) in parts.iter().enumerate() {
        for _ in 0..p.onset.len() + 1 + p.coda.len() {
            starts_in[owner[i]].get_or_insert(n);
            i += 1;
        }
    }
    let mut texts = vec![String::new(); parts.len()];
    let (mut k, mut at) = (0, 0);
    for c in word.chars() {
        if is_letter(c) {
            at = starts_in.get(k).copied().flatten().unwrap_or(at);
            k += 1;
        }
        if let Some(t) = texts.get_mut(at) {
            t.push(c);
        }
    }
    parts
        .into_iter()
        .zip(texts)
        .map(|(p, text)| Syllable { text: if text.is_empty() { "~".into() } else { text }, onset: p.onset, nucleus: p.nucleus, coda: p.coda, ..Default::default() })
        .collect()
}

/// The Hebrew front end for sung lyrics, over the readings of the lyrics it is about to read:
/// each word takes the next chunks, one per letter.
pub struct Hebrew<'a> {
    readings: &'a [String],
    next: Cell<usize>,
}

impl<'a> Hebrew<'a> {
    pub fn new(readings: &'a [String]) -> Self {
        Hebrew { readings, next: Cell::new(0) }
    }
}

impl FrontEnd for Hebrew<'_> {
    fn starts(&self, c: char) -> bool {
        is_letter(c)
    }
    fn continues(&self, c: char) -> bool {
        is_letter(c) || is_mark(c)
    }
    fn word(&self, word: &str) -> Vec<Syllable> {
        let n = word.chars().filter(|&c| is_letter(c)).count();
        let from = self.next.get();
        self.next.set(from + n);
        // readings that run out (they were made for other lyrics) leave the word unread
        self.readings.get(from..from + n).map(|chunks| word_syllables(word, chunks)).unwrap_or_default()
    }
}
