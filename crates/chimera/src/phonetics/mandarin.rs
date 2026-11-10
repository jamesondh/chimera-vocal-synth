//! Mandarin pinyin → phoneme segments with lexical tones. Syllables take tone numbers ("ni3")
//! or tone marks ("nǐ"); unmarked ones are neutral. Words are space-separated and split into
//! syllables by apostrophes, tone numbers and longest match ("zhongguo" → zhong guo).
//! Tone 3 sandhi (3 3 → 2 3) is applied; the planner turns tones into pitch contours.

use super::english::Phrase;
use std::collections::HashMap;
use super::{Intonation, Segment, Syllable};

/// initial → (IPA, aspirated). Mandarin contrasts aspiration, not voicing: unaspirated b d g
/// are voiced [b d g] as between vowels in running speech and song (voiceless [p t k] silenced
/// every closure and made sung lines choppy); the affricates keep [ts ʈʂ tɕ] unaspirated.
fn initial(i: &str) -> Option<(&'static str, Option<bool>)> {
    Some(match i {
        "b" => ("b", None),
        "p" => ("p", Some(true)),
        "m" => ("m", None),
        "f" => ("f", None),
        "d" => ("d", None),
        "t" => ("t", Some(true)),
        "n" => ("n", None),
        "l" => ("l", None),
        "g" => ("g", None),
        "k" => ("k", Some(true)),
        // [x] is the textbook h, but the engine's velar frication was heard as [f]; with [h] the
        // Whisper judge's syllable errors fell ~2 points in both voices
        "h" => ("h", None),
        "j" => ("tɕ", Some(false)),
        "q" => ("tɕ", Some(true)),
        "x" => ("ɕ", None),
        "zh" => ("ʈʂ", Some(false)),
        "ch" => ("ʈʂ", Some(true)),
        "sh" => ("ʂ", None),
        "r" => ("ɻ", None),
        "z" => ("ts̪", Some(false)),
        "c" => ("ts̪", Some(true)),
        "s" => ("s̪", None),
        _ => return None,
    })
}

/// Canonical final (y/w spellings and abbreviations undone) → IPA. Each has exactly one vowel
/// segment, the nucleus that carries the tone (a diphthong is one segment).
fn final_ipa(f: &str) -> Option<&'static [&'static str]> {
    Some(match f {
        "a" => &["a"],
        "o" => &["w", "o"],
        "e" => &["ɤ"],
        "ai" => &["aɪ"],
        "ei" => &["eɪ"],
        "ao" => &["aʊ"],
        "ou" => &["oʊ"],
        "an" => &["a", "n"],
        "en" => &["ə", "n"],
        "ang" => &["ɑ", "ŋ"],
        "eng" => &["ə", "ŋ"],
        "ong" => &["ʊ", "ŋ"],
        "er" => &["ɚ"],
        "i" => &["i"],
        "ia" => &["j", "a"],
        "ie" => &["j", "ɛ"],
        "iao" => &["j", "aʊ"],
        "iou" => &["j", "oʊ"],
        "ian" => &["j", "ɛ", "n"],
        "in" => &["i", "n"],
        "iang" => &["j", "ɑ", "ŋ"],
        "ing" => &["i", "ŋ"],
        "iong" => &["j", "ʊ", "ŋ"],
        "u" => &["u"],
        "ua" => &["w", "a"],
        "uo" => &["w", "o"],
        "uai" => &["w", "aɪ"],
        "uei" => &["w", "eɪ"],
        "uan" => &["w", "a", "n"],
        "uen" => &["w", "ə", "n"],
        "uang" => &["w", "ɑ", "ŋ"],
        "ueng" => &["w", "ə", "ŋ"],
        "ü" => &["y"],
        "üe" => &["ɥ", "ɛ"],
        "üan" => &["ɥ", "ɛ", "n"],
        "ün" => &["y", "n"],
        // ponytail: the apical vowels of zi/zhi as [ɨ]; a calibrated [ɹ̩ ɻ̩] if they sound off
        "-i" => &["ɨ"],
        _ => return None,
    })
}

