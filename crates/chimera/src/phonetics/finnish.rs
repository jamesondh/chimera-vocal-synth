//! Finnish text → phoneme segments and sung syllables.
//!
//! Finnish orthography is nearly phonemic, so no dictionary is needed:
//! - Primary stress always falls on the first syllable of a word; secondary stress falls on odd
//!   non-final syllables (`idx = 2, 4, …`). Unstressed vowels do not reduce.
//! - Voiceless stops `/p t̪ k/` use full glottal abduction during closure with a tight voice onset
//!   time (`aspirated: Some(true), vot: Some(0.012)`): `/t/` uses the dental closure `t̪` (like
//!   Finnish, Swedish, and Japanese) rather than English's alveolar `/t/`, and the tight 12 ms VOT
//!   snaps the glottis shut right on release (no `[d]` voicing bleed and no English `[tʰ]` puff).
//! - Doubled vowels (`aa ee ii oo uu yy ää öö`) are single long vowels (`LONG_VOWEL = 1.40`
//!   stressed vs `SHORT_VOWEL_STRESSED = 0.55`, ~2.5× contrast); doubled consonants
//!   (`rr ll tt pp kk ss nn mm hh`) are single long geminates (`GEMINATE = 2.2` in speech;
//!   consolidated into a single lengthened onset `155–175 ms` in singing).
//! - `ng` between vowels is a long velar nasal `[ŋː]`, `ng` in a coda cluster is short `[ŋ]`, and
//!   `nk` is `[ŋk]`.
//! - All 18 standard Finnish diphthongs stay in one syllable:
//!   - Closing diphthongs (`ai au ei ou oi äi öi ui yi äy öy eu iu ey iy`) use calibrated Finnish
//!     vowel targets (`ɑ̠i ɑ̠u̝ eɪ ou̝ oi æ̞i œi u̝i yi æ̞y œy eu̝ iu̝ ey iy`).
//!   - First-syllable opening diphthongs `ie uo yö` map to `ie` (`i→ɛ`), `ʊɔ` (`u̝→ɔ`), `yœ`
//!     (`y→œ`) with `offglide_share: Some(0.60)` in singing; non-initial `ie uo yö` (`tähtien`)
//!     split across syllables (`täh-ti-en`).

use super::english::{Phrase, is_phrase_punct, split_keeping};
use super::mandarin::MandarinLexicon;
use super::syllables::{FrontEnd, Lang, lyrics_in};
use super::{Intonation, Segment, Syllable};

/// Spoken duration multiplier for a doubled Finnish consonant (`kk`, `tt`, `rr`, `ll`, …).
const GEMINATE: f64 = 2.2;
/// Spoken duration multipliers for single (short) Finnish vowels:
/// - Klatt's English stressed vowels are 210–240 ms inherent, so `0.55` brings a stressed short
///   Finnish vowel to ~80–90 ms (giving a crisp ~2.5× contrast against `LONG_VOWEL = 1.40`).
/// - Klatt already halves medial unstressed English vowels (schwa reduction); Finnish does not
///   reduce unstressed vowels, so medial unstressed short vowels stay near `1.0` (~75–85 ms)
///   instead of being crushed to 40 ms (`lämpöä` → `lämpyä`, `tähtönen` → `tähtnen`).
const SHORT_VOWEL_STRESSED: f64 = 0.55;
const SHORT_VOWEL_MEDIAL: f64 = 1.00;
const SHORT_VOWEL_FINAL: f64 = 0.60;
/// Spoken duration multiplier for a doubled Finnish vowel (`aa`, `ee`, `uu`, `ää`, …): ~2.5× a
/// short vowel (`0.55` vs `1.40`) so `tuli / tuuli / tulli` contrast drastically.
const LONG_VOWEL: f64 = 1.40;
/// Spoken duration multiplier for a Finnish diphthong (enough room for both vowel targets,
/// especially open→close `au` in `laulu` where the lips travel from 2.0 down to 0.14).
const DIPHTHONG_LEN: f64 = 1.35;

