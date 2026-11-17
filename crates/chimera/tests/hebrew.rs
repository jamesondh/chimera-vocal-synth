//! Hebrew lyrics: per-letter readings (ReNikud's, made in the web app) → sung syllables.
use chimera::phonetics::english::EnglishG2P;
use chimera::phonetics::hebrew::word_syllables;
use chimera::phonetics::syllables::{lyrics_to_syllables, lyrics_to_syllables_with};
use chimera::phonetics::Syllable;

fn chunks(c: &[&str]) -> Vec<String> {
    c.iter().map(|s| s.to_string()).collect()
}
/// "onset.nucleus.coda" per syllable, with the text: "ש=ʃ.a."
fn shown(s: &[Syllable]) -> Vec<String> {
    let ipa = |v: &[chimera::phonetics::Segment]| v.iter().map(|s| s.ipa.as_str()).collect::<String>();
    s.iter().map(|s| format!("{}={}.{}.{}", s.text, ipa(&s.onset), s.nucleus.ipa, ipa(&s.coda))).collect()
}

#[test]
fn a_silent_letter_stays_with_the_letter_before_it() {
    // שלום: ו is the o's letter and has no sound of its own
    let s = word_syllables("שלום", &chunks(&["ʃa", "lˈo", "", "m"]));
    assert_eq!(shown(&s), ["ש=ʃ.a.", "לום=l.o.m"]);
    assert_eq!(s[1].nucleus.stress, Some(1));
    assert_eq!(s[0].nucleus.stress, Some(0));
}

#[test]
fn between_vowels_one_consonant_goes_right_and_a_first_cluster_stays_whole() {
    // מדבר midbˈaʁ: d closes the first syllable
    assert_eq!(shown(&word_syllables("מדבר", &chunks(&["mi", "d", "bˈa", "ʁ"]))), ["מד=m.i.d", "בר=b.a.ʁ"]);
    // שתיים ʃtˈajim
    assert_eq!(shown(&word_syllables("שתיים", &chunks(&["ʃ", "tˈa", "ji", "", "m"]))), ["שת=ʃt.a.", "יים=j.i.m"]);
}

#[test]
fn a_final_het_after_its_vowel_is_its_own_syllable() {
    // רוח ʁˈuaχ: the model writes the last ח as "aχ"
    assert_eq!(shown(&word_syllables("רוח", &chunks(&["ʁˈu", "", "aχ"]))), ["רו=ʁ.u.", "ח=.a.χ"]);
}

#[test]
fn the_models_g_and_affricates_are_sounds_the_engine_has() {
    // ɡ (U+0261) is the model's g; ג׳ and צ׳ are dʒ and tʃ, one sound each
    let s = word_syllables("ג׳ירפה", &chunks(&["dʒˈi", "", "ʁa", "fa", ""]));
    assert_eq!(shown(&s), ["ג׳י=dʒ.i.", "ר=ʁ.a.", "פה=f.a."]);
    assert_eq!(word_syllables("גג", &chunks(&["ɡˈa", "ɡ"]))[0].onset[0].ipa, "g");
}

#[test]
fn lyrics_take_the_readings_letter_by_letter_and_mix_with_other_scripts() {
    let g = EnglishG2P::new();
    let r = chunks(&["ʃa", "lˈo", "", "m", "ʔo", "", "lˈa", "m"]);
    let s = lyrics_to_syllables_with("שלום, star עולם", &g, &r);
    assert_eq!(s.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(), ["ש", "לום", "star", "עו", "לם"]);
    assert!(s[1].word_end && s[1].phrase_end);
    assert!(s[4].word_end && !s[3].word_end);
}

#[test]
fn hebrew_without_readings_is_skipped_and_the_rest_is_sung() {
    let g = EnglishG2P::new();
    assert_eq!(lyrics_to_syllables("שלום star", &g).len(), 1);
    // readings for other lyrics (too few): the words they cover are read, the rest skipped
    let s = lyrics_to_syllables_with("שלום עולם star", &g, &chunks(&["ʃa", "lˈo", "", "m"]));
    assert_eq!(s.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(), ["ש", "לום", "star"]);
    // a word with no vowel in its reading has no note
    assert!(word_syllables("ו", &chunks(&["v"])).is_empty());
}
