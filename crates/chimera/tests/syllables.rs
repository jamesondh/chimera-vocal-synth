//! Port of tests/syllables.test.ts.
use chimera::phonetics::Segment;
use chimera::phonetics::english::arpabet_to_segments;
use chimera::phonetics::syllables::{lyrics_to_syllables, split_spelling, syllabify_segments, unknown_lyric_ipa};

// a few CMUdict entries, enough to test without loading the dictionary
fn dict(w: &str) -> Option<&'static str> {
    Some(match w {
        "twinkle" => "T W IH1 NG K AH0 L",
        "little" => "L IH1 T AH0 L",
        "star" => "S T AA1 R",
        "how" => "HH AW1",
        "i" => "AY1",
        "wonder" => "W AH1 N D ER0",
        "what" => "W AH1 T",
        "you" => "Y UW1",
        "are" => "AA1 R",
        "extra" => "EH1 K S T R AH0",
        "of" => "AH1 V",
        _ => return None,
    })
}
fn word(w: &str, _reduce: bool) -> Vec<Segment> {
    dict(w).map(arpabet_to_segments).unwrap_or_default()
}
fn ipa(xs: &[Segment]) -> String {
    xs.iter().map(|x| x.ipa.as_str()).collect()
}

#[test]
fn maximal_onset_with_legal_english_clusters() {
    let tw = syllabify_segments(&word("twinkle", false));
    let got: Vec<[String; 3]> = tw.iter().map(|s| [ipa(&s.onset), s.nucleus.ipa.clone(), ipa(&s.coda)]).collect();
    assert_eq!(got, [["tw", "ɪ", "ŋ"].map(String::from), ["k", "ə", "l"].map(String::from)]);
    // "extra": k + str → ek-stra (str is a legal onset)
    let ex = syllabify_segments(&word("extra", false));
    assert_eq!(ex.iter().map(|s| ipa(&s.onset)).collect::<Vec<_>>(), ["", "stɹ"]);
    assert_eq!(ipa(&ex[0].coda), "k");
}

#[test]
fn spelling_splits_to_match() {
    assert_eq!(split_spelling("twinkle", 2), ["twin", "kle"]);
    assert_eq!(split_spelling("little", 2), ["lit", "tle"]);
    assert_eq!(split_spelling("wonder", 2), ["won", "der"]);
    assert_eq!(split_spelling("are", 1), ["are"]);
}

#[test]
fn lyrics_words_melisma_phrase_ends_unreduced_function_words() {
    let syl = lyrics_to_syllables("Twinkle little star, how I - wonder", &word);
    assert_eq!(syl.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(), ["Twin", "kle", "lit", "tle", "star", "how", "I", "~", "won", "der"]);
    assert!(syl[4].phrase_end);
    assert_eq!(syl[7].melisma, Some(true));
    assert_eq!(syl[7].nucleus.ipa, "aɪ");
    assert!(syl[1].word_end && !syl[0].word_end);
}

#[test]
fn slashed_words_are_ipa_and_skip_the_dictionary() {
    // 你問我愛你有多深: one syllable per slashed word, mixed with dictionary words
    let syl = lyrics_to_syllables("/ni/ /wən/ /wɔ/ /aɪ/ star /joʊ/ /dwɔ/ /ʂən/", &word);
    let got: Vec<[String; 3]> = syl.iter().map(|s| [ipa(&s.onset), s.nucleus.ipa.clone(), ipa(&s.coda)]).collect();
    let want = [["n", "i", ""], ["w", "ə", "n"], ["w", "ɔ", ""], ["", "aɪ", ""], ["st", "ɑ", "ɹ"], ["j", "oʊ", ""], ["dw", "ɔ", ""], ["ʂ", "ə", "n"]];
    assert_eq!(got, want.map(|x| x.map(String::from)));
    assert_eq!(syl[1].text, "wən");
    assert!(syl.iter().all(|s| s.word_end && s.melisma.is_none()));
}

#[test]
fn a_slashed_word_with_two_vowels_is_two_syllables() {
    // the dictionary is never asked: `word` knows no IPA, yet this still yields syllables
    let syl = lyrics_to_syllables("/ni.xaʊ/ - /ma/,", &word);
    let text: Vec<&str> = syl.iter().map(|s| s.text.as_str()).collect();
    // "." puts x in the second syllable's onset, though x is no English onset
    assert_eq!(text, ["ni", "xaʊ", "~", "ma"]);
    assert_eq!(syl.iter().map(|s| s.word_end).collect::<Vec<_>>(), [false, true, true, true]);
    assert_eq!(syl[2].melisma, Some(true));
    assert!(syl[3].phrase_end);
}

#[test]
fn an_unclosed_slash_is_ignored() {
    let syl = lyrics_to_syllables("star /", &word);
    assert_eq!(syl.len(), 1);
}

#[test]
fn mandarin_initials_and_aspiration() {
    // 去 學 吃 知 西 月 怕 爸
    let syl = lyrics_to_syllables("/tɕʰy/ /ɕɥɛ/ /ʈʂʰɨ/ /ʈʂɨ/ /ɕi/ /ɥɛ/ /pʰa/ /pa/", &word);
    let onsets: Vec<String> = syl.iter().map(|s| ipa(&s.onset)).collect();
    assert_eq!(onsets, ["tɕ", "ɕɥ", "ʈʂ", "ʈʂ", "ɕ", "ɥ", "p", "p"]);
    assert_eq!(syl[0].nucleus.ipa, "y");
    // unmarked voiceless stops and affricates are unaspirated in IPA lyrics, not English-aspirated
    let asp: Vec<Option<bool>> = syl.iter().map(|s| s.onset[0].aspirated).collect();
    assert_eq!(asp, [Some(true), None, Some(true), Some(false), None, None, Some(true), Some(false)]);
}

#[test]
fn apical_vowels_and_erhua() {
    // 四 是 兒 花兒
    let syl = lyrics_to_syllables("/sɿ/ /ʂʅ/ /ɚ/ /xwaɚ/", &word);
    let got: Vec<[String; 3]> = syl.iter().map(|s| [ipa(&s.onset), s.nucleus.ipa.clone(), ipa(&s.coda)]).collect();
    let want = [["s", "ɨ", ""], ["ʂ", "ɨ", ""], ["", "ɚ", ""], ["xw", "a", "ɻ"]];
    assert_eq!(got, want.map(|x| x.map(String::from)));
    assert_eq!(lyrics_to_syllables("/sz̩/ /ʂɻ̩/", &word).iter().map(|s| s.nucleus.ipa.as_str()).collect::<Vec<_>>(), ["ɨ", "ɨ"]);
}

#[test]
fn unknown_ipa_in_lyrics_is_reported() {
    assert_eq!(unknown_lyric_ipa("star /tɕʰy/ /sɿ/"), Vec::<String>::new());
    // a stray capital and a nasal vowel (precomposed ã): sounds the voice lacks, each named once
    assert_eq!(unknown_lyric_ipa("/Ŋɔ/ /ã/ /Ŋa/"), ["Ŋ", "ã"]);
}