fn is_letter(c: char) -> bool {
    let l = c.to_lowercase().next().unwrap_or(c);
    l.is_ascii_lowercase() || matches!(l, 'ä' | 'ö' | 'å' | 'š' | 'ž')
}

fn is_vowel_letter(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'ä' | 'ö' | 'å')
}

fn is_opening_diphthong(a: char, b: char) -> bool {
    matches!((a, b), ('i', 'e') | ('u', 'o') | ('y', 'ö'))
}

fn is_closing_diphthong(a: char, b: char) -> bool {
    matches!(
        (a, b),
        ('a', 'i')
            | ('e', 'i')
            | ('o', 'i')
            | ('u', 'i')
            | ('y', 'i')
            | ('ä', 'i')
            | ('ö', 'i')
            | ('a', 'u')
            | ('e', 'u')
            | ('i', 'u')
            | ('o', 'u')
            | ('e', 'y')
            | ('i', 'y')
            | ('ä', 'y')
            | ('ö', 'y')
    )
}

fn single_vowel_ipa(c: char) -> &'static str {
    match c {
        'a' => "ɑ̠",
        'ä' => "æ̞",
        'ö' => "œ",
        'å' => "o",
        'e' => "e",
        'i' => "i",
        'o' => "o",
        'u' => "u̝",
        'y' => "y",
        _ => "ɑ̠",
    }
}

/// Returns `(ipa, offglide_share)` for a 2-letter Finnish diphthong.
fn diphthong_ipa(a: char, b: char) -> Option<(&'static str, Option<f64>)> {
    Some(match (a, b) {
        // Finnish falling diphthongs with pure Finnish endpoints (avoiding English lax ɪ/ʊ and fronted u)
        ('a', 'i') => ("ɑ̠i", Some(0.45)),
        ('a', 'u') => ("ɑ̠u̝", Some(0.52)),
        ('e', 'i') => ("eɪ", Some(0.42)),
        ('o', 'u') => ("ou̝", Some(0.50)),
        ('o', 'i') => ("oi", Some(0.45)),
        // Opening diphthongs: spend ~60% of the sung note on the more open second half
        ('i', 'e') => ("ie", Some(0.60)),
        ('u', 'o') => ("ʊɔ", Some(0.60)),
        ('y', 'ö') => ("yœ", Some(0.60)),
        // Remaining Finnish closing/front diphthongs
        ('ä', 'i') => ("æ̞i", Some(0.42)),
        ('ö', 'i') => ("œi", Some(0.42)),
        ('u', 'i') => ("u̝i", Some(0.42)),
        ('y', 'i') => ("yi", Some(0.42)),
        ('ä', 'y') => ("æ̞y", Some(0.42)),
        ('ö', 'y') => ("œy", Some(0.42)),
        ('e', 'u') => ("eu̝", Some(0.42)),
        ('i', 'u') => ("iu̝", Some(0.42)),
        ('e', 'y') => ("ey", Some(0.42)),
        ('i', 'y') => ("iy", Some(0.42)),
        _ => return None,
    })
}

/// Lengthened sung onset duration (s) for a geminate consonant across a syllable boundary.
fn sung_geminate_dur(ipa: &str) -> Option<f64> {
    Some(match ipa {
        "r" => 0.165,
        "l" | "l̪" => 0.175,
        "s" => 0.160,
        "t" | "t̪" | "p" | "k" | "h" => 0.155,
        "n" | "m" => 0.155,
        _ => return None,
    })
}

/// Tight voice onset time (s) for Finnish voiceless stops `/p t̪ k/`: opens the glottis cleanly
/// during closure (`ABDUCTION = 0.36`, avoiding `[d]`-like voicing bleed) and tightens the
/// closing ramp (`18 ms` in `planner.rs`) so there is no English `[tʰ]` breathy tail.
const STOP_VOT: f64 = 0.012;

fn seg_consonant(ipa: &str) -> Segment {
    let stop = matches!(ipa, "p" | "t" | "t̪" | "k");
    Segment {
        ipa: ipa.to_string(),
        aspirated: stop.then_some(true),
        vot: stop.then_some(STOP_VOT),
        ..Default::default()
    }
}

