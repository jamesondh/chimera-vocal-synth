//! Polish G2P: letters to sounds, softening i, nasal vowels, voicing, stress, phrases, and the
//! lyrics front end.
use chimera::anatomy::human::BodyMix;
use chimera::phonetics::inventory::phoneme;
use chimera::phonetics::mandarin::MandarinLexicon;
use chimera::phonetics::planner::{PhraseInput, PlanOptions, is_vowel, plan_phrases};
use chimera::phonetics::polish::{lyrics_to_syllables, to_phrases, word};
use chimera::phonetics::sing::{SongOptions, SungNote, plan_song};
use chimera::phonetics::syllables::{Lang, lyrics_in};
use chimera::phonetics::{Intonation, Segment, Syllable};

/// Segments as IPA: a stress mark before the stressed vowel, ʲ after a palatalized consonant, a
/// tilde after a nasalized vowel, ː after a long sound, and a space after each word.
fn shown(segs: &[Segment]) -> String {
    segs.iter()
        .map(|s| {
            let long = s.length.is_some_and(|k| k > 1.5);
            format!(
                "{}{}{}{}{}{}",
                if s.stress == Some(1) { "ˈ" } else { "" },
                s.ipa,
                if s.palatalized == Some(true) { "ʲ" } else { "" },
                if s.nasalized == Some(true) { "\u{303}" } else { "" },
                if long { "ː" } else { "" },
                if s.word_end == Some(true) { " " } else { "" }
            )
        })
        .collect::<String>()
        .trim_end()
        .to_string()
}
/// A text as the CLI shows it: phrases joined by " | ".
fn pl(text: &str) -> String {
    to_phrases(text).iter().map(|p| shown(&p.segments)).collect::<Vec<_>>().join(" | ")
}
fn w(typed: &str) -> String {
    shown(&word(typed))
}

#[test]
fn dzien_dobry() {
    assert_eq!(pl("Dzień dobry"), "dʑˈɛɲ dˈɔbɾɨ");
}

#[test]
fn digraphs_are_one_sound() {
    assert_eq!(w("szum"), "ʃˈum");
    assert_eq!(w("czas"), "tʃˈas");
    assert_eq!(w("rzeka"), "ʒˈɛka");
    assert_eq!(w("żaba"), "ʒˈaba");
    assert_eq!(w("dżem"), "dʒˈɛm");
    assert_eq!(w("chata"), "xˈata");
    assert_eq!(w("dzwon"), "dzvˈɔn");
    assert_eq!(w("łódź"), "wˈutɕ");
    // rz is r and z in a few words, and only in the listed stem: prze- keeps its [ʃ]
    assert_eq!(w("marznąć"), "mˈaɾznɔɲtɕ");
    assert_eq!(w("przemarznąć"), "pʃɛmˈaɾznɔɲtɕ");
}

#[test]
fn i_before_a_vowel_only_softens() {
    assert_eq!(w("się"), "ɕɛ");
    assert_eq!(w("ziemia"), "ʑˈɛmʲa");
    assert_eq!(w("ciało"), "tɕˈawɔ");
    assert_eq!(w("niebo"), "ɲˈɛbɔ");
    assert_eq!(w("pies"), "pʲˈɛs");
    assert_eq!(w("kiedy"), "cˈɛdɨ");
    assert_eq!(w("chiński"), "xʲˈiɲsci");
    // before a consonant or at the end it is also a vowel
    assert_eq!(w("nic"), "ɲˈits");
    assert_eq!(w("dzisiaj"), "dʑˈiɕaj");
    assert_eq!(w("kino"), "cˈinɔ");
    // in loans t, d and r take a glide
    assert_eq!(w("radio"), "ɾˈadjɔ");
    assert_eq!(w("historia"), "xʲistˈɔɾja");
    // after sz, cz, ż it is a vowel
    assert_eq!(w("szia"), "ʃˈia");
}

#[test]
fn nasal_vowels_follow_the_next_sound() {
    // a nasal of the stop's place
    assert_eq!(w("ręka"), "ɾˈɛŋka");
    assert_eq!(w("kąt"), "kˈɔnt");
    assert_eq!(w("pięć"), "pʲˈɛɲtɕ");
    assert_eq!(w("dęby"), "dˈɛmbɨ");
    // nasalized before a fricative and at the end
    assert_eq!(w("mąż"), "mˈɔ\u{303}ʃ");
    assert_eq!(w("są"), "sˈɔ\u{303}");
    assert_eq!(w("gęsi"), "gˈɛ\u{303}ɕi");
    // oral before l and ł, and a final ę
    assert_eq!(w("wziął"), "vʑˈɔw");
    assert_eq!(w("zaczęli"), "zatʃˈɛlʲi");
    assert_eq!(w("idę"), "ˈidɛ");
}

