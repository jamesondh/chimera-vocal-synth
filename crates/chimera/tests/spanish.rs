//! The Spanish G2P, one test per rule: digraphs and silent h, soft/hard c and g, v→b, the r
//! tap/trill rules, diphthongs and hiatus, the written-accent stress rules, function words,
//! allophones (β ð ɣ, s voicing, n velarisation), the dialect switches, Spanish syllabification
//! and the singing path. `say` runs the word through the real phrase path; the stress mark ˈ
//! sits directly on the stressed segment (the syllable's vowel), as `Segment.stress` carries it.

use chimera::phonetics::spanish::{CASTELLANO, Dialect, Ll, NEUTRO, RIOPLATENSE, SpanishG2P};
use chimera::phonetics::syllables::{Lang, lyrics_to_syllables, split_spelling, split_spelling_lang, syllabify_segments_lang};
use chimera::phonetics::Segment;

/// one word through the real phrase path (word-level allophones included), IPA with stress marks
fn say(g: &SpanishG2P, w: &str) -> String {
    g.to_phrases(w)[0].segments.iter().map(|s| {
        let mut out = String::new();
        match s.stress {
            Some(1) => out.push('ˈ'),
            Some(2) => out.push('ˌ'),
            _ => {}
        }
        out.push_str(&s.ipa);
        out
    }).collect()
}

fn g2p() -> SpanishG2P {
    SpanishG2P::new()
}

/// the words of a phrase and their IPA, comma-joined for readability
fn phrase(g: &SpanishG2P, text: &str) -> String {
    g.to_phrases(text).iter().map(|p| {
        let ipa = p.segments.iter().map(|s| s.ipa.as_str()).collect::<Vec<_>>().join("");
        format!("{}({})", p.words.join(" "), ipa)
    }).collect::<Vec<_>>().join(" || ")
}

#[test]
fn digraphs_and_silent_h() {
    let g = g2p();
    assert_eq!(say(&g, "chico"), "tʃˈiko");
    assert_eq!(say(&g, "llave"), "ʝˈaβe");
    assert_eq!(say(&g, "perro"), "pˈero");
    assert_eq!(say(&g, "guerra"), "gˈera");
    assert_eq!(say(&g, "guitarra"), "gitˈara");
    assert_eq!(say(&g, "hueso"), "wˈeso");
    assert_eq!(say(&g, "hola"), "ˈola");
    assert_eq!(say(&g, "pingüino"), "piŋgwˈino");
    assert_eq!(say(&g, "vergüenza"), "beɾɣwˈensa");
}

#[test]
fn soft_and_hard_c_and_g() {
    let g = g2p();
    // soft c → [s] (seseo); hard c → [k]; "cc" = [ks]
    assert_eq!(say(&g, "cine"), "sˈine");
    assert_eq!(say(&g, "casa"), "kˈasa");
    assert_eq!(say(&g, "cocción"), "koksjˈon");
    assert_eq!(say(&g, "química"), "kˈimika");
    assert_eq!(say(&g, "queso"), "kˈeso");
    assert_eq!(say(&g, "quota"), "kwˈota");
    // soft g → [x]; hard g → [g]
    assert_eq!(say(&g, "gente"), "xˈente");
    assert_eq!(say(&g, "jamón"), "xamˈon");
    assert_eq!(say(&g, "mango"), "mˈaŋgo");
    assert_eq!(say(&g, "agenda"), "axˈenda");
    // x = [ks]
    assert_eq!(say(&g, "examen"), "eksˈamen");
}

#[test]
fn v_is_b_and_the_stops_fricativise() {
    let g = g2p();
    assert_eq!(say(&g, "vaso"), "bˈaso");
    // intervocalic (and after a continuant) b d g are fricatives
    assert_eq!(say(&g, "lobo"), "lˈoβo");
    assert_eq!(say(&g, "dedo"), "dˈeðo");
    assert_eq!(say(&g, "verde"), "bˈeɾðe");
    assert_eq!(say(&g, "algo"), "ˈalɣo");
    // after a pause or a nasal they are stops
    assert_eq!(say(&g, "tengo"), "tˈeŋgo");
    assert_eq!(say(&g, "guapo"), "gwˈapo");
    assert_eq!(say(&g, "agua"), "ˈaɣwa");
}

#[test]
fn tap_and_trill_rules() {
    let g = g2p();
    // rr = trill; single r between vowels = tap
    assert_eq!(say(&g, "carro"), "kˈaro");
    assert_eq!(say(&g, "caro"), "kˈaɾo");
    // trill at word start and after n, l, s
    assert_eq!(say(&g, "rey"), "rˈei");
    assert_eq!(say(&g, "enredo"), "enrˈeðo");
    assert_eq!(say(&g, "alrededor"), "alreðeðˈoɾ");
    assert_eq!(say(&g, "Israel"), "izraˈel");
}

