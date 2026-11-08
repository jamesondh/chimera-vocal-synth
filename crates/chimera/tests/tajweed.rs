//! Recitation: the tajweed the reader annotates is what the voice does. Readings and lengths in
//! counts (Ḥafṣ by way of al-Shāṭibiyya), and the articulation measured in a render: the soft
//! palate open through a ghunna, the lips shut through an iqlāb.

use chimera::phonetics::arabic::{ArabicOptions, ArabicPhrase, to_phrases};
use chimera::phonetics::planner::{PhraseInput, PlanOptions, plan_phrases};
use chimera::voice::human_voice::HumanVoice;

fn recite(text: &str) -> Vec<ArabicPhrase> {
    to_phrases(text, &ArabicOptions { recite: true, ..Default::default() })
}

fn read(text: &str) -> String {
    recite(text).iter().map(|p| p.ipa.clone()).collect::<Vec<_>>().join(" | ")
}

#[test]
fn madd_lengths_follow_hafs() {
    let count = |text: &str, ipa: &str, k: usize| -> f64 {
        let p = &recite(text)[0];
        let i = p.segments.iter().enumerate().filter(|(_, s)| s.ipa == ipa).map(|(i, _)| i).nth(k).unwrap();
        p.counts[i]
    };
    // muttaṣil 4, munfaṣil 4, lāzim 6, ʿāriḍ 4 (the default), natural 2
    assert_eq!(count("جَآءَ", "a", 0), 4.0);
    assert_eq!(count("بِمَآ أُنزِلَ", "a", 0), 4.0);
    assert_eq!(count("وَلَا ٱلضَّآلِّينَ", "aˤ", 0), 6.0);
    assert_eq!(count("ٱلرَّحِيمِ", "i", 0), 4.0);
    assert_eq!(count("ٱلرَّحْمَٰنِ ٱلرَّحِيمِ", "a", 1), 2.0); // not before a pause
    assert_eq!(count("ٱلرَّحْمَٰنِ", "a", 1), 4.0); // before a pause: ʿāriḍ
    let p = &to_phrases("ٱلرَّحِيمِ", &ArabicOptions { recite: true, arid: 6.0, ..Default::default() })[0];
    assert!(p.counts.contains(&6.0));
}

#[test]
fn nun_sakin_and_tanwin() {
    assert_eq!(read("هُدًى مِّن رَّبِّهِمْ"), "huda mːi rˤːaˤbːihim"); // idghām with and without ghunna
    assert_eq!(read("مَن يَقُولُ"), "ma jːaquˤːːl"); // idghām into yāʾ, nasalized
    assert_eq!(read("أَلِيمٌۢ بِمَا"), "ʔaliːmumː bimaː"); // iqlāb
    assert_eq!(read("أُنزِلَ"), "ʔunːzil"); // ikhfāʾ, held
    assert_eq!(read("مِن قَبْلِكَ"), "miɴː qaˤblik"); // ikhfāʾ toward the uvula; qalqala on the bāʾ
    assert_eq!(read("وَمَا هُم بِمُؤْمِنِينَ"), "wamaː humː bimuʔminiːːn"); // ikhfāʾ shafawī
    assert_eq!(read("فِى قُلُوبِهِم مَّرَضٌ"), "fiː quˤluːbihi mːaɾˤaˤdˤ"); // idghām shafawī
    let p = &recite("مَن يَقُولُ")[0];
    let j = p.segments.iter().position(|s| s.ipa == "j").unwrap();
    assert_eq!(p.segments[j].nasalized, Some(true));
}

#[test]
fn like_and_near_letters_merge() {
    assert_eq!(read("قَد تَّبَيَّنَ"), "qaˤ tːabajːan");
    assert_eq!(read("بَل رَّفَعَهُ"), "ba rˤːaˤfaʕah");
}

#[test]
fn no_word_accents_and_the_aya_falls() {
    assert_eq!(read("قُلْ هُوَ ٱللَّهُ أَحَدٌ"), "quˤl huwa ɫːaˤːhu ʔaħad");
    let p = &recite("قُلْ هُوَ ٱللَّهُ أَحَدٌ")[0];
    assert!(p.segments.iter().all(|s| s.stress == Some(0) || s.stress.is_none()));
    assert_eq!(p.intonation, chimera::phonetics::Intonation::Statement);
}

const FS: f64 = 48000.0;

/// Through each segment called `seg`: the most the velum port opens, the narrowest lip opening,
/// and the share of it voiced.
fn through(text: &str, seg: &str) -> (f64, f64, f64) {
    let ps = recite(text);
    let inputs: Vec<PhraseInput> = ps.iter().map(|p| PhraseInput { segments: p.segments.clone(), intonation: Some(p.intonation), pause: Some(p.pause) }).collect();
    let plan = plan_phrases(&inputs, &PlanOptions { body: "male".into(), t0: 0.05, f0: Some(110.0), ..Default::default() });
    let s = plan.segments.iter().find(|s| s.ipa == seg).unwrap_or_else(|| panic!("{text}: no {seg}"));
    let mut v = HumanVoice::with_defaults(FS);
    v.schedule(&plan.gestures);
    let (mut out, mut i) = (vec![0.0f32; 48], 0usize);
    let (mut port, mut lips, mut voiced, mut n) = (0.0f64, f64::INFINITY, 0, 0);
    while (i as f64) / FS < s.t1 {
        v.render(&mut out);
        i += 48;
        let t = i as f64 / FS;
        if t >= s.t0 + 0.1 * (s.t1 - s.t0) {
            let f = v.airway.frame.as_ref().unwrap();
            port = port.max(f.velum_port);
            lips = lips.min(v.airway.or_min[v.airway.or_min.len() - 1]);
            voiced += (v.glottis.voicing > 0.5) as usize;
            n += 1;
        }
    }
    (port, lips, voiced as f64 / n.max(1) as f64)
}

#[test]
fn the_voice_does_what_the_rules_say() {
    // ghunna on an idghām into yāʾ: the soft palate opens over the doubled glide
    let (port, _, _) = through("مَن يَقُولُ", "j");
    assert!(port > 0.2, "velum port {port:.2} cm² over the nasalized yāʾ");
    // iqlāb: the n becomes an m, the lips shut and the nose open
    let (port, lips, _) = through("أَلِيمٌۢ بِمَا", "m");
    assert!(port > 0.2 && lips < 0.05, "iqlāb's m: velum {port:.2} cm², lips {lips:.3} cm²");
}
