//! Russian spelling → segments (`phonetics::russian`), through speech and through sung lyrics.
use chimera::anatomy::human::BodyMix;
use chimera::phonetics::english::EnglishG2P;
use chimera::phonetics::planner::{PlanOptions, plan_utterance};
use chimera::phonetics::russian;
use chimera::phonetics::songs::DEMO_SONGS;
use chimera::phonetics::syllables::lyrics_to_syllables;

fn ipa(segs: &[chimera::phonetics::Segment]) -> String {
    segs.iter().map(|s| s.ipa.as_str()).collect::<Vec<_>>().join(" ")
}
/// as sung: no vowel reduction
fn sung(w: &str) -> String {
    ipa(&russian::word(w, false))
}
/// as spoken: unstressed vowels reduce
fn said(w: &str) -> String {
    ipa(&russian::word(w, true))
}
/// a whole spoken phrase
fn phrase(text: &str) -> String {
    ipa(&EnglishG2P::new().to_phrases(text)[0].segments)
}

#[test]
fn vowels_and_consonants() {
    assert_eq!(sung("аоуыэи"), "a o u ɨ e i");
    assert_eq!(sung("шар"), "ʂ a ɾ");
    assert_eq!(sung("хор"), "x o ɾ");
    assert_eq!(sung("цирк"), "ts ɨ ɾ k");
    assert_eq!(sung("жир"), "ʐ ɨ ɾ");
    // р is a tap: the trill closed the voice off for ~100 ms and "рыба" was heard as "дыба"
    assert_eq!(sung("рыба"), "ɾ ɨ b a");
}

#[test]
fn iotation_and_softness() {
    assert_eq!(sung("ёж"), "j o ʂ");
    assert_eq!(sung("маяк"), "m a j a k");
    assert_eq!(sung("съезд"), "s j e s t");
    // a soft consonant is the plain one and a short glide; л also changes from dark to clear
    assert_eq!(sung("лук"), "ɫ u k");
    assert_eq!(sung("люк"), "l j u k");
    let soft = russian::word("люк", false);
    assert_eq!(soft[1].dur, Some(0.025));
    // ь before и and о: the vowel gets its own /j/ (it was dropped)
    assert_eq!(sung("чьи"), "tʃ j j i");
    assert_eq!(sung("бульо́н"), "b u l j j o n");
}

#[test]
fn stress_marks_and_the_fallbacks() {
    let s = russian::word("молоко́", true);
    assert_eq!(ipa(&s), "m ɐ ɫ ɐ k o");
    assert_eq!(s.iter().filter_map(|s| s.stress).collect::<Vec<_>>(), [0, 0, 1]);
    // ё is stressed
    assert_eq!(said("поёт"), "p ɐ j o t");
    // a frequent word from the table, then the first vowel
    assert_eq!(said("тебя"), "t j ɪ b j a");
    assert_eq!(said("мама"), "m a m ɐ");
    // singing keeps full vowels
    assert_eq!(sung("молоко́"), "m o ɫ o k o");
}

#[test]
fn unstressed_ya_and_a_after_ch() {
    // я was /ɐ/ everywhere; before the stress it is /ɪ/
    assert_eq!(said("язы́к"), "j ɪ z ɨ k");
    assert_eq!(said("пяти́"), "p j ɪ t j i");
    assert_eq!(said("часы́"), "tʃ j ɪ s ɨ");
    // but word-final я and а stay open
    assert_eq!(said("неде́ля"), "n j ɪ d j e l j ɐ");
    assert_eq!(said("ту́ча"), "t u tʃ j ɐ");
}

#[test]
fn devoicing_and_assimilation_inside_a_word() {
    assert_eq!(sung("дуб"), "d u p");
    assert_eq!(sung("лодка"), "ɫ o t k a");
    assert_eq!(sung("сдать"), "z d a t j");
    assert_eq!(sung("вторник"), "f t o ɾ n j i k");
    // в doesn't voice what's before it
    assert_eq!(sung("твой"), "t v o j");
    // through the softness glide
    assert_eq!(sung("ло́шадь"), "ɫ o ʂ a t j");
}

#[test]
fn a_preposition_is_part_of_the_next_word() {
    // each was devoiced as if it ended a phrase: "нат лесом", "ф лесу", "ис дома"
    assert_eq!(phrase("над ле́сом"), "n ɐ d l j e s ɐ m");
    assert_eq!(phrase("в лесу́"), "v l j ɪ s u");
    assert_eq!(phrase("из до́ма"), "i z d o m ɐ");
    // and it assimilates to an obstruent
    assert_eq!(phrase("в ти́хом"), "f t j i x ɐ m");
    assert_eq!(phrase("с горы́"), "z g ɐ ɾ ɨ");
    assert_eq!(phrase("из-под стола́"), "i s p ɐ t s t ɐ ɫ a");
    // sung words come one at a time: the preposition keeps its spelling's voicing
    assert_eq!(sung("над"), "n a d");
    // a word that isn't a preposition still devoices
    assert_eq!(phrase("са́д мо́й"), "s a t m o j");
}

#[test]
fn g_in_the_genitive_ending_is_v() {
    assert_eq!(said("его́"), "j ɪ v o");
    assert_eq!(said("сего́дня"), "s j ɪ v o d n j ɐ");
    assert_eq!(said("кра́сного"), "k ɾ a s n ɐ v ɐ");
    assert_eq!(said("ничего́"), "n j i tʃ j ɪ v o");
    assert_eq!(said("мно́го"), "m n o g ɐ");
}

