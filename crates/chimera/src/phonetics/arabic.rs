//! Arabic script → articulatory segments and sung syllables.
//! Unvowelled text uses a deliberately small /a/ insertion heuristic; tashkeel is authoritative
//! whenever present. The dialect changes only the consonants whose everyday pronunciations differ.

use super::english::{EnglishG2P, Phrase, expand_numbers};
use super::syllables::{FrontEnd, syllabify_with};
use super::{Intonation, Segment, Syllable};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArabicDialect {
    #[default]
    Msa,
    Egyptian,
}

#[derive(Clone, Copy)]
struct Glyph {
    letter: char,
    vowel: Option<&'static str>,
    sukun: bool,
    shadda: bool,
    tanwin: bool,
    dagger_alif: bool,
}

pub fn is_letter(c: char) -> bool {
    matches!(c, '\u{0621}'..='\u{063A}' | '\u{0641}'..='\u{064A}' | '\u{0671}')
}

fn is_mark(c: char) -> bool {
    matches!(c, '\u{064B}'..='\u{065F}' | '\u{0670}')
}

fn starts(c: char) -> bool {
    is_letter(c)
}

fn continues(c: char) -> bool {
    is_letter(c) || is_mark(c) || c == '\u{0640}'
}

fn glyphs(word: &str) -> Vec<Glyph> {
    let mut out: Vec<Glyph> = Vec::new();
    for c in word.chars() {
        if c == '\u{0640}' {
            continue;
        }
        if is_mark(c) {
            if let Some(g) = out.last_mut() {
                match c {
                    '\u{064E}' | '\u{064B}' => g.vowel = Some("a"),
                    '\u{0650}' | '\u{064D}' => g.vowel = Some("i"),
                    '\u{064F}' | '\u{064C}' => g.vowel = Some("u"),
                    '\u{0652}' => g.sukun = true,
                    '\u{0651}' => g.shadda = true,
                    '\u{0670}' => g.dagger_alif = true,
                    _ => {}
                }
                g.tanwin |= matches!(c, '\u{064B}' | '\u{064C}' | '\u{064D}');
            }
        } else if is_letter(c) {
            out.push(Glyph {
                letter: c,
                vowel: None,
                sukun: false,
                shadda: false,
                tanwin: false,
                dagger_alif: false,
            });
        }
    }
    out
}

fn consonant(c: char, dialect: ArabicDialect) -> Option<&'static str> {
    Some(match c {
        'ء' | 'أ' | 'إ' | 'ؤ' | 'ئ' => "ʔ",
        'ب' => "b",
        'ت' => "t",
        'ث' if dialect == ArabicDialect::Egyptian => "s",
        'ث' => "θ",
        'ج' if dialect == ArabicDialect::Egyptian => "g",
        'ج' => "dʒ",
        'ح' => "ħ",
        'خ' => "x",
        'د' => "d",
        'ذ' if dialect == ArabicDialect::Egyptian => "z",
        'ذ' => "ð",
        'ر' => "ɾ",
        'ز' => "z",
        'س' => "s",
        'ص' => "sˤ",
        'ش' => "ʃ",
        'ض' => "dˤ",
        'ط' => "tˤ",
        'ظ' if dialect == ArabicDialect::Egyptian => "zˤ",
        'ظ' => "ðˤ",
        'ع' => "ʕ",
        'غ' => "ɣ",
        'ف' => "f",
        'ق' if dialect == ArabicDialect::Egyptian => "ʔ",
        'ق' => "q",
        'ك' => "k",
        'ل' => "l",
        'م' => "m",
        'ن' => "n",
        'ه' => "h",
        'و' => "w",
        'ي' => "j",
        'ة' => return None,
        _ => return None,
    })
}

fn is_sun_letter(c: char) -> bool {
    "تثدذرزسشصضطظلن".contains(c)
}

fn is_vowel_segment(s: &Segment) -> bool {
    matches!(s.ipa.as_str(), "a" | "i" | "u")
}

fn push_vowel(out: &mut Vec<Segment>, ipa: &str, long: bool, stress: u8) {
    out.push(Segment {
        ipa: ipa.into(),
        dur: long.then_some(0.18),
        stress: Some(stress),
        ..Default::default()
    });
}

