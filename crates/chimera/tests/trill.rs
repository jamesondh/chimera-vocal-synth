//! A trill strikes and opens about 26 times a second; it does not seal into a stop.
//!
//! The bug this keeps fixed: each strike was a full oral gesture whose ramps lasted ~70 ms, so at
//! 38 ms spacing they overlapped and the tip stayed shut for the whole trill (110 ms of silence in
//! /arːa/), heard as a /d/ or, pharyngealized, a /tˤ/.

use chimera::phonetics::ipa::tokenize_ipa;
use chimera::phonetics::planner::{PlanOptions, plan_utterance};
use chimera::voice::human_voice::HumanVoice;

const FS: f64 = 48000.0;

/// The narrowest area of the tract every 2 ms through the first segment called `seg`.
fn min_area_through(ipa: &str, seg: &str) -> Vec<f64> {
    let mut v = HumanVoice::with_defaults(FS);
    let plan = plan_utterance(&tokenize_ipa(ipa), &PlanOptions { body: "male".into(), t0: 0.05, f0: Some(110.0), ..Default::default() });
    v.schedule(&plan.gestures);
    let s = plan.segments.iter().find(|s| s.ipa == seg).unwrap();
    let mut out = vec![0.0f32; 96];
    let (mut i, mut got) = (0usize, vec![]);
    while (i as f64) / FS < s.t1 {
        v.render(&mut out);
        i += 96;
        let t = i as f64 / FS;
        if t >= s.t0 {
            got.push(v.airway.ph_min.iter().chain(v.airway.or_min.iter()).cloned().fold(f64::INFINITY, f64::min));
        }
    }
    got
}

/// Times the area crosses from below `lo` to above `hi` (openings after a strike).
fn openings(a: &[f64], lo: f64, hi: f64) -> usize {
    let (mut n, mut struck) = (0, false);
    for &x in a {
        if x < lo {
            struck = true;
        } else if x > hi && struck {
            n += 1;
            struck = false;
        }
    }
    n
}

#[test]
fn a_trill_opens_between_its_strikes() {
    for (ipa, seg) in [("ˈarːa", "r"), ("ˈarˤːaˤ", "rˤ"), ("ˈaʀːa", "ʀ"), ("ˈaʙːa", "ʙ")] {
        let a = min_area_through(ipa, seg);
        let n = openings(&a, 0.03, 0.1);
        assert!(n >= 2, "{ipa}: the passage opened {n} times after a strike, want ≥ 2 ({:?})", a.iter().map(|x| (x * 1000.0).round() / 1000.0).collect::<Vec<_>>());
    }
}