/// Tone-marked vowel → (vowel, tone).
fn unmark(c: char) -> Option<(char, u8)> {
    const MARKS: [(&str, char); 6] = [("āáǎà", 'a'), ("ēéěè", 'e'), ("īíǐì", 'i'), ("ōóǒò", 'o'), ("ūúǔù", 'u'), ("ǖǘǚǜ", 'ü')];
    MARKS.iter().find_map(|(m, v)| m.chars().position(|x| x == c).map(|k| (*v, k as u8 + 1)))
}

/// One pinyin syllable (lowercase, tone mark or trailing digit) → its segments and tone.
pub fn syllable(s: &str) -> Option<(Vec<Segment>, u8)> {
    let mut tone = 5u8;
    let mut base = String::new();
    for c in s.chars() {
        if let Some((v, t)) = unmark(c) {
            base.push(v);
            tone = t;
        } else if let Some(d) = c.to_digit(10) {
            tone = if (1..=4).contains(&d) { d as u8 } else { 5 };
        } else {
            base.push(if c == 'v' { 'ü' } else { c });
        }
    }
    let ini = ["zh", "ch", "sh"].into_iter().find(|i| base.starts_with(i)).or_else(|| base.get(..1).filter(|i| "bpmfdtnlgkhjqxrzcsyw".contains(*i)));
    let (ini, rest) = match ini {
        Some(i) => (i, &base[i.len()..]),
        None => ("", base.as_str()),
    };
    let fin: String = match ini {
        "y" if rest.starts_with('u') => format!("ü{}", &rest[1..]),
        "y" if rest.starts_with('i') => rest.to_string(),
        "y" if rest == "ong" => "iong".into(),
        "y" => format!("i{rest}"),
        "w" if rest.starts_with('u') => rest.to_string(),
        "w" => format!("u{rest}"),
        "j" | "q" | "x" if rest.starts_with('u') => format!("ü{}", &rest[1..]),
        "z" | "c" | "s" | "zh" | "ch" | "sh" | "r" if rest == "i" => "-i".into(),
        _ => match rest {
            "iu" => "iou".into(),
            "ui" => "uei".into(),
            "un" => "uen".into(),
            r => r.to_string(),
        },
    };
    let parts = final_ipa(&fin)?;
    let mut out = Vec::with_capacity(4);
    if !matches!(ini, "" | "y" | "w") {
        let (ipa, aspirated) = initial(ini)?;
        out.push(Segment { aspirated, ..Segment::new(ipa) });
    }
    for p in parts {
        // a neutral-tone e is [ə] (了 的 个): with the full [ɤ], short, 了 went unheard 27 times in 120
        // sentences. A full-tone [ə] (歌 特) the listener rejected against [ɤ] in A/B.
        let p = if tone == 5 && *p == "ɤ" { &"ə" } else { p };
        let nucleus = !matches!(*p, "j" | "w" | "ɥ" | "n" | "ŋ");
        out.push(if nucleus { Segment { tone: Some(tone), stress: Some(if tone < 5 { 1 } else { 0 }), ..Segment::new(p) } } else { Segment::new(p) });
    }
    Some((out, tone))
}

/// A word → syllables: split at apostrophes and after tone digits, then by longest match.
fn split_word(w: &str) -> Option<Vec<String>> {
    let mut out = Vec::new();
    let mut chunk = String::new();
    let mut chunks = Vec::new();
    for c in w.chars() {
        if c == '\'' || c == '’' || c == '-' {
            chunks.push(std::mem::take(&mut chunk));
            continue;
        }
        chunk.push(c);
        if c.is_ascii_digit() {
            chunks.push(std::mem::take(&mut chunk));
        }
    }
    chunks.push(chunk);
    // a chunk is letters and at most a tone digit at its end (the digit marks the last syllable)
    for c in chunks.iter().filter(|c| !c.is_empty()) {
        let letters = c.trim_end_matches(|x: char| x.is_ascii_digit());
        let cs: Vec<char> = letters.chars().collect();
        let mut i = 0;
        while i < cs.len() {
            // ponytail: greedy longest match ("xian" is one syllable; type "xi'an" for two)
            let n = (1..=(cs.len() - i).min(6)).rev().find(|&n| syllable(&cs[i..i + n].iter().collect::<String>()).is_some())?;
            out.push(cs[i..i + n].iter().collect::<String>());
            i += n;
        }
        if let Some(last) = out.last_mut() {
            last.push_str(&c[letters.len()..]);
        }
    }
    Some(out)
}