fn consonants_to_segments(cs: &[char]) -> Vec<Segment> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < cs.len() {
        if i + 1 < cs.len() {
            match (cs[i], cs[i + 1]) {
                ('n', 'g') => {
                    // Within a single cluster (`englanti`, `ping`) `ng` is short [ŋ]; intervocalic
                    // `ng` (`rengas`, `langat`) becomes geminate [ŋː] in `word()`'s cross-syllable merge.
                    out.push(seg_consonant("ŋ"));
                    i += 2;
                    continue;
                }
                ('n', 'k') => {
                    out.push(seg_consonant("ŋ"));
                    out.push(seg_consonant("k"));
                    i += 2;
                    continue;
                }
                ('s', 'h') => {
                    out.push(seg_consonant("ʃ"));
                    i += 2;
                    continue;
                }
                (a, b) if a == b && !is_vowel_letter(a) => {
                    for mut s in single_consonant(a) {
                        s.length = Some(GEMINATE);
                        out.push(s);
                    }
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        out.extend(single_consonant(cs[i]));
        i += 1;
    }
    out
}

fn single_consonant(c: char) -> Vec<Segment> {
    match c {
        'p' => vec![seg_consonant("p")],
        't' => vec![seg_consonant("t̪")],
        'k' | 'c' | 'q' => vec![seg_consonant("k")],
        'x' => vec![seg_consonant("k"), seg_consonant("s")],
        'z' => vec![seg_consonant("t̪"), seg_consonant("s")],
        'w' | 'v' => vec![seg_consonant("v")],
        'š' => vec![seg_consonant("ʃ")],
        'ž' => vec![seg_consonant("ʒ")],
        'b' => vec![seg_consonant("b")],
        'd' => vec![seg_consonant("d")],
        'f' => vec![seg_consonant("f")],
        'g' => vec![seg_consonant("g")],
        'h' => vec![seg_consonant("h")],
        'j' => vec![seg_consonant("j")],
        'l' => vec![seg_consonant("l̪")],
        'm' => vec![seg_consonant("m")],
        'n' => vec![seg_consonant("n")],
        'r' => vec![seg_consonant("r")],
        's' => vec![seg_consonant("s")],
        _ => Vec::new(),
    }
}

struct RawSyllable {
    text: String,
    onset: Vec<char>,
    nucleus: Vec<char>,
    coda: Vec<char>,
}

/// Splits a Finnish word into orthographic syllables (preserving original casing in `text`).
/// An apostrophe (`vaa'an`, `ha'un`, `lu'un`) forces a syllable break between identical/hiatus
/// vowels without splitting the word.
fn split_syllables(typed: &str) -> Vec<RawSyllable> {
    let mut syls: Vec<RawSyllable> = Vec::new();
    for (chunk_idx, chunk) in typed.replace('’', "'").split('\'').enumerate() {
        let orig: Vec<char> = chunk.chars().filter(|c| is_letter(*c)).collect();
        let lower: Vec<char> = orig.iter().map(|c| c.to_lowercase().next().unwrap_or(*c)).collect();
        if lower.is_empty() {
            continue;
        }
        let chunk_start = syls.len();
        let mut i = 0;
        while i < lower.len() {
            let start = i;
            let mut onset = Vec::new();
            while i < lower.len() && !is_vowel_letter(lower[i]) {
                onset.push(lower[i]);
                i += 1;
            }
            if i >= lower.len() {
                if let Some(last) = syls.last_mut() {
                    last.coda.extend(onset);
                    last.text.extend(&orig[start..]);
                }
                break;
            }
            let mut nucleus = vec![lower[i]];
            i += 1;
            if i < lower.len() && is_vowel_letter(lower[i]) {
                let v1 = nucleus[0];
                let v2 = lower[i];
                // Opening diphthongs `ie / uo / yö` only stay in one syllable in the first syllable
                // of a word (`kierros`, `Suomi`, `yöllä`); in non-initial syllables (`tähtien`,
                // `pilvien`, `filippiinien`), they split across a syllable boundary (`täh-ti-en`).
                let diph = is_closing_diphthong(v1, v2) || (syls.is_empty() && is_opening_diphthong(v1, v2));
                if v2 == v1 || diph {
                    nucleus.push(v2);
                    i += 1;
                }
            }
            let mut c_end = i;
            while c_end < lower.len() && !is_vowel_letter(lower[c_end]) {
                c_end += 1;
            }
            let coda = if c_end == lower.len() {
                let c = lower[i..c_end].to_vec();
                i = c_end;
                c
            } else if c_end - i >= 2 {
                let c = lower[i..c_end - 1].to_vec();
                i = c_end - 1;
                c
            } else {
                Vec::new()
            };
            let mut text: String = orig[start..i].iter().collect();
            if chunk_idx > 0 && syls.len() == chunk_start {
                text.insert(0, '\'');
            }
            syls.push(RawSyllable { text, onset, nucleus, coda });
        }
    }
    syls
}

fn syllable_stress(idx: usize, len: usize) -> u8 {
    if idx == 0 {
        1
    } else if idx.is_multiple_of(2) && idx + 1 < len {
        2
    } else {
        0
    }
}

/// One Finnish word to spoken segments with primary/secondary stress, long vowels, and geminates.
pub fn word(typed: &str) -> Vec<Segment> {
    let syls = split_syllables(typed);
    if syls.is_empty() {
        return Vec::new();
    }
    let len = syls.len();
    let mut out: Vec<Segment> = Vec::new();
    for (idx, syl) in syls.iter().enumerate() {
        let stress = syllable_stress(idx, len);
        let mut onset = consonants_to_segments(&syl.onset);
        // Merge cross-syllable geminates and ng/nk into a single spoken consonant
        if let (Some(prev), Some(first)) = (out.last_mut(), onset.first()) {
            if prev.ipa == "n" && first.ipa == "g" {
                prev.ipa = "ŋ".into();
                prev.length = Some(GEMINATE);
                onset.remove(0);
            } else if prev.ipa == "n" && first.ipa == "k" {
                prev.ipa = "ŋ".into();
            } else if prev.ipa == first.ipa && prev.stress.is_none() {
                prev.length = Some(GEMINATE);
                onset.remove(0);
            }
        }
        out.extend(onset);
        let short_scale = if stress == 1 {
            SHORT_VOWEL_STRESSED
        } else if idx + 1 < len {
            SHORT_VOWEL_MEDIAL
        } else {
            SHORT_VOWEL_FINAL
        };
        let (ipa, len_scale) = match syl.nucleus.as_slice() {
            [a] => (single_vowel_ipa(*a), short_scale),
            [a, b] if a == b => (single_vowel_ipa(*a), if stress == 1 { LONG_VOWEL } else { LONG_VOWEL * 1.25 }),
            [a, b] => (diphthong_ipa(*a, *b).map_or_else(|| single_vowel_ipa(*a), |(d, _)| d), DIPHTHONG_LEN),
            _ => ("ɑ̠", short_scale),
        };
        out.push(Segment { ipa: ipa.into(), stress: Some(stress), length: Some(len_scale), ..Default::default() });
        let mut coda = consonants_to_segments(&syl.coda);
        // Coda `r` before a following onset consonant (`turma`, `perhe`, `kerta`) gets enough
        // room to strike clearly before the next consonant's closure (e.g. `m`'s early lip seal).
        if idx + 1 < len
            && let Some(last) = coda.last_mut()
            && last.ipa == "r"
            && last.length.is_none()
        {
            last.length = Some(1.45);
        }
        out.extend(coda);
    }
    out
}

/// One Finnish word's sung syllables, consolidating cross-syllable geminates and `ng`/`nk`.
fn sung_syllables(typed: &str) -> Vec<Syllable> {
    let raw = split_syllables(typed);
    let len = raw.len();
    let mut out: Vec<Syllable> = raw
        .iter()
        .enumerate()
        .map(|(idx, syl)| {
            let stress = syllable_stress(idx, len);
            let (ipa, offglide_share) = match syl.nucleus.as_slice() {
                [a] => (single_vowel_ipa(*a), None),
                [a, b] if a == b => (single_vowel_ipa(*a), None),
                [a, b] => diphthong_ipa(*a, *b).unwrap_or((single_vowel_ipa(*a), None)),
                _ => ("ɑ̠", None),
            };
            let onset = consonants_to_segments(&syl.onset).into_iter().map(|s| Segment { length: None, ..s }).collect();
            let coda = consonants_to_segments(&syl.coda).into_iter().map(|s| Segment { length: None, ..s }).collect();
            Syllable {
                text: syl.text.clone(),
                onset,
                nucleus: Segment { ipa: ipa.into(), stress: Some(stress), ..Default::default() },
                coda,
                offglide_share,
                ..Default::default()
            }
        })
        .collect();

    for i in 0..out.len().saturating_sub(1) {
        let (left, right) = out.split_at_mut(i + 1);
        let prev = &mut left[i];
        let next = &mut right[0];
        let (Some(last_coda), Some(first_onset)) = (prev.coda.last(), next.onset.first()) else {
            continue;
        };
        if last_coda.ipa == "n" && first_onset.ipa == "g" {
            prev.coda.pop();
            next.onset[0] = Segment { ipa: "ŋ".into(), dur: Some(0.155), ..Default::default() };
        } else if last_coda.ipa == "n" && first_onset.ipa == "k" {
            if let Some(c) = prev.coda.last_mut() {
                c.ipa = "ŋ".into();
            }
        } else if last_coda.ipa == first_onset.ipa
            && let Some(dur) = sung_geminate_dur(&first_onset.ipa)
        {
            prev.coda.pop();
            next.onset[0].dur = Some(dur);
        } else if matches!(last_coda.ipa.as_str(), "p" | "t" | "t̪" | "k") && matches!(first_onset.ipa.as_str(), "l" | "l̪") && first_onset.dur.is_none() {
            next.onset[0].dur = Some(0.088);
        } else if last_coda.ipa == "r" && last_coda.dur.is_none() {
            // Coda `r` before a following onset consonant (`tur-ma`, `ker-ta`, `per-he`) needs
            // ~135 ms so the trill strikes clearly before the next consonant's closure.
            if let Some(c) = prev.coda.last_mut() {
                c.dur = Some(0.135);
            }
        }
    }
    out
}

/// The Finnish front end for the lyrics router ([`super::syllables::read_lyrics`]).
/// Finnish shares the Latin alphabet with English and Turkish, so the player chooses it (`Lang::Fi`).
pub struct Finnish;

impl FrontEnd for Finnish {
    fn starts(&self, c: char) -> bool {
        is_letter(c)
    }
    fn continues(&self, c: char) -> bool {
        is_letter(c) || c == '-' || c == '\''
    }
    fn word(&self, tok: &str) -> Vec<Syllable> {
        let mut out = Vec::new();
        for part in tok.split('-').filter(|p| p.chars().any(is_letter)) {
            out.append(&mut sung_syllables(part));
        }
        out
    }
}

/// Across word boundaries (`Revontulet leiskuvat`): give an `l` onset enough room after a
/// voiceless stop coda (`p / t / k`) so the stop's release tail doesn't mask it.
pub(crate) fn adjust_boundaries(out: &mut [Syllable]) {
    for i in 0..out.len().saturating_sub(1) {
        if out[i].phrase_end {
            continue;
        }
        let stop_coda = out[i].coda.last().is_some_and(|c| matches!(c.ipa.as_str(), "p" | "t" | "t̪" | "k"));
        if stop_coda
            && let Some(on) = out[i + 1].onset.first_mut()
            && matches!(on.ipa.as_str(), "l" | "l̪")
            && on.dur.is_none()
        {
            on.dur = Some(0.088);
        }
    }
}

/// Lyrics → sung syllables with Latin script read as Finnish ([`Lang::Fi`]).
pub fn lyrics_to_syllables(text: &str) -> Vec<Syllable> {
    lyrics_in(text, &|_: &str, _: bool| Vec::<Segment>::new(), &MandarinLexicon::new(), Lang::Fi)
}

/// Finnish text → phrases of segments for Speak mode. Numbers are read out in Finnish.
pub fn to_phrases(text: &str) -> Vec<Phrase> {
    let normalized = expand_numbers(&text.replace('’', "'"));
    let mut phrases = Vec::new();
    let mut words: Vec<String> = Vec::new();
    let mut flush = |delim: &str, words: &mut Vec<String>| {
        let mut segs: Vec<Segment> = Vec::new();
        let mut said: Vec<String> = Vec::new();
        for w in words.drain(..) {
            let mut ws = Vec::new();
            for part in w.split('-').filter(|p| p.chars().any(is_letter)) {
                ws.append(&mut word(part));
            }
            let Some(last) = ws.last_mut() else { continue };
            last.word_end = Some(true);
            segs.append(&mut ws);
            said.push(w.to_lowercase());
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
            words.extend(
                part.split(|c: char| !(is_letter(c) || c == '-' || c == '\''))
                    .map(|w| w.trim_matches(|c| c == '-' || c == '\''))
                    .filter(|w| w.chars().any(is_letter))
                    .map(str::to_string),
            );
        }
    }
    flush("", &mut words);
    phrases
}

// ---------------------------------------------------------------- numbers
const ONES: [&str; 10] = ["nolla", "yksi", "kaksi", "kolme", "neljä", "viisi", "kuusi", "seitsemän", "kahdeksan", "yhdeksän"];

fn below_thousand(n: u64) -> String {
    if n == 0 {
        return String::new();
    }
    if n < 10 {
        return ONES[n as usize].into();
    }
    if n == 10 {
        return "kymmenen".into();
    }
    if n < 20 {
        return format!("{}toista", ONES[(n - 10) as usize]);
    }
    if n < 100 {
        let (t, o) = ((n / 10) as usize, (n % 10) as usize);
        return if o == 0 { format!("{}kymmentä", ONES[t]) } else { format!("{}kymmentä{}", ONES[t], ONES[o]) };
    }
    let (h, rem) = ((n / 100) as usize, n % 100);
    let prefix = if h == 1 { "sata".to_string() } else { format!("{}sataa", ONES[h]) };
    if rem == 0 { prefix } else { format!("{prefix}{}", below_thousand(rem)) }
}

fn number_words(digits: &str) -> String {
    let d = digits.trim_start_matches('0');
    if d.is_empty() {
        return ONES[0].into();
    }
    let Some(mut n) = d.parse::<u64>().ok().filter(|n| *n < 1_000_000_000_000_000) else {
        return digits.bytes().map(|b| ONES[(b - b'0') as usize]).collect::<Vec<_>>().join(" ");
    };
    let mut out = Vec::new();
    for (scale, sing, part) in [
        (1_000_000_000_000u64, "biljoona", "biljoonaa"),
        (1_000_000_000, "miljardi", "miljardia"),
        (1_000_000, "miljoona", "miljoonaa"),
        (1_000, "tuhat", "tuhatta"),
    ] {
        let q = n / scale;
        if q > 0 {
            if q == 1 {
                out.push(sing.to_string());
            } else {
                out.push(format!("{} {part}", below_thousand(q)));
            }
            n %= scale;
        }
    }
    if n > 0 {
        out.push(below_thousand(n));
    }
    out.join(" ")
}

fn expand_numbers(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let mut int_digits = String::new();
            while i < chars.len() && chars[i].is_ascii_digit() {
                int_digits.push(chars[i]);
                i += 1;
            }
            out.push(' ');
            out.push_str(&number_words(&int_digits));
            if i + 1 < chars.len() && chars[i] == ',' && chars[i + 1].is_ascii_digit() {
                i += 1;
                let mut frac = String::new();
                while i < chars.len() && chars[i].is_ascii_digit() {
                    frac.push(chars[i]);
                    i += 1;
                }
                out.push_str(" pilkku ");
                let zeros = frac.chars().take_while(|&c| c == '0').count();
                let rest = &frac[zeros..];
                for k in 0..zeros {
                    if k > 0 {
                        out.push(' ');
                    }
                    out.push_str(ONES[0]);
                }
                if !rest.is_empty() {
                    if zeros > 0 {
                        out.push(' ');
                    }
                    out.push_str(&number_words(rest));
                }
            }
            out.push(' ');
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}
