//! The syllable a note takes from the lyrics, and for a melisma the syllable it continues.
use chimera::control::singer::{Singer, syllables_from};
use chimera::phonetics::english::EnglishG2P;
use chimera::phonetics::sing::LiveSyllable;
use chimera::phonetics::syllables::lyrics_to_syllables;

/// The singer remembers a melisma's parent while its cursor moves in order, so that a run of
/// melismas is looked through once and not once a note. What it remembers must be what a search
/// from the melisma back would find, wherever the cursor has been.
#[test]
fn a_melisma_continues_the_syllable_before_its_run_wherever_the_cursor_went() {
    let g2p = EnglishG2P::default();
    // a melisma first (nothing to continue: it sings "ah"), runs of them, and a wrap past the end
    let mut lyrics = syllables_from(&lyrics_to_syllables("one two three four five six one two three four five six", &g2p));
    assert_eq!(lyrics.len(), 12);
    for (syllable, melisma) in lyrics.iter_mut().zip([true, true, false, true, true, false, false, true, false, true, true, true]) {
        syllable.melisma = melisma;
    }
    let search = |i: usize| lyrics[..=i].iter().rev().find(|s| !s.melisma).copied().unwrap_or_else(LiveSyllable::ah);
    let mut s = Singer::new();
    s.set_syllables(lyrics.clone());
    let mut taken = 0;
    // in order, three times round, then from each syllable in turn, forwards and backwards
    let jumps: Vec<Option<usize>> = std::iter::repeat_n(None, 40).chain((0..12).flat_map(|i| [Some(i), None, None, Some(11 - i), None])).collect();
    for jump in jumps {
        if let Some(i) = jump {
            s.set_cursor(i);
        }
        let at = s.cursor();
        let lyric = s.next_lyric();
        assert_eq!(lyric.index, at);
        assert_eq!(lyric.syllable, lyrics[at]);
        assert_eq!(lyric.parent, search(at), "the parent of syllable {at}, after {taken} notes");
        assert_eq!(s.cursor(), (at + 1) % lyrics.len());
        taken += 1;
    }
    // new lyrics: nothing of the old ones is remembered
    let mut shorter = syllables_from(&lyrics_to_syllables("la la", &g2p));
    shorter[0].melisma = true;
    s.set_cursor(4);
    s.next_lyric();
    s.set_syllables(shorter.clone());
    assert_eq!(s.next_lyric().parent, LiveSyllable::ah());
    assert_eq!(s.next_lyric().parent, shorter[1]);
}