fn is_phrase_punct(c: char) -> bool {
    ".!?;:,，。！？；：、".contains(c)
}

/// One syllable of a phrase: its segments, tone and the character it was read from.
struct Syl {
    segs: Vec<Segment>,
    tone: u8,
    han: Option<char>,
    /// as written (a character or a pinyin syllable)
    text: String,
}

fn is_han(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c) || c == '〇'
}

fn is_pinyin_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '\'' || c == '’' || c == 'ü' || unmark(c).is_some()
}

/// Whether text is Mandarin: it has Chinese characters or tone-numbered pinyin ("ni3").
pub fn is_mandarin(text: &str) -> bool {
    text.chars().any(|c| is_han(c) || unmark(c).is_some())
        || text.to_lowercase().split(|c: char| !c.is_alphanumeric()).any(|w| w.ends_with(|c| ('1'..='5').contains(&c)) && w.starts_with(|c: char| c.is_alphabetic()) && split_word(w).is_some())
}

/// Mandarin text (characters, pinyin, or both) → phrases. Characters need the dictionary
/// (`public/data/mandarin.txt.gz`); pinyin works without it.
#[derive(Clone, Debug, Default)]
pub struct MandarinG2P {
    dict: Option<HashMap<String, String>>,
    /// longest entry (characters)
    max_len: usize,
}

impl MandarinG2P {
    pub fn new() -> MandarinG2P {
        MandarinG2P::default()
    }

    /// The dictionary as text ("汉字\tpīn yīn" lines).
    pub fn load_dict(&mut self, text: &str) {
        let mut m = HashMap::new();
        let mut max_len = 1;
        for line in text.split('\n') {
            if let Some((k, v)) = line.split_once('\t') {
                max_len = max_len.max(k.chars().count());
                m.insert(k.to_string(), v.to_string());
            }
        }
        self.dict = Some(m);
        self.max_len = max_len;
    }

    /// Text → phrases of segments (punctuation breaks phrases; "?" makes a question). A digit
    /// not after a pinyin letter is read as its character (2026 → 二零二六).
    pub fn to_phrases(&self, text: &str) -> Vec<Phrase> {
        let mut phrases = Vec::new();
        let mut cur = String::new();
        let mut prev: Option<char> = None;
        for c in text.chars() {
            if is_phrase_punct(c) {
                self.flush(&mut cur, Some(c), &mut phrases);
            } else if let Some(d) = c.to_digit(10).filter(|_| !prev.is_some_and(char::is_alphabetic)) {
                cur.push("零一二三四五六七八九".chars().nth(d as usize).unwrap());
            } else {
                cur.push(c);
            }
            prev = Some(c);
        }
        self.flush(&mut cur, None, &mut phrases);
        phrases
    }

    /// Characters → dictionary words by forward maximum matching (unknown characters are skipped).
    fn han_words(&self, run: &[char], words: &mut Vec<(String, Vec<Syl>)>) {
        let Some(dict) = &self.dict else { return };
        let mut i = 0;
        while i < run.len() {
            let found = (1..=self.max_len.min(run.len() - i)).rev().find_map(|n| {
                let w: String = run[i..i + n].iter().collect();
                dict.get(&w).map(|p| (w, n, p))
            });
            let Some((w, n, p)) = found else {
                i += 1;
                continue;
            };
            let readings: Vec<&str> = p.split(' ').collect();
            if readings.len() == n {
                let syls = readings.iter().zip(&run[i..i + n]).filter_map(|(r, &h)| syllable(r).map(|(segs, tone)| Syl { segs, tone, han: Some(h), text: h.to_string() })).collect();
                words.push((w, syls));
            }
            i += n;
        }
    }

