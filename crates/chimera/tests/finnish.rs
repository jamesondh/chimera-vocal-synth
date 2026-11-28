//! Finnish G2P: letters, diphthongs, long vowels, geminates, stress, numbers, phrases, and song planning.
use chimera::anatomy::human::BodyMix;
use chimera::phonetics::finnish::{lyrics_to_syllables, to_phrases, word};
use chimera::phonetics::planner::{PlanOptions, is_vowel, plan_phrases};
use chimera::phonetics::sing::{SongOptions, SungNote, plan_song};
use chimera::phonetics::syllables::{Lang, lyrics_in};
use chimera::phonetics::{Segment, inventory::phoneme};

fn ipa(segs: &[Segment]) -> String {
    segs.iter().map(|s| s.ipa.as_str()).collect()
}

#[test]
fn letters_diphthongs_and_unaspirated_stops() {
    let w = word("kierros");
    assert_eq!(ipa(&w), "kieros");
    assert_eq!(w[0].aspirated, Some(true));
    assert_eq!(w[0].vot, Some(0.012));
    assert_eq!(w[1].ipa, "ie");
    assert_eq!(w[1].stress, Some(1));
    assert_eq!(w[2].length, Some(2.2)); // geminate rr

    assert_eq!(ipa(&word("Suomi")), "sʊɔmi");
    assert_eq!(ipa(&word("yöllä")), "yœl̪æ̞");
    assert_eq!(ipa(&word("tähtönen")), "t̪æ̞ht̪œnen");
    assert_eq!(ipa(&word("pöytä")), "pœyt̪æ̞");
    assert_eq!(ipa(&word("Revontulet")), "revont̪u̝l̪et̪");
    assert_eq!(ipa(&word("kukka")), "ku̝kɑ̠");
    assert_eq!(ipa(&word("langat")), "l̪ɑ̠ŋɑ̠t̪");
    assert_eq!(ipa(&word("lanka")), "l̪ɑ̠ŋkɑ̠");
    assert_eq!(ipa(&word("yöhön")), "yœhœn");
    // Non-intervocalic `ng` (`englanti`) stays short [ŋ], not geminate
    let eng = word("englanti");
    assert_eq!(eng[1].ipa, "ŋ");
    assert_eq!(eng[1].length, None);
    // Apostrophe forces a syllable boundary (`ha'un`, `vaa'an`) without splitting into two words
    assert_eq!(lyrics_to_syllables("tähtien ha'un vaa'an").iter().map(|s| s.text.as_str()).collect::<Vec<_>>(), ["täh", "ti", "en", "ha", "'un", "vaa", "'an"]);
    let haun = to_phrases("ha'un");
    assert_eq!(haun[0].words, ["ha'un"]);
    assert_eq!(ipa(&haun[0].segments), "hɑ̠u̝n");
}

#[test]
fn long_vowels_and_geminates_keep_length_in_speech() {
    let maa = word("maa");
    assert_eq!(ipa(&maa), "mɑ̠");
    assert_eq!(maa[1].length, Some(1.40));
    let ma = word("ma");
    assert_eq!(ma[1].length, Some(0.55));

    let p = to_phrases("kierros Suomi yöllä");
    let plan = plan_phrases(
        &p.iter().map(|ph| chimera::phonetics::planner::PhraseInput { segments: ph.segments.clone(), intonation: Some(ph.intonation), pause: Some(ph.pause) }).collect::<Vec<_>>(),
        &PlanOptions { body: BodyMix::from("female"), t0: 0.0, ..Default::default() },
    );
    // Opening diphthongs expand to i+ɛ, u̝+ɔ, y+œ in speech
    let planned: Vec<&str> = plan.segments.iter().map(|s| s.ipa.as_str()).collect();
    assert!(planned.windows(2).any(|w| w == ["i", "ɛ"]), "{planned:?}");
    assert!(planned.windows(2).any(|w| w == ["u̝", "ɔ"]), "{planned:?}");
    assert!(planned.windows(2).any(|w| w == ["y", "œ"]), "{planned:?}");
}

#[test]
fn numbers_are_read_out_in_finnish() {
    let p = to_phrases("Kierros 14, arvo 3,05.");
    assert_eq!(p[0].words, ["kierros", "neljätoista"]);
    assert_eq!(p[1].words, ["arvo", "kolme", "pilkku", "nolla", "viisi"]);
}

#[test]
fn every_sound_is_one_the_voice_can_make() {
    let body = BodyMix::from("female");
    for phrase in to_phrases("Revontulet leiskuvat yöllä. Kierros 14, Suomi soi hiljaa. Ystäväiseni, kuule ikuinen laulu!") {
        for s in &phrase.segments {
            assert!(is_vowel(&s.ipa, body) || phoneme(&s.ipa, body).is_some(), "unknown sound {:?}", s.ipa);
        }
    }
}

#[test]
fn latin_script_is_finnish_only_when_chosen() {
    let text = |lang| {
        lyrics_in("kierros yöllä", &|w: &str, _| vec![Segment::new(w)], &Default::default(), lang)
            .into_iter()
            .map(|s| s.text)
            .collect::<Vec<_>>()
    };
    assert_eq!(text(Lang::Fi), ["kier", "ros", "yöl", "lä"]);
}

#[test]
fn finnish_lyrics_plan_as_a_song_with_balanced_opening_diphthongs() {
    let syls = lyrics_to_syllables("Revontulet leiskuvat, kierros neljätoista");
    assert_eq!(syls.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(), ["Re", "von", "tu", "let", "leis", "ku", "vat", "kier", "ros", "nel", "jä", "tois", "ta"]);
    // `let` (coda t) -> `leis` (onset l boosted to 88ms)
    assert_eq!(syls[4].onset[0].dur, Some(0.088));
    // `kier` -> opening diphthong `ie` with 60% offglide share, geminate `r` consolidated to `ros` onset (165ms)
    assert_eq!(syls[7].nucleus.ipa, "ie");
    assert_eq!(syls[7].offglide_share, Some(0.60));
    assert!(syls[7].coda.is_empty());
    assert_eq!(syls[8].onset[0].dur, Some(0.165));

    let notes: Vec<SungNote> = syls.into_iter().map(|syl| SungNote { midi: 64.0, beats: 0.5, syl }).collect();
    let plan = plan_song(&notes, &SongOptions { body: BodyMix::from("female"), bpm: 80.0, t0: 0.25, vibrato: 30.0 });
    // `kier` planned as i (~104ms) + ɛ (~156ms), both well above MIN_VOWEL!
    let pair = plan.segments.windows(2).find(|w| w[0].ipa == "i" && w[1].ipa == "ɛ").expect("ie diphthong");
    assert!(pair[0].t1 - pair[0].t0 > 0.08);
    assert!(pair[1].t1 - pair[1].t0 > 0.12);
}
