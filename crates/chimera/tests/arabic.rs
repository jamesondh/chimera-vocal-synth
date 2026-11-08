//! Arabic: the reading of vowelled text. The expected readings are written by hand from the rules
//! of correct reading (Ḥafṣ ʿan ʿĀṣim where the Qurʾān differs from plain MSA).

use chimera::phonetics::arabic::{ArabicOptions, rules, to_ipa, to_phrases};
use chimera::phonetics::ipa::tokenize_ipa;

/// The reading without stress marks.
fn read(text: &str) -> String {
    to_ipa(text, &ArabicOptions::default()).replace('ˈ', "")
}

#[track_caller]
fn reads(text: &str, expected: &str) {
    assert_eq!(read(text), expected, "{text}");
}

#[test]
fn letters_vowels_and_length() {
    reads("كَتَبَ", "katab");
    reads("كَاتِبٌ", "kaːtib");
    reads("كِتَابٌ جَدِيدٌ", "kitaːbun dʒadiːd");
    reads("نُورٌ", "nuːrˤ");
    reads("بَيْتٌ", "bajt");
    reads("يَوْمٌ", "jawm");
    reads("ثَلْجٌ", "θaldʒ");
    reads("ذَهَبٌ", "ðahab");
}

#[test]
fn shadda_doubles_one_segment() {
    reads("عَلَّمَ", "ʕalːam");
    reads("دَرَّسَ", "darˤːaˤs");
    reads("قُوَّةٌ", "quˤwːah");
    reads("سَيِّدٌ", "sajːid");
    reads("أُمٌّ", "ʔumː");
}

#[test]
fn hamza_on_every_seat_and_maddah() {
    reads("سَأَلَ", "saʔal");
    reads("سُئِلَ", "suʔil");
    reads("مُؤْمِنٌ", "muʔmin");
    reads("مَاءٌ", "maːʔ");
    reads("سَمَاءً", "samaːʔaː");
    reads("آمَنَ", "ʔaːman");
    reads("الْقُرْآنُ", "ʔalquˤrˤʔaːn");
}

#[test]
fn the_article_before_moon_and_sun_letters() {
    reads("الْقَمَرُ", "ʔalqaˤmarˤ");
    reads("الشَّمْسُ", "ʔaʃːams");
    reads("وَالشَّمْسِ", "waʃːams");
    reads("بِالسَّيَّارَةِ", "bisːajːaːɾˤaˤh");
    reads("لِلنَّاسِ", "linːaːs");
    // unvowelled, the sun letter is doubled all the same
    reads("الشام", "ʔaʃːaːm");
}

#[test]
fn hamzat_al_wasl_is_said_only_at_the_start() {
    reads("اِقْرَأْ", "ʔiqrˤaˤʔ");
    reads("ٱقْرَأْ", "ʔiqrˤaˤʔ");
    reads("ٱدْعُ", "ʔudʕ");
    reads("ٱبْنٌ", "ʔibn");
    reads("الَّذِي", "ʔalːaðiː");
    reads("هُوَ الَّذِي", "huwa lːaðiː");
    // a long vowel before it shortens; a sākin end takes a joining vowel
    reads("فِي الْبَيْتِ", "fi lbajt");
    reads("عَلَى السَّمَاءِ", "ʕala sːamaːʔ");
    reads("مِنْ الْبَيْتِ", "mina lbajt");
    reads("كِتَابٌ الْيَوْمَ", "kitaːbuni ljawm");
}

#[test]
fn waqf_drops_final_vowels_and_reads_taa_marbuta_as_h() {
    reads("مَدْرَسَةٌ", "madrˤaˤsah");
    reads("مَدْرَسَةُ الْبَنَاتِ", "madrˤaˤsatu lbanaːt");
    reads("شُكْرًا", "ʃukrˤaˤː");
    reads("هُدًى", "hudaː");
    let joined = to_ipa("مَدْرَسَةٌ", &ArabicOptions { waqf: false, ..Default::default() }).replace('ˈ', "");
    assert_eq!(joined, "madrˤaˤsatun");
}