    fn flush(&self, cur: &mut String, delim: Option<char>, phrases: &mut Vec<Phrase>) {
        let mut words = self.words(cur);
        cur.clear();
        let mut syls: Vec<&mut Syl> = words.iter_mut().flat_map(|w| w.1.iter_mut()).collect();
        yi_bu_sandhi(&mut syls);
        third_tone_sandhi(&mut syls);
        let n = syls.len();
        for (k, s) in syls.iter_mut().enumerate() {
            speech_durations(s, k == n - 1);
        }
        let mut segs = Vec::new();
        let mut said = Vec::new();
        for (w, ss) in &words {
            let start = segs.len();
            for s in ss {
                segs.extend(s.segs.iter().cloned());
            }
            if segs.len() > start {
                segs.last_mut().unwrap().word_end = Some(true);
                said.push(w.clone());
            }
        }
        if segs.is_empty() {
            return;
        }
        let q = matches!(delim, Some('?' | '？'));
        let pause = match delim {
            Some('.' | '!' | '?' | '。' | '！' | '？') | None => 0.35,
            _ => 0.18,
        };
        phrases.push(Phrase { segments: segs, intonation: if q { Intonation::Question } else { Intonation::Statement }, pause, words: said });
    }

    /// Lyrics → sung syllables, one per character (or pinyin syllable). Tones are not sung (the
    /// melody sets the pitch). Punctuation and line breaks end a phrase (a breath follows); a
    /// lone "-" or "~" holds the previous vowel onto the next note.
    pub fn lyrics(&self, text: &str) -> Vec<Syllable> {
        let mut out: Vec<Syllable> = Vec::new();
        let mut word = String::new();
        let push_word = |word: &mut String, out: &mut Vec<Syllable>| {
            for (_, syls) in self.words(word) {
                let n = syls.len();
                for (k, s) in syls.into_iter().enumerate() {
                    let v = s.segs.iter().position(|x| x.tone.is_some()).unwrap_or(0);
                    let mut nucleus = s.segs[v].clone();
                    nucleus.tone = None;
                    nucleus.dur = None;
                    out.push(Syllable { text: s.text, onset: s.segs[..v].to_vec(), nucleus, coda: s.segs[v + 1..].to_vec(), melisma: None, word_end: k == n - 1, phrase_end: false });
                }
            }
            word.clear();
        };
        for c in text.chars() {
            if is_phrase_punct(c) || c == '\n' {
                push_word(&mut word, &mut out);
                if let Some(last) = out.last_mut() {
                    last.phrase_end = true;
                }
            } else if c.is_whitespace() {
                push_word(&mut word, &mut out);
            } else if (c == '-' || c == '~') && word.is_empty() {
                if let Some(prev) = out.last() {
                    let s = Syllable { text: "~".into(), onset: vec![], nucleus: prev.nucleus.clone(), coda: vec![], melisma: Some(true), word_end: prev.word_end, phrase_end: false };
                    out.push(s);
                }
            } else {
                word.push(c);
            }
        }
        push_word(&mut word, &mut out);
        out
    }

    /// Text without phrase punctuation → words of syllables (dictionary words, pinyin words).
    fn words(&self, text: &str) -> Vec<(String, Vec<Syl>)> {
        let mut words: Vec<(String, Vec<Syl>)> = Vec::new();
        let chars: Vec<char> = text.to_lowercase().chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let start = i;
            if is_han(chars[i]) {
                while i < chars.len() && is_han(chars[i]) {
                    i += 1;
                }
                self.han_words(&chars[start..i], &mut words);
            } else if is_pinyin_char(chars[i]) {
                while i < chars.len() && is_pinyin_char(chars[i]) {
                    i += 1;
                }
                let w: String = chars[start..i].iter().collect();
                if let Some(syls) = split_word(&w) {
                    words.push((w, syls.iter().filter_map(|s| syllable(s).map(|(segs, tone)| Syl { segs, tone, han: None, text: s.clone() })).collect()));
                }
            } else {
                i += 1;
            }
        }
        words
    }
}

/// 不 bù → bú before a fourth tone; 一 yī → yí before a fourth tone, yì before the others
/// (not after a numeral or 第: 十一, 第一 keep yī).
fn yi_bu_sandhi(syls: &mut [&mut Syl]) {
    for i in 0..syls.len().saturating_sub(1) {
        let next = syls[i + 1].tone;
        let after_number = i > 0 && syls[i - 1].han.is_some_and(|h| "第零一二三四五六七八九十百千万".contains(h));
        match (syls[i].han, syls[i].tone) {
            (Some('不'), 4) if next == 4 => set_tone(syls[i], 2),
            (Some('一'), 1) if !after_number && next == 4 => set_tone(syls[i], 2),
            (Some('一'), 1) if !after_number && (1..=3).contains(&next) => set_tone(syls[i], 4),
            _ => {}
        }
    }
}

