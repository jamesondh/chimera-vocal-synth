//! Turkish lyrics: letters → phonemes, syllable breaks and the spelling shown on each note.

use chimera::phonetics::Syllable;
use chimera::phonetics::turkish::lyrics_to_syllables;

fn texts(s: &[Syllable]) -> Vec<&str> {
    s.iter().map(|x| x.text.as_str()).collect()
}

/// each syllable as "onset.nucleus.coda" in IPA
fn ipa(s: &[Syllable]) -> Vec<String> {
    let j = |v: &[chimera::phonetics::Segment]| v.iter().map(|x| x.ipa.as_str()).collect::<String>();
    s.iter().map(|x| format!("{}.{}.{}", j(&x.onset), x.nucleus.ipa, j(&x.coda))).collect()
}

#[test]
fn one_consonant_opens_the_next_syllable() {
    let s = lyrics_to_syllables("kardeş Üsküdar'a");
    assert_eq!(texts(&s), ["kar", "deş", "Üs", "kü", "da", "r'a"]);
    assert_eq!(ipa(&s), ["k.a.ɾ", "d.e.ʃ", ".y.s", "c.y.", "d.a.", "ɾ.a."]);
    assert_eq!(s.iter().map(|x| x.word_end).collect::<Vec<_>>(), [false, true, false, false, false, true]);
}

#[test]
fn all_eight_vowels() {
    let s = lyrics_to_syllables("a e ı i o ö u ü");
    assert_eq!(s.iter().map(|x| x.nucleus.ipa.as_str()).collect::<Vec<_>>(), ["a", "e", "ɯ", "i", "o", "ø", "u", "y"]);
}

#[test]
fn k_g_l_follow_vowel_frontness() {
    // front: kedi, gel, renk; back: kol, gol, kalk
    assert_eq!(ipa(&lyrics_to_syllables("kedi gel renk")), ["c.e.", "d.i.", "ɟ.e.l", "ɾ.e.nc"]);
    assert_eq!(ipa(&lyrics_to_syllables("kol gol kalk")), ["k.o.ɫ", "g.o.ɫ", "k.a.ɫk"]);
    // a circumflex fronts the consonant before a back vowel
    assert_eq!(ipa(&lyrics_to_syllables("kâr lâle")), ["c.a.ɾ", "l.a.", "l.e."]);
}

#[test]
fn turkish_capitals_and_letters() {
    // I is dotless ı, İ is dotted i
    assert_eq!(ipa(&lyrics_to_syllables("Işık İstanbul")), [".ɯ.", "ʃ.ɯ.k", ".i.s", "t.a.n", "b.u.ɫ"]);
    assert_eq!(texts(&lyrics_to_syllables("İstanbul")), ["İs", "tan", "bul"]);
    assert_eq!(ipa(&lyrics_to_syllables("çocuk jale")), ["tʃ.o.", "dʒ.u.k", "ʒ.a.", "l.e."]);
}

#[test]
fn soft_g_is_silent_or_a_glide() {
    // silent after a back vowel, written with the syllable before it
    let s = lyrics_to_syllables("dağ ağaç");
    assert_eq!(texts(&s), ["dağ", "ağ", "aç"]);
    assert_eq!(ipa(&s), ["d.a.", ".a.", ".a.tʃ"]);
    // between front vowels it glides
    assert_eq!(ipa(&lyrics_to_syllables("değil")), ["d.e.", "j.i.l"]);
}

#[test]
fn hiatus_and_initial_clusters() {
    assert_eq!(texts(&lyrics_to_syllables("saat")), ["sa", "at"]);
    assert_eq!(ipa(&lyrics_to_syllables("tren")), ["tɾ.e.n"]);
}

#[test]
fn melisma_hyphens_and_breaths() {
    let s = lyrics_to_syllables("yâr - gel, gi-der");
    assert_eq!(texts(&s), ["yâr", "~", "gel", "gi", "der"]);
    assert_eq!(s[1].melisma, Some(true));
    assert_eq!(s[1].nucleus.ipa, "a");
    assert!(s[2].phrase_end);
    assert!(!s[3].word_end && s[4].word_end);
}

#[test]
fn every_sound_is_in_the_inventory() {
    // the planner must know every phoneme the rules produce, for every body
    use chimera::anatomy::human::BodyMix;
    use chimera::phonetics::inventory::phoneme;
    use chimera::phonetics::planner::is_vowel;
    chimera::phonetics::inventory::init();
    let s = lyrics_to_syllables("abcçdefgğhıijklmnoöprsştuüvyz kâr değil gel kol");
    for x in &s {
        assert!(is_vowel(&x.nucleus.ipa, BodyMix::MALE), "{} is not a vowel", x.nucleus.ipa);
        for c in x.onset.iter().chain(&x.coda) {
            assert!(phoneme(&c.ipa, BodyMix::MALE).is_some(), "no phoneme {}", c.ipa);
        }
    }
}
