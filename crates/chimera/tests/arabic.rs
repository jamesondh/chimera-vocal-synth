use chimera::phonetics::arabic::{self, ArabicDialect};
use chimera::phonetics::english::EnglishG2P;
use chimera::phonetics::mandarin::MandarinLexicon;
use chimera::phonetics::planner::{PlanOptions, plan_utterance};
use chimera::phonetics::sing::{NoteOnOptions, SongOptions, SungNote, plan_note_on, plan_song};
use chimera::phonetics::syllables::{FrontEnd, Lang, lyrics_in_with_dialect};
use chimera::{anatomy::human::ArticKey, control::score::ControlKey};

fn ipa(segs: &[chimera::phonetics::Segment]) -> Vec<&str> {
    segs.iter().map(|s| s.ipa.as_str()).collect()
}

#[test]
fn tashkeel_sets_short_vowels_and_sukun_blocks_inference() {
    assert_eq!(
        ipa(&arabic::word_segments("كَتَبَ", ArabicDialect::Msa)),
        ["k", "a", "t", "a", "b", "a"]
    );
    assert_eq!(
        ipa(&arabic::word_segments("مُدْرِس", ArabicDialect::Msa)),
        ["m", "u", "d", "ɾ", "i", "s"]
    );
    assert_eq!(
        ipa(&arabic::word_segments("اِسْمِي", ArabicDialect::Msa)),
        ["i", "s", "m", "i"]
    );
}

#[test]
fn alif_lengthens_the_vowel_and_shadda_lengthens_its_consonant() {
    let salaam = arabic::word_segments("سَلَام", ArabicDialect::Msa);
    assert_eq!(ipa(&salaam), ["s", "a", "l", "a", "m"]);
    assert_eq!(salaam[3].dur, Some(0.18));

    let shadda = arabic::word_segments("مُدَرِّس", ArabicDialect::Msa);
    assert_eq!(
        shadda.iter().find(|s| s.ipa == "ɾ").unwrap().dur,
        Some(0.12)
    );

    let sun = arabic::word_segments("الشَّمْس", ArabicDialect::Msa);
    assert_eq!(sun.iter().find(|s| s.ipa == "ʃ").unwrap().dur, Some(0.12));
}

#[test]
fn partial_vocalization_keeps_the_unmarked_vowel_fallback() {
    assert_eq!(
        ipa(&arabic::word_segments("مَدّ", ArabicDialect::Msa)),
        ["m", "a", "d"]
    );
}

#[test]
fn dagger_alif_and_tanwin_are_realized_as_vowels_and_final_n() {
    let haadha = arabic::word_segments("هٰذَا", ArabicDialect::Msa);
    assert!(haadha.iter().any(|s| s.ipa == "a" && s.dur == Some(0.18)));

    let kitaaban = arabic::word_segments("كِتَابًا", ArabicDialect::Msa);
    assert_eq!(kitaaban.last().unwrap().ipa, "n");
}

#[test]
fn dialect_changes_arabic_phonology_without_changing_script_routing() {
    assert_eq!(
        ipa(&arabic::word_segments("جْقْ", ArabicDialect::Msa)),
        ["dʒ", "q"]
    );
    assert_eq!(
        ipa(&arabic::word_segments("جْقْ", ArabicDialect::Egyptian)),
        ["g", "ʔ"]
    );

    let g2p = EnglishG2P::new();
    let syllables = lyrics_in_with_dialect("كَتَبَ hello", &g2p, &MandarinLexicon::new(), Lang::Ar, ArabicDialect::Egyptian);
    assert!(
        syllables
            .iter()
            .any(|s| s.text.contains('ك') || s.text.contains('ت'))
    );
    assert!(
        syllables
            .iter()
            .any(|s| s.onset.first().is_some_and(|x| x.ipa == "h"))
    );
}

#[test]
fn arabic_lyrics_keep_syllables_on_written_letter_boundaries() {
    let syllables = lyrics_in_with_dialect(
        "أنا اسمي الفاروق",
        &EnglishG2P::new(),
        &MandarinLexicon::new(),
        Lang::Ar,
        ArabicDialect::Egyptian,
    );
    let spelling: Vec<&str> = syllables.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(spelling, ["أ", "نا", "اس", "مي", "ال", "فا", "روق"]);

    let marked = lyrics_in_with_dialect("كَتَبَ", &EnglishG2P::new(), &MandarinLexicon::new(), Lang::Ar, ArabicDialect::Msa);
    let spelling: Vec<&str> = marked.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(spelling, ["كَ", "تَ", "بَ"]);
}

