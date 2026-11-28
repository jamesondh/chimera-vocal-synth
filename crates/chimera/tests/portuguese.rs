//! Portuguese (Brazil) G2P: letters to sounds, the three r's, the closed vowels, the stress, the
//! numbers, the phrases and the lyrics front end.
use chimera::anatomy::human::BodyMix;
use chimera::phonetics::inventory::phoneme;
use chimera::phonetics::mandarin::MandarinLexicon;
use chimera::phonetics::planner::{PhraseInput, PlanOptions, is_vowel, plan_phrases};
use chimera::phonetics::portuguese::{lyrics_to_syllables, to_phrases, word};
use chimera::phonetics::syllables::{Lang, lyrics_in};
use chimera::phonetics::{Intonation, Segment, Syllable};

/// Segments as IPA, a stress mark before the stressed vowel and a space between the sounds.
fn shown(segs: &[Segment]) -> String {
    segs.iter()
        .map(|s| format!("{}{}", if s.stress == Some(1) { "ˈ" } else { "" }, s.ipa))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A text as the CLI shows it: phrases joined by " | ".
fn pt(text: &str) -> String {
    to_phrases(text).iter().map(|p| shown(&p.segments)).collect::<Vec<_>>().join(" | ")
}

/// One word.
fn w(typed: &str) -> String {
    shown(&word(typed))
}

/// The words a text is read as (the numbers are read out).
fn words(text: &str) -> String {
    to_phrases(text).iter().flat_map(|p| p.words.clone()).collect::<Vec<_>>().join(" ")
}

#[test]
fn letters_map_to_sounds() {
    assert_eq!(w("chave"), "ʃ ˈa v i");
    assert_eq!(w("filho"), "f ˈi ʎ u");
    assert_eq!(w("unha"), "ˈu ɲ ɐ");
    assert_eq!(w("guerra"), "g ˈɛ ʁ ɐ");
    assert_eq!(w("braço"), "b ɾ ˈa s u");
    assert_eq!(w("quinze"), "k ˈi\u{303} n z i");
    assert_eq!(w("homem"), "ˈo m ɛ̃ m");
}

#[test]
fn the_three_rs() {
    // at the start of a word: the uvular fricative
    assert_eq!(w("real"), "ʁ e ˈa w");
    // in the onset before a vowel: the tap
    assert_eq!(w("carioca"), "k a ɾ i ˈo k ɐ");
    assert_eq!(w("coração"), "k o ɾ a s ˈɐ̃w");
    assert_eq!(w("braço"), "b ɾ ˈa s u");
    // before a consonant and at the end of a word: the uvular again
    assert_eq!(w("guerra"), "g ˈɛ ʁ ɐ");
    assert_eq!(w("mar"), "m ˈa ʁ");
    assert_eq!(w("Português"), "p u ʁ t u g ˈɛ s");
}

#[test]
fn the_s_and_the_v() {
    // [z] between vowels, [s] elsewhere
    assert_eq!(w("casa"), "k ˈa z ɐ");
    assert_eq!(w("sapato"), "s a p ˈa t u");
    // the final s after a front vowel stays [s]
    assert_eq!(w("três"), "t ɾ ˈɛ s");
    // [v] everywhere — Portuguese has no [β]
    assert_eq!(w("você"), "v o s ˈe");
    assert_eq!(w("Brasil"), "b ɾ a z ˈi w");
}

#[test]
fn ti_and_di_are_the_palatal_affricates() {
    assert_eq!(w("tia"), "tʃ ˈi ɐ");
    assert_eq!(w("dia"), "dʒ ˈi ɐ");
}

#[test]
fn the_closed_vowels() {
    // an unstressed /a e o/ in a closed syllable, and a word-final one, are short and central
    assert_eq!(w("casa"), "k ˈa z ɐ");
    assert_eq!(w("mesa"), "m ˈe z ɐ");
    // a word-final /o/ rises to [u]
    assert_eq!(w("sapato"), "s a p ˈa t u");
    // a stressed /a/ stays open
    assert_eq!(w("mar"), "m ˈa ʁ");
}

#[test]
fn the_nasal_diphthongs_are_one_nucleus() {
    assert_eq!(w("pão"), "p ˈɐ̃w");
    assert_eq!(w("mãe"), "m ˈɐ̃j");
    assert_eq!(w("coração"), "k o ɾ a s ˈɐ̃w");
    // a vowel before a nasal in its coda is nasalised
    assert_eq!(w("homem"), "ˈo m ɛ̃ m");
    assert_eq!(w("quinze"), "k ˈi\u{303} n z i");
}

#[test]
fn the_stress_rules() {
    // a written accent wins
    assert_eq!(w("café"), "k a f ˈe");
    assert_eq!(w("único"), "ˈu n i k u");
    // a word ending in a nasal vowel falls on it
    assert_eq!(w("irmão"), "i ʁ m ˈɐ̃w");
    assert_eq!(w("manhã"), "m a ɲ ˈa\u{303}");
    // everything else: the penult
    assert_eq!(w("casa"), "k ˈa z ɐ");
    assert_eq!(w("Português"), "p u ʁ t u g ˈɛ s");
    // the endings that carry their own stress
    assert_eq!(w("falou"), "f a l ˈow");
    assert_eq!(w("falei"), "f a l ˈej");
    assert_eq!(w("falaria"), "f a l ˈa ɾ i ɐ");
    // -mente carries the stress
    assert_eq!(w("rapidamente"), "ʁ a p i d a m ˈɛ̃ n tʃ i");
}

// ------------------------------------------------------- the rules, one word each
//
// Every rule the native ear confirmed, held to the IPA it agreed with.

#[test]
fn the_v_is_v_everywhere() {
    // Portuguese keeps [v] everywhere; the [β] of the Spanish b/v is not ours
    assert_eq!(w("nova"), "n ˈo v ɐ");
    assert_eq!(w("chave"), "ʃ ˈa v i");
}

#[test]
fn the_palatal_affricates_change_before_i_only() {
    // before [i] — the letter "i" and the word-final unstressed "e"
    assert_eq!(w("noite"), "n ˈoj tʃ i");
    assert_eq!(w("tarde"), "t ˈa ʁ dʒ i");
    // before a sounded [e ɛ] the plain [t] and [d] stay
    assert_eq!(w("tempo"), "t ˈɛ\u{303} m p u");
    assert_eq!(w("tela"), "t ˈe l ɐ");
}

#[test]
fn the_word_final_unstressed_e_is_i() {
    assert_eq!(w("fome"), "f ˈo m i");
    assert_eq!(w("dente"), "d ˈɛ\u{303} n tʃ i");
}

#[test]
fn the_coda_l_is_w() {
    assert_eq!(w("alto"), "ˈa w t u");
    assert_eq!(w("calma"), "k ˈa w m ɐ");
}

#[test]
fn the_final_s_is_s() {
    // before a pause it is [s]; [z] belongs before a vowel across a word boundary
    assert_eq!(w("casas"), "k ˈa z ɐ s");
}

#[test]
fn the_oxytone_endings_fall_on_the_last_syllable() {
    assert_eq!(w("Brasil"), "b ɾ a z ˈi w");
    assert_eq!(w("feliz"), "f e l ˈi z");
    assert_eq!(w("aqui"), "a k ˈi");
    assert_eq!(w("comum"), "k o m ˈu\u{303} m");
}

#[test]
fn the_ia_ending_falls_on_the_right_syllable() {
    assert_eq!(w("Maria"), "m a ɾ ˈi ɐ");
    assert_eq!(w("academia"), "a k a d ˈe m i ɐ");
    assert_eq!(w("falaria"), "f a l ˈa ɾ i ɐ");
}

#[test]
fn the_mente_carries_the_stress() {
    assert_eq!(w("rapidamente"), "ʁ a p i d a m ˈɛ\u{303} n tʃ i");
}

#[test]
fn the_plain_diphthongs_are_one_nucleus() {
    assert_eq!(w("pai"), "p ˈaj");
    assert_eq!(w("meu"), "m ˈew");
    assert_eq!(w("coisa"), "k ˈoj z ɐ");
}

#[test]
fn a_monosyllabic_stressed_a_closes() {
    assert_eq!(w("já"), "ʒ ˈɐ");
}

#[test]
fn numbers_are_read_out() {
    assert_eq!(words("123"), "cento e vinte e três");
    assert_eq!(words("15"), "quinze");
    assert_eq!(words("21"), "vinte e um");
    assert_eq!(words("100"), "cem");
    assert_eq!(words("1000"), "mil");
    assert_eq!(words("1.923,5"), "mil novecentos e vinte e três vírgula cinco");
    assert_eq!(words("0"), "zero");
}

#[test]
fn phrases_are_broken_by_the_punctuation() {
    assert_eq!(pt("Bom dia! Como você está?"), "b ˈɔ̃ m dʒ ˈi ɐ | k ˈo m u v o s ˈe ɛ s t ˈa");
}

#[test]
fn every_sound_is_in_the_inventory() {
    let body = BodyMix::MALE;
    for seg in to_phrases("Que嘻就能 não é possível: a mãe, o pão, três, você, chi!?").iter().flat_map(|p| p.segments.iter()) {
        // a diphthong is one segment with two symbols; each symbol must be known
        for symbol in seg.ipa.chars() {
            if symbol == '\u{0303}' {
                continue;
            }
            assert!(
                is_vowel(&symbol.to_string(), body) || phoneme(&symbol.to_string(), body).is_some(),
                "unknown sound {symbol} in {}",
                seg.ipa
            );
        }
    }
}

#[test]
fn portuguese_plans_as_speech() {
    let body = BodyMix::MALE;
    let phrases = to_phrases("Bom dia! Como você está?");
    let inputs: Vec<PhraseInput> = phrases
        .iter()
        .map(|p| PhraseInput { segments: p.segments.clone(), intonation: Some(p.intonation), pause: Some(p.pause) })
        .collect();
    let plan = plan_phrases(&inputs, &PlanOptions { body, ..Default::default() });
    assert!(!plan.gestures.is_empty());
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
fn lyrics_split_where_portuguese_spelling_does() {
    assert_eq!(sung("a casa e a rua"), ["a", "ca", "sa", "e", "a", "ru", "a"]);
    let syl = lyrics_to_syllables("admissão bom dia");
    let p: Vec<String> = syl.iter().map(parts).collect();
    // the cluster splits: the last consonant opens the next syllable
    assert_eq!(p[0], ".ɐ.d");
    assert_eq!(p[1], "m.i.s");
    assert_eq!(p[2], "s.ɐ̃w.");
    assert_eq!(syl.iter().filter(|s| s.word_end).count(), 3);
}

#[test]
fn the_lyrics_keep_the_spelling_of_each_syllable() {
    // "Português" is por-tu-guês: the "gu" before the accented e is one onset
    assert_eq!(sung("Português"), ["por", "tu", "guê"]);
}

#[test]
fn latin_script_is_portuguese_only_when_chosen() {
    let no_dict = |_: &str, _: bool| Vec::<Segment>::new();
    let text = |lang| {
        lyrics_in("Português coração", &no_dict, &MandarinLexicon::new(), lang)
            .into_iter()
            .map(|s| s.text)
            .collect::<Vec<_>>()
    };
    // Portuguese claims the Latin script when it is chosen
    assert_eq!(text(Lang::Pt), ["por", "tu", "guê", "co", "ra", "ção"]);
    // English claims it otherwise (and the no-dictionary source gives no English at all)
    assert!(text(Lang::En).is_empty());
}

#[test]
fn the_other_scripts_still_go_to_their_own_front_ends() {
    let no_dict = |_: &str, _: bool| Vec::<Segment>::new();
    let text = lyrics_in("Português 빵 дом", &no_dict, &MandarinLexicon::new(), Lang::Pt)
        .into_iter()
        .map(|s| s.text)
        .collect::<Vec<_>>();
    assert_eq!(text, ["por", "tu", "guê", "빵", "дом"]);
}

#[test]
fn portuguese_phrases_carry_the_intonation_of_the_punctuation() {
    let phrases = to_phrases("Olá! Tudo bem?");
    assert_eq!(phrases.len(), 2);
    assert_eq!(phrases[0].intonation, Intonation::Statement);
    assert_eq!(phrases[1].intonation, Intonation::Statement);
}