/// One Arabic word to IPA. Explicit short vowels, sukūn, shadda and tanwīn are read from Unicode
/// combining marks; alif/wāw/yāʾ lengthen the preceding matching vowel.
pub fn word_segments(word: &str, dialect: ArabicDialect) -> Vec<Segment> {
    let gs = glyphs(word);
    if gs.is_empty() {
        return Vec::new();
    }
    // A word is explicitly vocalized only when it has vowel marks and its
    // consonant marks do not hide an otherwise missing vowel. Shadda and sukun
    // constrain consonants, but do not supply the vowel in partial input such as
    // "مدّ".
    let vowelled = gs.iter().any(|g| g.vowel.is_some() || g.tanwin)
        && gs
            .iter()
            .filter(|g| is_letter(g.letter) && !matches!(g.letter, 'ا' | 'و' | 'ي' | 'ى'))
            .all(|g| g.vowel.is_some() || g.sukun || g.shadda || g.tanwin);
    let mut out: Vec<Segment> = Vec::with_capacity(gs.len() * 2);

    for (i, g) in gs.iter().enumerate() {
        let next = gs.get(i + 1).map(|n| n.letter);
        let previous = i.checked_sub(1).and_then(|j| gs.get(j)).map(|p| p.letter);
        // Assimilation belongs to the definite article at the start of a word.
        // A lām after an ordinary alif is a real consonant.
        let article_lam = i == 1
            && g.letter == 'ل'
            && previous == Some('ا')
            && gs[0].vowel.is_none()
            && next.is_some_and(is_sun_letter);
        let sun_assimilated =
            i >= 2 && gs[i - 1].letter == 'ل' && gs[i - 2].letter == 'ا' && is_sun_letter(g.letter);
        if article_lam {
            continue;
        }

        let hamza_alif_vowel = match g.letter {
            'أ' if g.vowel.is_none() => Some("a"),
            'إ' if g.vowel.is_none() => Some("i"),
            _ => None,
        };
        let marked_consonant = g.vowel.is_some() || g.shadda || g.sukun || g.tanwin;
        let is_long_waw = g.letter == 'و' && !marked_consonant && out.last().is_some_and(|s| s.ipa == "u");
        let is_long_yaa = g.letter == 'ي' && !marked_consonant && out.last().is_some_and(|s| s.ipa == "i");
        let c = if g.letter == 'ة' || is_long_waw || is_long_yaa {
            None
        } else {
            consonant(g.letter, dialect)
        };

        if let Some(ipa) = c {
            let doubled = g.shadda || sun_assimilated;
            let dur = if doubled { Some(0.12) } else { None };
            out.push(Segment {
                ipa: ipa.into(),
                dur,
                ..Default::default()
            });
        }

        let mut vowel = g.vowel.or(hamza_alif_vowel);
        let mut long = false;
        if g.letter == 'ا' || g.letter == 'آ' || g.letter == 'ى' || g.letter == 'ٱ' {
            if g.letter == 'ا' && previous.is_some_and(|_| gs[i - 1].tanwin) {
                // In common spelling the supporting alif follows the tanwin-bearing consonant;
                // the preceding glyph already supplied the vowel and final /n/.
                continue;
            }
            if g.letter == 'آ' && out.last().is_none_or(|s| s.ipa != "ʔ") {
                out.push(Segment::new("ʔ"));
            }
            if let Some(v) = g.vowel {
                push_vowel(&mut out, v, false, 0);
            } else if i == 0 && next == Some('ل') {
                push_vowel(&mut out, "a", false, 0);
            } else if out
                .last()
                .is_some_and(|s| is_vowel_segment(s) && s.ipa == "a")
            {
                if let Some(last) = out.last_mut() {
                    last.dur = Some(0.18);
                }
            } else {
                push_vowel(&mut out, "a", true, 0);
            }
            if g.tanwin {
                out.push(Segment::new("n"));
            }
            continue;
        }
        if g.dagger_alif {
            if g.vowel == Some("a") {
                push_vowel(&mut out, "a", true, 0);
            } else if let Some(v) = g.vowel {
                push_vowel(&mut out, v, false, 0);
                push_vowel(&mut out, "a", true, 0);
            } else {
                push_vowel(&mut out, "a", true, 0);
            }
            if g.tanwin {
                out.push(Segment::new("n"));
            }
            continue;
        }
        if g.letter == 'و' && out.last().is_some_and(|s| s.ipa == "u") {
            if let Some(last) = out.last_mut() {
                last.dur = Some(0.18);
            }
            continue;
        }
        if g.letter == 'ي' && out.last().is_some_and(|s| s.ipa == "i") {
            if let Some(last) = out.last_mut() {
                last.dur = Some(0.18);
            }
            continue;
        }
        if g.letter == 'ة' {
            // Tāʾ marbūṭa is /a/ in pause, and /at/ before a case vowel.
            if g.vowel.is_some() {
                out.push(Segment::new("t"));
            }
            vowel = Some(g.vowel.unwrap_or("a"));
            long = false;
        }

        if let Some(v) = vowel {
            push_vowel(&mut out, v, long, 0);
            if g.tanwin {
                out.push(Segment::new("n"));
            }
        } else if g.letter == 'و' || g.letter == 'ي' {
            // These are glides unless the preceding short vowel made them long-vowel letters.
            if !g.sukun {
                out.push(Segment::new(if g.letter == 'و' { "w" } else { "j" }));
            }
        } else if !g.sukun
            && !vowelled
            && next.is_some_and(is_letter)
            && !matches!(next, Some('ا' | 'و' | 'ي' | 'ى'))
        {
            // Bare Arabic spelling omits most short vowels. Keep a conservative, predictable
            // fallback; fully vocalized input always takes the explicit-mark path above.
            push_vowel(&mut out, "a", false, 0);
        }
    }

    // Arabic lexical stress is not marked by tashkīl; use the usual final-heavy / penultimate
    // fallback, while explicit short/long vowel marks remain the source of the vowel quality.
    let vowels: Vec<usize> = out
        .iter()
        .enumerate()
        .filter(|(_, s)| is_vowel_segment(s))
        .map(|(i, _)| i)
        .collect();
    if let Some(&last) = vowels.last() {
        let is_heavy = |index: usize| {
            if out[index].dur.is_some() {
                return true;
            }
            let Some(next) = out.get(index + 1) else { return false };
            !is_vowel_segment(next)
                && !out[index + 2..].iter().any(is_vowel_segment)
        };
        let stress = if is_heavy(last) {
            last
        } else if vowels.len() > 1 && is_heavy(vowels[vowels.len() - 2]) {
            vowels[vowels.len() - 2]
        } else if vowels.len() > 2 {
            vowels[vowels.len() - 3]
        } else {
            vowels[0]
        };
        out[stress].stress = Some(1);
        for &i in &vowels {
            if i != stress {
                out[i].stress = Some(0);
                if out[i].dur.is_none() {
                    out[i].dur = Some(0.085);
                }
            }
        }
    }
    for i in 0..out.len() {
        if !matches!(out[i].ipa.as_str(), "sˤ" | "zˤ" | "tˤ" | "dˤ" | "ðˤ") {
            continue;
        }
        if i > 0 && is_vowel_segment(&out[i - 1]) {
            out[i - 1].pharyngealized = Some(true);
        }
        if i + 1 < out.len() && is_vowel_segment(&out[i + 1]) {
            out[i + 1].pharyngealized = Some(true);
        }
    }
    out
}