#[test]
fn emphatic_letters_keep_a_separate_retracted_tongue_articulation() {
    assert_eq!(
        ipa(&arabic::word_segments("صَطَضَظَ", ArabicDialect::Msa)),
        ["sˤ", "a", "tˤ", "a", "dˤ", "a", "ðˤ", "a"]
    );
    for symbol in ["sˤ", "tˤ", "dˤ", "ðˤ"] {
        assert!(
            chimera::phonetics::inventory::phoneme(symbol, chimera::anatomy::human::BodyMix::MALE)
                .is_some()
        );
    }
    assert_eq!(
        ipa(&chimera::phonetics::ipa::tokenize_ipa("sˤ tˤ")),
        ["sˤ", "tˤ"]
    );
    assert_eq!(ipa(&chimera::phonetics::ipa::tokenize_ipa("sˤa")), ["sˤ", "a"]);
    assert_eq!(ipa(&chimera::phonetics::ipa::tokenize_ipa("s ˤa")), ["s", "a"]);
    assert!(chimera::phonetics::ipa::skipped_ipa("sˤ").is_empty());
    assert_eq!(ipa(&chimera::phonetics::ipa::tokenize_ipa("s\tˤa")), ["s", "a"]);
    assert_eq!(ipa(&chimera::phonetics::ipa::tokenize_ipa("s\u{00a0}ˤa")), ["s", "a"]);
}

#[test]
fn emphatics_retract_the_tongue_body_in_adjacent_arabic_vowels() {
    let segments = arabic::word_segments("صَسَ", ArabicDialect::Msa);
    assert_eq!(
        segments
            .iter()
            .filter(|s| s.pharyngealized == Some(true))
            .map(|s| s.ipa.as_str())
            .collect::<Vec<_>>(),
        ["a"]
    );

    let plain = arabic::word_segments("سَسَ", ArabicDialect::Msa);
    let options = PlanOptions::default();
    let plain_plan = plan_utterance(&plain, &options);
    let emphatic_plan = plan_utterance(&segments, &options);
    let first_vowel_root = |plan: &chimera::phonetics::Plan| {
        plan.gestures
            .iter()
            .find(|g| g.label_str() == "a")
            .and_then(|g| g.targets.get(ControlKey::Artic(ArticKey::Root)))
            .unwrap()
    };
    assert!(first_vowel_root(&emphatic_plan) < first_vowel_root(&plain_plan));
}

#[test]
fn arabic_tts_preserves_question_punctuation_and_mixed_english_words() {
    let phrases = arabic::text_to_phrases("كَتَبَ؟ hello.", &EnglishG2P::new(), ArabicDialect::Msa);
    assert_eq!(phrases.len(), 2);
    assert_eq!(
        phrases[0].intonation,
        chimera::phonetics::Intonation::Question
    );
    assert_eq!(phrases[0].words, ["كَتَبَ"]);
    assert_eq!(phrases[1].words, ["hello"]);
    assert!(phrases[0].segments.iter().any(|s| s.ipa == "a"));
    let mixed = arabic::text_to_phrases("hello 12 I’m مَرْحَبًا", &EnglishG2P::new(), ArabicDialect::Msa);
    let words: Vec<String> = mixed.into_iter().flat_map(|p| p.words).collect();
    assert!(words.iter().any(|w| w == "twelve"));
    assert!(words.iter().any(|w| w == "i'm"));
    let lyric_syllables = lyrics_in_with_dialect("مَرْحَبًا، كَيْفَ حَالُكَ؟", &EnglishG2P::new(), &MandarinLexicon::new(), Lang::Ar, ArabicDialect::Msa);
    assert_eq!(lyric_syllables.iter().filter(|s| s.phrase_end).count(), 2);
    let english = EnglishG2P::new().to_phrases("hello 12 I’m\ngot it");
    let arabic = arabic::text_to_phrases("hello 12 I’m\ngot it", &EnglishG2P::new(), ArabicDialect::Msa);
    assert_eq!(arabic.iter().flat_map(|p| p.segments.iter().map(|s| s.ipa.as_str())).collect::<Vec<_>>(), english.iter().flat_map(|p| p.segments.iter().map(|s| s.ipa.as_str())).collect::<Vec<_>>());
}