#[test]
fn diphthongs_and_hiatus() {
    let g = g2p();
    // falling diphthongs are one token (the planner glides them)
    assert_eq!(say(&g, "aire"), "ˈaiɾe");
    assert_eq!(say(&g, "seis"), "sˈeis");
    assert_eq!(say(&g, "auto"), "ˈauto");
    assert_eq!(say(&g, "soy"), "sˈoi");
    // rising diphthongs are a glide + nucleus
    assert_eq!(say(&g, "tierra"), "tjˈera");
    assert_eq!(say(&g, "piano"), "pjˈano");
    assert_eq!(say(&g, "cuento"), "kwˈento");
    assert_eq!(say(&g, "ciudad"), "sjuðˈað");
    assert_eq!(say(&g, "cuidado"), "kwiðˈaðo");
    // accented weak vowels and strong+strong break the hiatus
    assert_eq!(say(&g, "país"), "paˈis");
    assert_eq!(say(&g, "púa"), "pˈua");
    assert_eq!(say(&g, "día"), "dˈia");
    assert_eq!(say(&g, "leer"), "leˈeɾ");
    assert_eq!(say(&g, "cae"), "kˈae");
    // the y of "muy" is the offglide [j], not a glide before a nucleus
    assert_eq!(say(&g, "muy"), "mˈui");
}

#[test]
fn written_accent_stress_rules() {
    let g = g2p();
    // the accent wins (agudas, llanas, esdrújulas alike)
    assert_eq!(say(&g, "camión"), "kamjˈon");
    assert_eq!(say(&g, "árbol"), "ˈaɾβol");
    assert_eq!(say(&g, "rápido"), "rˈapiðo");
    assert_eq!(say(&g, "azúcar"), "asˈukaɾ");
    assert_eq!(say(&g, "cómpramelo"), "kˈompɾamelo");
    // no accent: ends in vowel/n/s → penult, else last
    assert_eq!(say(&g, "casa"), "kˈasa");
    assert_eq!(say(&g, "cantan"), "kˈantan");
    assert_eq!(say(&g, "amores"), "amˈoɾes");
    assert_eq!(say(&g, "papel"), "papˈel");
    assert_eq!(say(&g, "raíz"), "raˈis");
    assert_eq!(say(&g, "reloj"), "relˈox");
    // monosyllabic content words are stressed, function words are not
    assert_eq!(say(&g, "sol"), "sˈol");
    assert_eq!(say(&g, "el"), "el");
    assert_eq!(say(&g, "de"), "de");
    assert_eq!(say(&g, "que"), "ke");
    assert_eq!(say(&g, "y"), "i");
    // accented function words stay stressed
    assert_eq!(say(&g, "sí"), "sˈi");
    assert_eq!(say(&g, "mí"), "mˈi");
    assert_eq!(say(&g, "qué"), "kˈe");
}

#[test]
fn allophones_in_phrases() {
    let g = g2p();
    // /s/ voices before a voiced consonant (and then fricativises the next stop too)
    assert!(phrase(&g, "mismo").contains("(mizmo)"));
    assert!(phrase(&g, "desde").contains("(dezðe)"));
    // /n/ velarises before k, g, x
    assert!(phrase(&g, "banco").contains("(baŋko)"));
    // cross-word b/d/g after a vowel are fricatives, after a nasal they are stops
    assert!(phrase(&g, "la boca").contains("(laβoka)"));
    assert!(phrase(&g, "en boca").contains("(enboka)"));
}

#[test]
fn phrases_and_intonation() {
    let g = g2p();
    let ps = g.to_phrases("Hola. ¿Cómo estás?");
    assert_eq!(ps.len(), 2);
    assert_eq!(ps[0].intonation, chimera::phonetics::Intonation::Statement);
    assert_eq!(ps[1].intonation, chimera::phonetics::Intonation::Question);
    assert_eq!(ps[0].words, vec!["hola".to_string()]);
    assert_eq!(ps[1].words, vec!["cómo".to_string(), "estás".to_string()]);
    // accented letters hold words together ("cami-ón" would be two words)
    assert_eq!(g.to_phrases("Camión").len(), 1);
    // digits say nothing yet (number words are a later step): "hola" survives, "3" is dropped
    assert_eq!(g.to_phrases("Hola 3")[0].words, vec!["hola".to_string()]);
}