#[test]
fn spelling_conventions() {
    assert_eq!(sung("что"), "ʂ t o");
    assert_eq!(sung("коне́чно"), "k o n j e ʂ n o");
    assert_eq!(sung("сча́стье"), "ʃ j a s t j j e");
    assert_eq!(sung("мужчи́на"), "m u ʃ j i n a");
    assert_eq!(sung("по́здно"), "p o z n o");
    assert_eq!(sung("со́лнце"), "s o n ts e");
    assert_eq!(sung("чу́вство"), "tʃ j u s t v o");
    assert_eq!(said("смеётся"), "s m j ɪ j o ts ɐ");
    assert_eq!(sung("купа́ться"), "k u p a ts a");
}

#[test]
fn a_doubled_letter_is_one_consonant() {
    // long between vowels, plain before a consonant (it was two separate closures)
    let vanna = russian::word("ва́нна", false);
    assert_eq!(ipa(&vanna), "v a n a");
    assert!(vanna[2].dur.is_some_and(|d| d > 0.1), "{:?}", vanna[2].dur);
    let russkiy = russian::word("ру́сский", false);
    assert_eq!(ipa(&russkiy), "ɾ u s k j i j");
    assert_eq!(russkiy[2].dur, None);
}

#[test]
fn speech_plans_and_english_is_untouched() {
    let g = EnglishG2P::new();
    let p = g.to_phrases("Приве́т! Ты меня́ слы́шишь?");
    assert_eq!(p.len(), 2);
    assert_eq!(p[0].words, ["приве\u{301}т"]);
    assert_eq!(p[1].words.len(), 3);
    for p in &p {
        let plan = plan_utterance(&p.segments, &PlanOptions { body: BodyMix::MALE, ..Default::default() });
        assert!(plan.end.is_finite() && plan.end > 0.0);
        assert!(!plan.gestures.is_empty());
    }
    // a line can mix both
    let mixed = g.to_phrases("hello ми́р");
    assert_eq!(mixed[0].words.len(), 2);
    assert!(ipa(&mixed[0].segments).ends_with("m j i ɾ"));
}

#[test]
fn lyrics_split_into_syllables() {
    let g = EnglishG2P::new();
    let s = lyrics_to_syllables("мо-ло-ко́ ~", &g);
    assert_eq!(s.len(), 4);
    assert_eq!(s[3].melisma, Some(true));
    let texts = |t: &str| lyrics_to_syllables(t, &g).iter().map(|s| s.text.clone()).collect::<Vec<_>>();
    assert_eq!(texts("мая́к"), ["ма", "я\u{301}к"]);
    assert_eq!(russian::split_spelling("поёт", 2), ["по", "ёт"]);
    // an obstruent and р start a syllable together (се-стра́); other clusters split (ло́д-ка)
    let onsets = |t: &str| lyrics_to_syllables(t, &g).iter().map(|s| ipa(&s.onset)).collect::<Vec<_>>();
    assert_eq!(onsets("сестра́"), ["s j", "s t ɾ"]);
    assert_eq!(onsets("ло́дка"), ["ɫ", "k"]);
}

#[test]
fn the_demo_song_has_a_note_per_syllable() {
    let g = EnglishG2P::new();
    let song = DEMO_SONGS.iter().find(|s| russian::contains_russian(s.lyrics)).expect("a Russian demo song");
    assert_eq!(lyrics_to_syllables(song.lyrics, &g).len(), song.notes.len());
}

#[test]
fn a_sung_preposition_without_a_vowel_joins_the_next_word() {
    // в, с, к have no vowel, so no note: they were silently dropped from the song
    let g = EnglishG2P::new();
    let s = lyrics_to_syllables("в лесу́", &g);
    assert_eq!(s.len(), 2);
    assert_eq!(s[0].text, "в ле");
    assert_eq!(ipa(&s[0].onset), "v l j");
    // and it takes the next word's voicing
    assert_eq!(ipa(&lyrics_to_syllables("к тебе́", &g)[0].onset), "k t j");
    assert_eq!(ipa(&lyrics_to_syllables("с горы́", &g)[0].onset), "z g");
    assert_eq!(ipa(&lyrics_to_syllables("в ти́хом", &g)[0].onset), "f t j");
}

#[test]
fn unstressed_vowels_keep_a_syllable_s_length() {
    // the duration rules are English's and have no /ɐ/: unstressed vowels came out 40-50 ms,
    // "Доброе утро" took 0.75 s for five syllables, and Whisper lost whole words
    let g = EnglishG2P::new();
    let p = &g.to_phrases("До́брое у́тро")[0];
    let weak: Vec<f64> = p.segments.iter().filter(|s| s.stress == Some(0)).filter_map(|s| s.dur).collect();
    assert_eq!(weak.len(), 3);
    assert!(weak.iter().all(|d| *d >= 0.07), "{weak:?}");
    // the last one, before a pause, is the longest
    assert!(weak[2] > weak[0]);
}

#[test]
fn a_stressed_y_is_as_long_as_the_other_stressed_vowels() {
    // the duration table's /ɨ/ is English's reduced vowel (110 ms inherent): "дым" came out with
    // half the vowel of "дум" and Whisper wrote "дум" for it
    let g = EnglishG2P::new();
    let y = |text: &str| g.to_phrases(text)[0].segments.iter().find(|s| s.ipa == "ɨ").unwrap().dur;
    assert_eq!(y("ды́м"), Some(0.19));
    assert_eq!(y("ды́м и до́м"), Some(0.12));
    // sung, the note sets the length
    assert!(russian::word("дым", false).iter().all(|s| s.ipa != "ɨ" || s.dur.is_none()));
}