#[test]
fn voicing_assimilates() {
    // a word's end devoices
    assert_eq!(w("chleb"), "xlˈɛp");
    assert_eq!(w("ogród"), "ˈɔgɾut");
    assert_eq!(w("ząb"), "zˈɔmp");
    // ... also before a word-final sonorant
    assert_eq!(w("bóbr"), "bˈupɾ");
    assert_eq!(w("jaźń"), "jˈaɕɲ");
    // an obstruent takes the voicing of the next
    assert_eq!(w("wódka"), "vˈutka");
    assert_eq!(w("prośba"), "pɾˈɔʑba");
    assert_eq!(w("także"), "tˈagʒɛ");
    assert_eq!(w("wszystko"), "fʃˈɨstkɔ");
    // w and rz devoice after a voiceless one, and don't voice it
    assert_eq!(w("kwiat"), "kfʲˈat");
    assert_eq!(w("twój"), "tfˈuj");
    assert_eq!(w("krzak"), "kʃˈak");
    assert_eq!(w("trzy"), "tʃˈɨ");
    assert_eq!(w("dwa"), "dvˈa");
}

#[test]
fn a_word_without_a_vowel_goes_with_the_next() {
    assert_eq!(pl("w domu"), "vdˈɔmu");
    assert_eq!(pl("w Polsce"), "fpˈɔlstsɛ");
    assert_eq!(pl("z tobą"), "stˈɔbɔ\u{303}");
    assert_eq!(pl("W czwartek trzej szczęśliwi chłopcy grzybów szukają w trzcinie."), "ftʃfˈaɾtɛk tʃˈɛj ʃtʃɛ\u{303}ɕlʲˈivʲi xwˈɔptsɨ gʒˈɨbuf ʃukˈajɔ\u{303} ftʃtɕˈiɲɛ");
    let ph = to_phrases("Jestem w domu.");
    assert_eq!(ph[0].words, ["jestem", "w domu"]);
}

#[test]
fn a_tap_after_t_or_d_gets_a_brief_vowel_in_speech() {
    // the tap merged into the stop's closure and the voice came back only in the vowel: "tramwaj"
    // sounded like "tamwaj"
    assert_eq!(w("tramwaj"), "təɾˈamvaj");
    assert_eq!(w("droga"), "dəɾˈɔga");
    let lead = word("trawa")[1].clone();
    assert_eq!((lead.ipa.as_str(), lead.dur, lead.stress), ("ə", Some(0.05), Some(0)));
    // after other consonants the tap is heard as it is
    assert_eq!(w("prawda"), "pɾˈavda");
    assert_eq!(w("Kraków"), "kɾˈakuf");
    // a sung note's onset stays a cluster
    let syl = lyrics_to_syllables("tramwaj");
    assert_eq!(parts(&syl[0]), "tɾ.a.m");
}

#[test]
fn doubled_letters_are_one_long_consonant() {
    assert_eq!(w("Anna"), "ˈanːa");
    assert_eq!(w("lekki"), "lˈɛcːi");
}

#[test]
fn stress_is_on_the_penult() {
    assert_eq!(w("dziękuję"), "dʑɛŋkˈujɛ");
    assert_eq!(w("Szczebrzeszynie"), "ʃtʃɛbʒɛʃˈɨɲɛ");
    assert_eq!(w("nauka"), "naˈuka");
    // au and eu starting a word are a diphthong in loans
    assert_eq!(w("auto"), "ˈawtɔ");
    assert_eq!(w("Europa"), "ɛwɾˈɔpa");
}

#[test]
fn stress_exceptions() {
    // past tense -liśmy, -łyście
    assert_eq!(w("byliśmy"), "bˈɨlʲiɕmɨ");
    assert_eq!(w("robiłyście"), "ɾɔbʲˈiwɨɕtɕɛ");
    // the conditional: one further back for -byśmy
    assert_eq!(w("chciałbym"), "xtɕˈawbɨm");
    assert_eq!(w("zrobiłaby"), "zɾɔbʲˈiwabɨ");
    assert_eq!(w("zrobilibyśmy"), "zɾɔbʲˈilʲibɨɕmɨ");
    // ... but not a word that only ends like one
    assert_eq!(w("jesteśmy"), "jɛstˈɛɕmɨ");
    assert_eq!(w("szczęście"), "ʃtʃˈɛ\u{303}ɕtɕɛ");
    assert_eq!(w("gdyby"), "gdˈɨbɨ");
    // loans in -yka/-ika, and listed words
    assert_eq!(w("matematyka"), "matɛmˈatɨka");
    assert_eq!(w("muzyce"), "mˈuzɨtsɛ");
    assert_eq!(w("uniwersytet"), "uɲivˈɛɾsɨtɛt");
}