fn spelling_parts(word: &str, count: usize) -> Vec<String> {
    let mut graphemes: Vec<(char, String)> = Vec::new();
    for c in word.chars() {
        if is_mark(c) {
            if let Some((_, g)) = graphemes.last_mut() {
                g.push(c);
            }
        } else if is_letter(c) {
            graphemes.push((c, c.to_string()));
        }
    }
    if count <= 1 {
        return vec![word.to_string()];
    }
    let has_vocalization = graphemes.iter().any(|(_, g)| {
        g.chars()
            .skip(1)
            .any(|c| matches!(c, '\u{064B}'..='\u{0650}' | '\u{0670}'))
    });
    let mut nuclei: Vec<usize> = graphemes
        .iter()
        .enumerate()
        .filter_map(|(i, (letter, spelling))| {
            let has_vowel_mark = spelling
                .chars()
                .skip(1)
                .any(|c| matches!(c, '\u{064B}'..='\u{0650}' | '\u{0670}'));
            let previous = i.checked_sub(1).map(|p| graphemes[p].1.as_str());
            let is_mater = match letter {
                'ا' | 'آ' | 'ى' => previous.is_some_and(|p| {
                    p.contains('\u{064E}') || p.contains('\u{064B}') || p.contains('\u{0670}')
                }),
                'و' => previous.is_some_and(|p| p.contains('\u{064F}')),
                'ي' => previous.is_some_and(|p| p.contains('\u{0650}')),
                _ => false,
            };
            let long_vowel = matches!(letter, 'ا' | 'أ' | 'إ' | 'آ' | 'ٱ' | 'ى')
                || (!has_vocalization && matches!(letter, 'و' | 'ي') && i > 0);
            (has_vowel_mark || (long_vowel && !is_mater)).then_some(i)
        })
        .collect();

    if nuclei.len() < count && !has_vocalization {
        for i in 0..graphemes.len().saturating_sub(1) {
            let (letter, _) = graphemes[i];
            let next = graphemes[i + 1].0;
            let fallback_vowel = !matches!(letter, 'ا' | 'آ' | 'ى' | 'ٱ' | 'و' | 'ي')
                && !matches!(next, 'ا' | 'و' | 'ي' | 'ى');
            if fallback_vowel && !nuclei.contains(&i) {
                nuclei.push(i);
            }
        }
        nuclei.sort_unstable();
    }

    let mut cuts = vec![0];
    if nuclei.len() >= count {
        for i in 1..count {
            let previous = nuclei[i - 1];
            let next = nuclei[i];
            // Arabic onsets are usually a single consonant; keep the last consonant
            // before a new nucleus with that syllable.
            cuts.push(if next > previous + 1 { next - 1 } else { next });
        }
    } else {
        for i in 1..count {
            cuts.push(i * graphemes.len() / count);
        }
    }
    cuts.push(graphemes.len());
    cuts.windows(2)
        .map(|range| {
            graphemes[range[0]..range[1]]
                .iter()
                .map(|(_, g)| g.as_str())
                .collect()
        })
        .collect()
}

