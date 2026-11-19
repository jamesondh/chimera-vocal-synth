use chimera::anatomy::human::{ArticKey, BodyMix};
use chimera::phonetics::sing::{SongOptions, SungNote, plan_song, plan_song_with_vowel_tuning};
use chimera::phonetics::{Plan, Segment, Syllable};

fn melody(body: BodyMix, vowel: &str, pitches: &[f64], tune: bool) -> Plan {
    let notes = pitches.iter().enumerate().map(|(i, &midi)| SungNote {
        midi,
        beats: 1.0,
        syl: Syllable { nucleus: Segment::new(vowel), melisma: Some(i > 0), ..Default::default() },
    }).collect::<Vec<_>>();
    let options = SongOptions { body, bpm: 90.0, t0: 0.3, vibrato: 0.0 };
    if tune { plan_song_with_vowel_tuning(&notes, &options, true) } else { plan_song(&notes, &options) }
}

#[test]
fn high_i_opens_on_a_melisma_and_returns_to_the_low_vowel() {
    for body in [BodyMix::FEMALE, BodyMix::MALE, BodyMix::CHILD, BodyMix::new(0.5, 0.2)] {
        let pitches = [55.0, 74.0, 55.0];
        let original = melody(body, "i", &pitches, false);
        let tuned = melody(body, "i", &pitches, true);
        assert_eq!(tuned.segments, original.segments);
        let vowels = |plan: &Plan| plan.gestures.iter()
            .filter(|g| g.label.as_ref().is_some_and(|l| l.as_str() == "i"))
            .map(|g| g.targets.clone()).collect::<Vec<_>>();
        let old = vowels(&original);
        let new = vowels(&tuned);
        assert_eq!(new.len(), 3);
        assert_eq!(new[0].iter().collect::<Vec<_>>(), old[0].iter().collect::<Vec<_>>());
        assert_eq!(new[2].iter().collect::<Vec<_>>(), old[2].iter().collect::<Vec<_>>());
        assert!(new[1].get(ArticKey::Jaw).unwrap() > old[1].get(ArticKey::Jaw).unwrap());
        assert!(new[1].get(ArticKey::TbY).unwrap() < old[1].get(ArticKey::TbY).unwrap());
        assert_eq!(new[1].get(ArticKey::TbX), old[1].get(ArticKey::TbX));
        assert_eq!(new[1].get(ArticKey::LipAp), old[1].get(ArticKey::LipAp));
    }
}

#[test]
fn switching_on_high_i_preserves_the_other_vowels() {
    let pitches = [69.0, 76.0];
    for vowel in ["a", "u", "ɪ", "ia", "eɪ"] {
        // Targets use NaN for an unset channel. Compare the public plans, which omit these channels.
        let plan = |tune| serde_json::to_value(melody(BodyMix::FEMALE, vowel, &pitches, tune)).unwrap();
        assert_eq!(plan(false), plan(true), "{vowel}");
    }
}