#[test]
fn clitics_take_no_stress() {
    assert_eq!(pl("jak się masz"), "jˈak ɕɛ mˈaʃ");
    assert_eq!(pl("na stole"), "na stˈɔlɛ");
    assert_eq!(pl("Czy mówisz po polsku?"), "tʃɨ mˈuvʲiʃ pɔ pˈɔlsku");
}

#[test]
fn yes_no_questions_rise_and_wh_questions_fall() {
    let ph = to_phrases("Czy mówisz po polsku? Gdzie jesteś? Masz czas? A gdzie jesteś?");
    let tunes: Vec<Intonation> = ph.iter().map(|p| p.intonation).collect();
    // a conjunction before the question word doesn't make it rise
    assert_eq!(tunes, [Intonation::Question, Intonation::Statement, Intonation::Question, Intonation::Statement]);
}

#[test]
fn y_is_a_full_vowel_in_speech() {
    let segs = word("były");
    let y = segs.iter().find(|s| s.ipa == "ɨ").unwrap();
    assert_eq!(y.length, Some(1.25));
}

#[test]
fn punctuation_makes_phrases_with_one_word_per_word_end() {
    let ph = to_phrases("Dzień dobry, jak się masz? Dobrze, dziękuję.");
    assert_eq!(ph.len(), 4);
    assert_eq!(ph[0].words, ["dzień", "dobry"]);
    for p in &ph {
        assert_eq!(p.words.len(), p.segments.iter().filter(|s| s.word_end == Some(true)).count());
    }
    assert_eq!(ph[0].pause, 0.18);
    assert_eq!(ph[1].pause, 0.35);
}