/// Mandarin is syllable-timed, and the English duration rules (stress-timed, polysyllabic
/// shortening) left nuclei of 60–70 ms, devoiced after aspiration. The nucleus gets a fixed
/// length: longer for a diphthong, shorter before a nasal coda, phrase-finally longer, neutral
/// tone shorter (at 80 ms 了 went unheard). That is twice a real speaker's, on purpose: with real
/// lengths (50–90 ms) Paraformer's character errors on 120 sentences were 52 %, at 125 ms 23 %,
/// at 150 ms 15 %, at 240 ms 13 %; in A/B the listener heard no gain from 180 over 150.
///
/// Consonants and glides get the mean length four real voices gave them (90 sentences, cut by
/// forced alignment); a glide after an initial is short, as the aligner hears it inside the
/// vowel. At the English defaults the same error was 13 points higher, mostly from the glides.
fn speech_durations(s: &mut Syl, last: bool) {
    let coda = s.segs.last().is_some_and(|x| x.ipa == "n" || x.ipa == "ŋ");
    let nucleus = s.segs.iter().position(|x| x.tone.is_some()).unwrap_or(s.segs.len());
    for (i, x) in s.segs.iter_mut().enumerate() {
        let ms = match x.ipa.as_str() {
            _ if i == nucleus => {
                let mut d = if s.tone == 5 { 120.0 } else { 150.0 };
                if x.ipa.chars().count() > 1 {
                    d += 40.0;
                }
                if coda {
                    d -= 30.0;
                }
                if last { d * 1.5 } else { d }
            }
            "b" | "d" | "g" => 58.0,
            "p" | "t" | "k" => 90.0,
            "tɕ" | "ʈʂ" | "ts̪" if x.aspirated == Some(true) => 101.0,
            "tɕ" | "ʈʂ" | "ts̪" => 73.0,
            "f" | "s̪" | "ʂ" | "ɕ" | "h" => 100.0,
            "m" => 63.0,
            "l" => 46.0,
            "ɻ" => 67.0,
            "n" | "ŋ" if i > nucleus => 67.0 * if last { 1.56 } else { 1.0 } * if s.tone == 5 { 0.85 } else { 1.0 },
            "n" => 50.0,
            "j" | "w" | "ɥ" if i > 0 => 30.0,
            _ => 57.0, // a glide that starts the syllable
        };
        x.dur = Some(ms / 1000.0);
    }
}

/// 3 3 → 2 3: in a run of third tones every one but the last rises.
// ponytail: whole-run rule; real sandhi follows prosodic words ("wo3 xiang3 mai3" → 2 2 3 or 3 2 3)
fn third_tone_sandhi(syls: &mut [&mut Syl]) {
    for i in 0..syls.len().saturating_sub(1) {
        if syls[i].tone == 3 && syls[i + 1].tone == 3 {
            set_tone(syls[i], 2);
        }
    }
}

