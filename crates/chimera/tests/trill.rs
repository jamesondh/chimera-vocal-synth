//! The alveolar trill /r/ (2026-11-16, Spanish "perro" heard as something strange): its strikes,
//! each a stop-like gesture with 30 ms and 18 ms ramps, overlapped into one held near-closure, so
//! the voice went quiet for 85–90 of the trill's ~100 ms.
//! A trill is voiced throughout: the tract opens between strikes.
use chimera::anatomy::human::{BODY_FEMALE, BODY_MALE, HumanBody};
use chimera::phonetics::Segment;
use chimera::phonetics::planner::{PlanOptions, plan_utterance};
use chimera::phonetics::spanish::SpanishG2P;
use chimera::voice::human_voice::{HumanVoice, SourcePatch};

const FS: f64 = 48000.0;

fn seg(ipa: &str, dur: Option<f64>, stress: u8) -> Segment {
    Segment { dur, stress: Some(stress), ..Segment::new(ipa) }
}

fn rms(x: &[f32], a: f64, b: f64) -> f64 {
    let (i, j) = ((a * FS) as usize, (b * FS) as usize);
    (x[i..j].iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / (j - i) as f64).sqrt()
}

#[test]
fn a_trill_stays_voiced_and_releases_without_a_burst() {
    let voices: [(HumanBody, &str, f64); 2] = [(BODY_MALE, "male", 110.0), (BODY_FEMALE, "female", 200.0)];
    for (body, name, f0) in voices {
        let segs = [seg("p", None, 0), seg("e", Some(0.105), 1), seg("r", None, 0), seg("o", Some(0.11), 0)];
        let plan = plan_utterance(&segs, &PlanOptions { body: name.into(), t0: 0.05, f0: Some(f0), ..Default::default() });
        let mut v = HumanVoice::new(FS, body, 2);
        v.set_source(&SourcePatch::f0(f0));
        v.schedule(&plan.gestures);
        let mut x = vec![0.0f32; (FS * (plan.end + 0.3)) as usize];
        for chunk in x.chunks_mut(128) {
            v.render(chunk);
        }
        let r = plan.segments.iter().find(|s| s.ipa == "r").unwrap();
        let e = plan.segments.iter().find(|s| s.ipa == "e").unwrap();
        let o = plan.segments.iter().find(|s| s.ipa == "o").unwrap();
        let vowel = rms(&x, e.t0 + 0.04, e.t1 - 0.02).max(rms(&x, o.t0 + 0.02, o.t0 + 0.07));
        // the longest run of quiet 5 ms frames (20 dB below the vowel) inside the trill, and the
        // quietest frame: the strikes still dip the sound (it is not a vowel)
        let (mut run, mut longest, mut quietest, mut t) = (0, 0, f64::MAX, r.t0);
        while t + 0.005 <= r.t1 {
            let level = rms(&x, t, t + 0.005) / vowel;
            run = if level < 0.1 { run + 1 } else { 0 };
            longest = longest.max(run);
            quietest = quietest.min(level);
            t += 0.005;
        }
        eprintln!("{name}: longest quiet run {} ms, quietest frame {:.1} dB", longest * 5, 20.0 * quietest.log10());
        assert!(longest * 5 <= 15, "{name}: the trill went quiet for {} ms", longest * 5);
        assert!(quietest < 0.5, "{name}: no strike dips the sound");
    }
}

#[test]
fn spanish_rr_gets_an_extra_cycle_without_lengthening_word_initial_r() {
    let g = SpanishG2P::new();
    let options = PlanOptions { body: "male".into(), t0: 0.05, f0: Some(112.0), ..Default::default() };
    let cycles = |word: &str| {
        let plan = plan_utterance(&g.word(word, false), &options);
        plan.gestures.iter().filter(|gesture| gesture.label_str() == "r").count()
    };
    // At 20 Hz each cycle contributes a closure and, except after the last one, an opening.
    // Orthographic rr gets three strikes; initial single r retains the two-cycle timing.
    assert_eq!(cycles("guerra"), 5);
    assert_eq!(cycles("régimen"), 3);
}

#[test]
fn spanish_guerra_renders_repeated_contacts() {
    let g = SpanishG2P::new();
    let plan = plan_utterance(&g.word("guerra", false), &PlanOptions { body: "male".into(), t0: 0.05, f0: Some(112.0), ..Default::default() });
    let mut voice = HumanVoice::new(FS, BODY_MALE, 2);
    voice.set_source(&SourcePatch::f0(112.0));
    voice.schedule(&plan.gestures);
    let mut audio = vec![0.0f32; (FS * (plan.end + 0.25)) as usize];
    for block in audio.chunks_mut(128) { voice.render(block); }
    let trill = plan.segments.iter().find(|s| s.ipa == "r").unwrap();
    let reference = plan.segments.iter().filter(|s| matches!(s.ipa.as_str(), "e" | "a"))
        .map(|s| rms(&audio, s.t0 + 0.02, s.t1 - 0.01)).fold(0.0, f64::max);
    let (mut t, mut inside, mut dips) = (trill.t0, false, 0);
    while t + 0.005 <= trill.t1 {
        let low = rms(&audio, t, t + 0.005) / reference < 0.5;
        if low && !inside { dips += 1; }
        inside = low;
        t += 0.005;
    }
    eprintln!("Spanish guerra: {dips} audible contact dips in the trill");
    assert!(dips >= 3, "only {dips} contact dips in the trill");
}