#[test]
fn dialect_switches() {
    let g2p = |d: Dialect| {
        let mut g = SpanishG2P::new();
        g.set_dialect(d);
        g
    };
    // distinción: z and soft c say [θ]
    assert_eq!(say(&g2p(CASTELLANO), "caza"), "kˈaθa");
    assert_eq!(say(&g2p(CASTELLANO), "zapato"), "θapˈato");
    // sheísmo: ll and consonantal y say [ʃ]
    assert_eq!(say(&g2p(RIOPLATENSE), "lluvia"), "ʃˈuβja");
    assert_eq!(say(&g2p(RIOPLATENSE), "mayo"), "mˈaʃo");
    // zheísmo and lleísta variants
    assert_eq!(say(&g2p(Dialect { seseo: true, ll: Ll::Zheismo, aspirated_s: false }), "llave"), "ʒˈaβe");
    assert_eq!(say(&g2p(Dialect { seseo: true, ll: Ll::Lleista, aspirated_s: false }), "llave"), "ʎˈaβe");
    // aspirated coda s: before a consonant and word-finally, but not before a vowel
    assert_eq!(say(&g2p(RIOPLATENSE), "mismo"), "mˈihmo");
    assert_eq!(say(&g2p(RIOPLATENSE), "más"), "mˈah");
    assert_eq!(say(&g2p(RIOPLATENSE), "casa"), "kˈasa");
    // the default is neutro: seseo, jeísmo, s kept
    assert_eq!(say(&g2p(NEUTRO), "llave"), "ʝˈaβe");
    assert_eq!(say(&g2p(NEUTRO), "caza"), "kˈasa");
    // the dialect round-trips through its JSON (what the wasm call passes)
    let d: Dialect = serde_json::from_str(r#"{"seseo": false, "ll": "lleista", "aspiratedS": true}"#).unwrap();
    assert_eq!(d, Dialect { seseo: false, ll: Ll::Lleista, aspirated_s: true });
}

#[test]
fn spanish_syllables_for_singing() {
    let g = g2p();
    // consonant–glide onsets and digraph codas split the Spanish way
    let parts = syllabify_segments_lang(&g.word("tierra", false), Lang::Spanish);
    let nuclei: Vec<String> = parts.iter().map(|p| p.nucleus.ipa.clone()).collect();
    assert_eq!(nuclei, vec!["e".to_string(), "a".to_string()]); // tje-rra: two nuclei
    let onsets: Vec<String> = parts.iter().map(|p| p.onset.iter().map(|s| s.ipa.as_str()).collect()).collect();
    assert_eq!(onsets, vec!["tj".to_string(), "r".to_string()]);
    // the hiatus keeps two nuclei ("pa-ís")
    assert_eq!(syllabify_segments_lang(&g.word("país", false), Lang::Spanish).len(), 2);
    // spelling splits keep the digraph right ("gue-rra") and "qu" whole ("que-so")
    assert_eq!(split_spelling_lang("guerra", 2, Lang::Spanish), vec!["gue", "rra"]);
    assert_eq!(split_spelling_lang("queso", 2, Lang::Spanish), vec!["que", "so"]);
    assert_eq!(split_spelling_lang("camión", 2, Lang::Spanish), vec!["ca", "mión"]);
    assert_eq!(split_spelling_lang("tierra", 2, Lang::Spanish), vec!["tie", "rra"]);
    // a hiatus breaks the written run too ([pa-ís], [le-er], [ca-e]); the diphthong stays whole
    assert_eq!(split_spelling_lang("país", 2, Lang::Spanish), vec!["pa", "ís"]);
    assert_eq!(split_spelling_lang("leer", 2, Lang::Spanish), vec!["le", "er"]);
    assert_eq!(split_spelling_lang("cae", 2, Lang::Spanish), vec!["ca", "e"]);
    assert_eq!(split_spelling_lang("ruido", 2, Lang::Spanish), vec!["rui", "do"]);
    // English spelling is untouched (its final e is silent; the Spanish final e never is)
    assert_eq!(split_spelling("little", 2), vec!["lit", "tle"]);
    assert_eq!(split_spelling_lang("café", 2, Lang::Spanish), vec!["ca", "fé"]);
}

#[test]
fn spanish_lyrics_to_syllables() {
    let g = g2p();
    // one syllable per note; the display text keeps its spelling (and its case)
    let syls = lyrics_to_syllables("La luna llena", &g);
    let texts: Vec<&str> = syls.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(texts, vec!["La", "lu", "na", "lle", "na"]);
    // the hiatus "país" gives two notes
    let syls = lyrics_to_syllables("país", &g);
    assert_eq!(syls.len(), 2);
    assert_eq!(syls[0].text, "pa");
    assert_eq!(syls[1].text, "ís");
    // accented letters are word characters in lyrics
    assert!(lyrics_to_syllables("camión", &g).len() >= 2);
    // a phrase break still ends the word before it
    let syls = lyrics_to_syllables("corazón, corazón", &g);
    assert!(syls[2].phrase_end);
}

#[test]
fn spanish_stops_are_not_aspirated() {
    use chimera::control::score::ControlKey;
    use chimera::phonetics::planner::{PlanOptions, plan_utterance};

    // a syllable-initial /p/ before a vowel: the English planner aspirates it, the Spanish one
    // never does (a shorter VOT and a narrower glottal opening)
    let input = vec![Segment::new("p"), Segment::new("a")];
    let aspiration = |lang: Option<Lang>| {
        let plan = plan_utterance(&input, &PlanOptions { body: "male".into(), t0: 0.05, lang, ..Default::default() });
        plan.gestures.iter().find(|g| g.label.as_ref().is_some_and(|label| label.as_str() == "aspiration")).map(|g| g.targets.get(ControlKey::ABDUCTION).unwrap_or(0.0))
    };
    assert_eq!(aspiration(None), Some(0.36)); // English is the default
    assert_eq!(aspiration(Some(Lang::English)), Some(0.36));
    assert_eq!(aspiration(Some(Lang::Spanish)), Some(0.22));
}