#[test]
fn vowels_are_as_short_as_a_polish_speakers() {
    // the duration rules are English's: before, a stressed vowel lasted 144 ms on average and a
    // phrase's last 198, where a Polish speaker's lasted 88 and 107 ("masz" was 290 ms)
    let ph = to_phrases("Dzień dobry, jak się masz? Dziękuję bardzo. Jedzie tramwaj, czy to prawda?");
    let inputs: Vec<PhraseInput> = ph.iter().map(|p| PhraseInput { segments: p.segments.clone(), intonation: Some(p.intonation), pause: Some(p.pause) }).collect();
    let plan = plan_phrases(&inputs, &PlanOptions { body: BodyMix::MALE, ..Default::default() });
    let full = |s: &str| matches!(s, "a" | "ɛ" | "i" | "ɔ" | "u");
    let (mut stressed, mut unstressed, mut last) = (Vec::new(), Vec::new(), Vec::new());
    let mut k = 0;
    for p in &ph {
        let segs = &plan.segments[k..k + p.segments.len()];
        k += p.segments.len();
        let end = segs.iter().rposition(|s| full(&s.ipa) || s.ipa == "ɨ").unwrap();
        for (i, s) in segs.iter().enumerate().filter(|(_, s)| full(&s.ipa)) {
            let d = (s.t1 - s.t0) * 1000.0;
            if i == end {
                last.push(d)
            } else if s.stress == Some(1) {
                stressed.push(d)
            } else {
                unstressed.push(d)
            }
        }
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let (st, un, la) = (mean(&stressed), mean(&unstressed), mean(&last));
    assert!((70.0..110.0).contains(&st), "stressed {st:.0} ms");
    assert!((50.0..80.0).contains(&un), "unstressed {un:.0} ms");
    assert!((85.0..135.0).contains(&la), "a phrase's last {la:.0} ms");
}

#[test]
fn every_sound_is_one_the_voice_can_make() {
    let text = "Pchnąć w tę łódź jeża lub ośm skrzyń fig. Zażółć gęślą jaźń! Dziękuję bardzo, trzej szczęśliwi chłopcy grzybów szukają w trzcinie.";
    let body = BodyMix::MALE;
    let phrases = to_phrases(text);
    for s in phrases.iter().flat_map(|p| &p.segments) {
        assert!(is_vowel(&s.ipa, body) || phoneme(&s.ipa, body).is_some(), "unknown sound {}", s.ipa);
    }
    let inputs: Vec<PhraseInput> = phrases.iter().map(|p| PhraseInput { segments: p.segments.clone(), intonation: Some(p.intonation), pause: Some(p.pause) }).collect();
    let plan = plan_phrases(&inputs, &PlanOptions { body, ..Default::default() });
    assert!(!plan.gestures.is_empty());
    assert_eq!(plan.segments.len(), inputs.iter().map(|p| p.segments.len()).sum::<usize>());
}

// ---------------------------------------------------------------- singing

fn sung(text: &str) -> Vec<String> {
    lyrics_to_syllables(text).iter().map(|s| s.text.clone()).collect()
}
fn parts(s: &Syllable) -> String {
    let ipa = |v: &[Segment]| v.iter().map(|x| x.ipa.as_str()).collect::<String>();
    format!("{}.{}.{}", ipa(&s.onset), s.nucleus.ipa, ipa(&s.coda))
}

#[test]
fn lyrics_split_into_open_syllables() {
    assert_eq!(sung("Dzień dobry"), ["Dzień", "do", "bry"]);
    assert_eq!(sung("Szczebrzeszynie"), ["Szcze", "brze", "szy", "nie"]);
    // a sonorant before a consonant stays with the vowel before
    assert_eq!(sung("Polska ręka"), ["Pol", "ska", "rę", "ka"]);
    let syl = lyrics_to_syllables("ręka Anna");
    let p: Vec<String> = syl.iter().map(parts).collect();
    assert_eq!(&p[..2], ["ɾ.ɛ.ŋ", "k.a."]);
    // a long consonant closes one sung syllable and opens the next
    assert_eq!(&p[2..], [".a.n", "n.a."]);
    assert_eq!(sung("Anna"), ["An", "na"]);
    // sung lengths come from the notes
    assert!(syl.iter().all(|s| s.onset.iter().chain(&s.coda).chain([&s.nucleus]).all(|c| c.dur.is_none() && c.length.is_none())));
}

#[test]
fn hyphens_only_place_the_breaks() {
    // the word is read whole: ręka's nasal [ŋ] and prośba's voiced [ʑ] stay, and the breaks are
    // where the hyphens are
    let p = |t: &str| lyrics_to_syllables(t).iter().map(parts).collect::<Vec<_>>();
    assert_eq!(p("rę-ka"), p("ręka"));
    assert_eq!(p("proś-ba"), ["pɾ.ɔ.ʑ", "b.a."]);
    assert_eq!(p("prośba"), ["pɾ.ɔ.", "ʑb.a."]);
    assert_eq!(p("An-na"), p("Anna"));
    assert_eq!(sung("do-bry"), ["do", "bry"]);
    assert_eq!(sung("dob-ry"), ["dob", "ry"]);
}

#[test]
fn a_vowelless_word_starts_the_next_sung_syllable() {
    let syl = lyrics_to_syllables("w domu, w trzcinie");
    assert_eq!(syl.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(), ["w do", "mu", "w trzci", "nie"]);
    assert_eq!(parts(&syl[0]), "vd.ɔ.");
    assert_eq!(parts(&syl[2]), "ftʃtɕ.i.");
    assert!(syl[1].word_end && syl[1].phrase_end);
}

#[test]
fn latin_script_is_polish_only_when_chosen() {
    let no_dict = |_: &str, _: bool| Vec::<Segment>::new();
    let text = |lang| lyrics_in("pół późno", &no_dict, &MandarinLexicon::new(), lang).into_iter().map(|s| s.text).collect::<Vec<_>>();
    // ó is a pinyin tone mark too: in Polish, pinyin doesn't get a piece of the word
    assert_eq!(text(Lang::Pl), ["pół", "pó", "źno"]);
    assert!(text(Lang::En).iter().all(|t| !t.contains('ł')));
    // other scripts still go to their own front ends
    let mixed = lyrics_to_syllables("dom 빵야 дом");
    assert_eq!(mixed.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(), ["dom", "빵", "야", "дом"]);
}

#[test]
fn polish_lyrics_plan_as_a_song() {
    let syl = lyrics_to_syllables("Sto lat, sto lat, niech żyje, żyje nam.");
    let notes: Vec<SungNote> = syl.iter().enumerate().map(|(i, s)| SungNote { midi: 60.0 + (i % 5) as f64, beats: 1.0, syl: s.clone() }).collect();
    let bpm = 100.0;
    let plan = plan_song(&notes, &SongOptions { body: BodyMix::FEMALE, bpm, t0: 0.35, vibrato: 35.0 });
    assert!(plan.end > 0.35 + notes.len() as f64 * 60.0 / bpm - 0.2, "{}", plan.end);
    assert!(plan.segments.iter().any(|s| s.ipa == "ɨ"));
}