pub struct ArabicFrontEnd(pub ArabicDialect);

impl FrontEnd for ArabicFrontEnd {
    fn starts(&self, c: char) -> bool {
        starts(c)
    }

    fn continues(&self, c: char) -> bool {
        continues(c)
    }

    fn word(&self, word: &str) -> Vec<Syllable> {
        let segs = word_segments(word, self.0);
        let parts = syllabify_with(&segs, |cluster| cluster.len() == 1);
        let spell = spelling_parts(word, parts.len());
        parts
            .into_iter()
            .enumerate()
            .map(|(i, p)| Syllable {
                text: spell.get(i).cloned().unwrap_or_else(|| word.to_string()),
                onset: p.onset,
                nucleus: p.nucleus,
                coda: p.coda,
                ..Default::default()
            })
            .collect()
    }
}

fn is_punct(c: char) -> bool {
    matches!(
        c,
        '.' | ',' | ';' | ':' | '!' | '?' | '،' | '؛' | '؟'
    )
}

/// Mixed-script text-to-speech. Arabic tokens are always sent through the Arabic articulatory
/// front end; Latin tokens keep the existing English G2P and allophone pass.
pub fn text_to_phrases(text: &str, g2p: &EnglishG2P, dialect: ArabicDialect) -> Vec<Phrase> {
    let mut phrases = Vec::new();
    let mut segments = Vec::new();
    let mut words = Vec::new();
    let mut latin = String::new();
    let normalized = expand_numbers(&text.replace('’', "'"));
    let mut chars = normalized.chars().peekable();
    let mut delimiter = String::new();

    let flush = |segments: &mut Vec<Segment>,
                 words: &mut Vec<String>,
                 delimiter: &str,
                 phrases: &mut Vec<Phrase>| {
        if segments.is_empty() {
            return;
        }
        let question = delimiter.contains('?') || delimiter.contains('؟');
        let pause = if delimiter.contains('.') || delimiter.contains('!') || question {
            0.35
        } else if !delimiter.is_empty() {
            0.18
        } else {
            0.3
        };
        phrases.push(Phrase {
            segments: core::mem::take(segments),
            intonation: if question {
                Intonation::Question
            } else {
                Intonation::Statement
            },
            pause,
            words: core::mem::take(words),
        });
    };

    let flush_latin = |latin: &mut String, segments: &mut Vec<Segment>, words: &mut Vec<String>| {
        if latin.trim().is_empty() {
            latin.clear();
            return;
        }
        for phrase in g2p.to_phrases(latin) {
            segments.extend(phrase.segments);
            words.extend(phrase.words);
        }
        latin.clear();
    };

    while let Some(c) = chars.next() {
        if is_punct(c) {
            flush_latin(&mut latin, &mut segments, &mut words);
            delimiter.push(c);
            while chars.peek().is_some_and(|n| is_punct(*n)) {
                delimiter.push(chars.next().unwrap());
            }
            flush(&mut segments, &mut words, &delimiter, &mut phrases);
            delimiter.clear();
            continue;
        }
        if starts(c) {
            flush_latin(&mut latin, &mut segments, &mut words);
            let mut word = String::from(c);
            while chars.peek().is_some_and(|n| continues(*n)) {
                word.push(chars.next().unwrap());
            }
            let mut ws = word_segments(&word, dialect);
            if let Some(last) = ws.last_mut() {
                last.word_end = Some(true);
                segments.append(&mut ws);
                words.push(word);
            }
            continue;
        }
        latin.push(c);
    }
    flush_latin(&mut latin, &mut segments, &mut words);
    flush(&mut segments, &mut words, "", &mut phrases);
    phrases
}