#[test]
fn heavy_letters_and_their_vowels() {
    reads("صَبَاحُ الْخَيْرِ", "sˤaˤbaːħu lχaˤjr");
    reads("قَالَ", "qaˤːl");
    reads("طِينٌ", "tˤiˤːn");
    reads("ضَرَبَ", "dˤaˤɾˤaˤb");
    reads("ظَلَّ", "ðˤaˤlː");
    reads("غَابَ", "ʁaˤːb");
}

#[test]
fn raa_is_heavy_or_light_by_its_vowels_and_trilled_at_word_edges() {
    // a tap between the vowels of a word, a short trill elsewhere
    reads("بَحْرٌ", "baħrˤ"); // sākin after a sākin after fatḥa: heavy
    reads("رَأَى", "rˤaˤʔaː");
    reads("مَرَّةً", "marˤːaˤh");
    reads("رِجَالٌ", "ridʒaːl");
    reads("رَبٌّ", "rˤaˤbː");
    reads("فِرْعَوْنُ", "firʕawn");
    reads("مِرْصَادٌ", "mirˤsˤaˤːd");
    reads("خَيْرٌ", "χaˤjr");
    reads("ٱلْقَدْرِ", "ʔalqaˤdrˤ");
    reads("ٱرْجِعِي", "ʔirˤdʒiʕiː");
}

#[test]
fn the_name_allah_and_hidden_long_vowels() {
    reads("اللَّهُ أَكْبَرُ", "ʔaɫːaˤːhu ʔakbarˤ");
    reads("بِسْمِ اللَّهِ", "bismi lːaːh");
    reads("قُلْ هُوَ اللَّهُ أَحَدٌ", "quˤl huwa ɫːaˤːhu ʔaħad");
    reads("لِلَّهِ", "lilːaːh");
    reads("وَاللَّهِ", "waɫːaˤːh");
    reads("هَذَا كِتَابٌ", "haːðaː kitaːb");
    reads("ذَلِكَ", "ðaːlik");
    reads("لَكِنَّ", "laːkinː");
}

#[test]
fn the_basmala_and_the_fatiha_in_uthmani_script() {
    reads("بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ", "bismi lːaːhi rˤːaˤħmaːni rˤːaˤħiːm");
    reads("ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ", "ʔalħamdu lilːaːhi rˤaˤbːi lʕaːlamiːn");
    reads("مَٰلِكِ يَوْمِ ٱلدِّينِ", "maːliki jawmi dːiːn");
    reads("إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ", "ʔijːaːka naʕbudu waʔijːaːka nastaʕiːn");
    reads("ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ", "ʔihdina sˤːiˤɾˤaˤːtˤaˤ lmustaqiˤːm");
}

#[test]
fn surat_al_ikhlas() {
    // 112:1–4, each āya read in pause; in 112:4 the nūn of يَكُن merges into the lām (idghām
    // without ghunna: yakul-lahū)
    let text = "قُلْ هُوَ ٱللَّهُ أَحَدٌ ۝ ٱللَّهُ ٱلصَّمَدُ ۝ لَمْ يَلِدْ وَلَمْ يُولَدْ ۝ وَلَمْ يَكُن لَّهُۥ كُفُوًا أَحَدٌۢ";
    assert_eq!(read(text), "quˤl huwa ɫːaˤːhu ʔaħad | ʔaɫːaˤːhu sˤːaˤmad | lam jalid walam juːlad | walam jaku lːahuː kufuwan ʔaħad");
    reads("مِن رَّبِّهِمْ", "mi rˤːaˤbːihim");
    let p = &to_phrases("وَلَمْ يَكُن لَّهُۥ", &ArabicOptions::default())[0];
    assert!(p.segments.iter().zip(&p.rules).any(|(s, r)| s.ipa == "l" && r.contains(&rules::IDGHAM)), "{:?}", p.rules);
}

#[test]
fn unvowelled_letters_are_guessed_and_counted() {
    let p = &to_phrases("كتب", &ArabicOptions::default())[0];
    assert_eq!(p.ipa.replace('ˈ', ""), "katab");
    assert_eq!(p.guessed, 2);
    let p = &to_phrases("كَتَبَ", &ArabicOptions::default())[0];
    assert_eq!(p.guessed, 0);
}