fn set_tone(s: &mut Syl, t: u8) {
    s.tone = t;
    for seg in s.segs.iter_mut().filter(|x| x.tone.is_some()) {
        seg.tone = Some(t);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ipa(s: &str) -> String {
        let (segs, t) = syllable(s).unwrap();
        let mut o: String = segs.iter().map(|x| format!("{}{}", x.ipa, if x.aspirated == Some(true) { "ʰ" } else { "" })).collect();
        o.push_str(&t.to_string());
        o
    }

    #[test]
    fn syllables() {
        assert_eq!(ipa("ni3"), "ni3");
        assert_eq!(ipa("hǎo"), "haʊ3");
        assert_eq!(ipa("zhong1"), "ʈʂʊŋ1");
        assert_eq!(ipa("qu4"), "tɕʰy4");
        assert_eq!(ipa("xue2"), "ɕɥɛ2");
        assert_eq!(ipa("yuan2"), "ɥɛn2");
        assert_eq!(ipa("you3"), "joʊ3");
        assert_eq!(ipa("wei4"), "weɪ4");
        assert_eq!(ipa("gui4"), "gweɪ4");
        assert_eq!(ipa("liu2"), "ljoʊ2");
        assert_eq!(ipa("lü4"), "ly4");
        assert_eq!(ipa("lv4"), "ly4");
        assert_eq!(ipa("shi4"), "ʂɨ4");
        assert_eq!(ipa("ci2"), "ts̪ʰɨ2");
        assert_eq!(ipa("de"), "də5");
        assert_eq!(ipa("bo1"), "bwo1");
        assert_eq!(ipa("yi1"), "i1");
        assert_eq!(ipa("er4"), "ɚ4");
        assert!(syllable("xyz").is_none());
    }

    #[test]
    fn every_syllable_has_one_toned_nucleus() {
        for s in ["a", "zhuang", "xiong", "lüe", "nüe", "yong", "weng", "chuai", "ri", "si"] {
            let (segs, _) = syllable(s).unwrap();
            assert_eq!(segs.iter().filter(|x| x.tone.is_some()).count(), 1, "{s}");
        }
    }

    #[test]
    fn words_and_sandhi() {
        assert_eq!(split_word("zhongguo").unwrap(), ["zhong", "guo"]);
        assert_eq!(split_word("zhong1guo2").unwrap(), ["zhong1", "guo2"]);
        assert_eq!(split_word("xi'an").unwrap(), ["xi", "an"]);
        let p = MandarinG2P::new().to_phrases("ni3 hao3, ni3 hao3 ma?");
        assert_eq!(p.len(), 2);
        let tones = |p: &Phrase| p.segments.iter().filter_map(|s| s.tone).collect::<Vec<_>>();
        assert_eq!(tones(&p[0]), [2, 3]);
        assert_eq!(tones(&p[1]), [2, 3, 5]);
        assert_eq!(p[1].intonation, Intonation::Question);
        assert_eq!(p[0].words, ["ni3", "hao3"]);
    }

    #[test]
    fn lyrics_and_detection() {
        let mut g = MandarinG2P::new();
        g.load_dict("小\txiǎo\n星\txīng\n星星\txīng xing\n");
        let l = g.lyrics("小星星 -\n星");
        assert_eq!(l.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(), ["小", "星", "星", "~", "星"]);
        assert_eq!(l[1].onset.iter().map(|s| s.ipa.as_str()).collect::<Vec<_>>(), ["ɕ"]);
        assert_eq!(l[1].nucleus.ipa, "i");
        assert_eq!(l[1].coda[0].ipa, "ŋ");
        assert!(l[2].word_end && l[3].melisma == Some(true) && l[3].phrase_end && !l[4].phrase_end);
        assert!(is_mandarin("你好") && is_mandarin("ni3 hao3") && is_mandarin("nǐ hǎo"));
        assert!(!is_mandarin("Hello. Can you hear me?") && !is_mandarin("mp3 player 2"));
    }

    #[test]
    fn characters() {
        let mut g = MandarinG2P::new();
        g.load_dict("你\tnǐ\n好\thǎo\n行\txíng\n银行\tyín háng\n不\tbù\n是\tshì\n一\tyī\n个\tgè\n天\ttiān\n十\tshí\n三\tsān\n");
        let p = g.to_phrases("你好，银行不是一个，十一天。");
        let tones = |p: &Phrase| p.segments.iter().filter_map(|s| s.tone).collect::<Vec<_>>();
        assert_eq!(p[0].words, ["你", "好"]);
        assert_eq!(tones(&p[0]), [2, 3]);
        assert_eq!(p[1].words, ["银行", "不", "是", "一", "个"]);
        assert_eq!(tones(&p[1]), [2, 2, 2, 4, 2, 4]);
        assert_eq!(tones(&p[2]), [2, 1, 1]); // 十一天: 一 after a numeral keeps yī
        assert!(p[1].segments.iter().any(|s| s.ipa == "h")); // háng, not xíng
        assert_eq!(tones(&g.to_phrases("3天")[0]), [1, 1]); // 三天
    }
}