#[test]
fn marked_waw_and_ordinary_alif_lam_keep_their_phones() {
    assert_eq!(ipa(&arabic::word_segments("\u{642}\u{64f}\u{648}\u{64e}\u{651}\u{629}\u{64c}", ArabicDialect::Msa)), ["q", "u", "w", "a", "t", "u", "n"]);
    assert_eq!(ipa(&arabic::word_segments("\u{627}\u{650}\u{644}\u{0652}\u{62a}\u{64e}\u{642}\u{64e}\u{649}", ArabicDialect::Msa)), ["i", "l", "t", "a", "q", "a"]);
}

#[test]
fn singing_keeps_pharyngealized_arabic_vowels() {
    let syl = arabic::ArabicFrontEnd(ArabicDialect::Msa).word("\u{635}\u{64e}")[0].clone();
    assert_eq!(syl.nucleus.pharyngealized, Some(true));
    let song = plan_song(&[SungNote { midi: 60.0, beats: 1.0, syl: syl.clone() }], &SongOptions { body: "male".into(), bpm: 90.0, t0: 0.5, vibrato: 0.0, ..Default::default() });
    assert!(song.gestures.iter().any(|g| g.targets.get(ControlKey::Artic(ArticKey::Root)).is_some()));
    let live = plan_note_on(&syl, 60.0, &NoteOnOptions { body: "male".into(), vibrato: 0.0, tag: 1.0, legato_from: None });
    assert!(live.iter().any(|g| g.targets.get(ControlKey::Artic(ArticKey::Root)).is_some()));
}

#[test]
fn ordinary_alif_lam_is_not_assimilated() {
    for word in ["\u{0642}\u{064e}\u{0627}\u{0644}\u{064e}\u{062a}\u{0652}",
        "\u{062e}\u{064e}\u{0627}\u{0644}\u{0650}\u{062f}",
        "\u{0648}\u{064e}\u{0627}\u{0644}\u{0650}\u{062f}"] {
        assert!(ipa(&arabic::word_segments(word, ArabicDialect::Msa)).contains(&"l"));
    }
}

#[test]
fn ta_marbuta_is_short_in_pause_and_at_before_case_vowel() {
    let pause_segments = arabic::word_segments("\u{0645}\u{064e}\u{062f}\u{0631}\u{064e}\u{0633}\u{064e}\u{0629}", ArabicDialect::Msa);
    let pause = ipa(&pause_segments);
    assert_eq!(pause.last(), Some(&"a"));
    assert!(!arabic::word_segments("\u{0645}\u{064e}\u{062f}\u{0631}\u{064e}\u{0633}\u{064e}\u{0629}", ArabicDialect::Msa).iter().any(|s| s.dur == Some(0.18)));
    let case_segments = arabic::word_segments("\u{0645}\u{064e}\u{062f}\u{0631}\u{064e}\u{0633}\u{064e}\u{0629}\u{064f}", ArabicDialect::Msa);
    let case_form = ipa(&case_segments);
    assert!(case_form.windows(2).any(|pair| pair == ["t", "u"]));
}

#[test]
fn arabic_stress_and_onsets_follow_msa_rules() {
    let kataba = arabic::word_segments("\u{0643}\u{064e}\u{062a}\u{064e}\u{0628}\u{064e}", ArabicDialect::Msa);
    assert_eq!(kataba.iter().position(|s| s.stress == Some(1)), Some(1));
    let maktab = chimera::phonetics::arabic::ArabicFrontEnd(ArabicDialect::Msa)
        .word("\u{0645}\u{064e}\u{0643}\u{0652}\u{062a}\u{064e}\u{0628}");
    assert_eq!(maktab.len(), 2);
    assert_eq!(maktab[0].coda.last().map(|s| s.ipa.as_str()), Some("k"));
}