#[test]
fn phrases_split_at_punctuation_and_questions_rise() {
    let ps = to_phrases("السَّلَامُ عَلَيْكُمْ. كَيْفَ حَالُكَ؟", &ArabicOptions::default());
    assert_eq!(ps.len(), 2);
    assert_eq!(ps[0].ipa.replace('ˈ', ""), "ʔasːalaːmu ʕalajkum");
    assert_eq!(ps[1].ipa.replace('ˈ', ""), "kajfa ħaːluk");
    assert_eq!(ps[1].intonation, chimera::phonetics::Intonation::Question);
    assert_eq!(ps[0].words, vec!["السَّلَامُ", "عَلَيْكُمْ"]);
}

#[test]
fn each_segment_knows_its_letters_and_rules() {
    let text = "ٱلرَّحْمَٰنِ";
    let p = &to_phrases(text, &ArabicOptions::default())[0];
    assert_eq!(p.segments.len(), p.sources.len());
    assert_eq!(p.segments.len(), p.rules.len());
    let chars: Vec<char> = text.chars().collect();
    // the doubled rāʾ covers the silent lām of the article too
    let r = p.segments.iter().position(|s| s.ipa == "rˤ").unwrap();
    let [a, b] = p.sources[r];
    assert_eq!(chars[a..b].iter().collect::<String>(), "لرَّ");
    assert!(p.rules[r].contains(&rules::SHAMSIYYA));
    assert!(p.rules[r].contains(&rules::TAFKHIM));
    // the long ā of the dagger alif is a madd of two counts
    let m = p.segments.iter().position(|s| s.ipa == "m").unwrap();
    assert_eq!(p.counts[m + 1], 2.0);
    assert!(p.rules[m + 1].contains(&rules::MADD));
}

#[test]
fn tajweed_contexts_are_tagged() {
    let o = ArabicOptions::default();
    let tagged = |text: &str, ipa: &str, rule: &str| {
        let p = &to_phrases(text, &o)[0];
        let ok = p.segments.iter().zip(&p.rules).any(|(s, r)| s.ipa.trim_end_matches('ˤ') == ipa && r.contains(&rule));
        assert!(ok, "{text}: no {ipa} with {rule} in {:?}", p.segments.iter().zip(&p.rules).map(|(s, r)| (s.ipa.clone(), r.clone())).collect::<Vec<_>>());
    };
    tagged("جَاءَ", "a", rules::MADD_MUTTASIL);
    tagged("فِي أَنْفُسِكُمْ", "i", rules::MADD_MUNFASIL);
    tagged("ٱلضَّآلِّينَ", "a", rules::MADD_LAZIM);
    tagged("ٱلرَّحِيمِ", "i", rules::MADD_ARID);
    tagged("إِنَّ", "n", rules::GHUNNA);
    tagged("مِنْ بَعْدِ", "n", rules::NUN_SAKIN);
    tagged("أَحَدٌ", "d", rules::QALQALA);
    tagged("يَقْطَعُونَ", "q", rules::QALQALA);
}

#[test]
fn a_reading_tokenizes_back_into_its_segments() {
    for text in ["بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ", "صَبَاحُ الْخَيْرِ", "قُلْ هُوَ اللَّهُ أَحَدٌ"] {
        let p = &to_phrases(text, &ArabicOptions::default())[0];
        let back: Vec<String> = tokenize_ipa(&p.ipa).into_iter().map(|s| s.ipa).collect();
        let segs: Vec<String> = p.segments.iter().map(|s| s.ipa.clone()).collect();
        assert_eq!(back, segs, "{text}");
    }
}

#[test]
fn a_doubled_consonant_holds_longer_than_a_single_one() {
    let dur = |text: &str, ipa: &str| {
        let p = &to_phrases(text, &ArabicOptions::default())[0];
        p.segments.iter().find(|s| s.ipa == ipa).unwrap().dur.unwrap()
    };
    assert!(dur("دَرَّسَ", "rˤ") >= 1.5 * dur("دَرَسَ", "ɾˤ"));
    assert!(dur("عَلَّمَ", "l") >= 1.5 * dur("عَلِمَ", "l"));
    // a doubled nasal is held two counts (its ghunna)
    assert!((dur("إِنَّ", "n") - 2.0 * ArabicOptions::default().count).abs() < 1e-9);
}
