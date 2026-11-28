# Finnish

Speak and Sing read Finnish once you choose it: **FI**, next to "Text is IPA" in Speak and next
to the lyrics in Sing. It is one setting for both tabs, kept on the device with Sing's words.
Finnish is written in Latin letters, so the lyrics router can't tell it from English by script
([language-front-ends.md](language-front-ends.md)); English stays the default. Toned pinyin,
Hangul, Cyrillic and Han characters in the same lyrics still go to their own front ends.

`crates/chimera/src/phonetics/finnish.rs` reads Finnish orthography by rule, with no dictionary.

## Writing it

- Type it as you'd write it: `a e i o u y ä ö å` (plus loan consonants `š ž`).
- In Sing, words split into Finnish syllables (`tui-ki täh-tö-nen`, `täh-ti-en`, `kier-ros`,
  `yöl-lä`). `-` alone holds the previous vowel onto the next note, and hyphens split compounds
  your way (`linja-auto`).
- Numbers are read out in Speak (`14` → `neljätoista`, `3,5` → `kolme pilkku viisi`).

## What it does

| | |
|---|---|
| Vowels | Calibrated open front `æ̞` (*ä*), mid front rounded `œ` (*ö*), close back `u̝` (*u*), and open back `ɑ̠` (*a*) from Kuronen (2000), Wiik (1965) / Iivonen & Laukkanen (1993), and Google FLEURS `fi_fi`, leaving English `æ u ɑ` untouched |
| Diphthongs | All 18 Finnish diphthongs stay on one syllable: first-syllable opening `ie uo yö` spend 60 % of a sung note on their open second half (`i→ɛ`, `u̝→ɔ`, `y→œ`); non-initial `ie uo yö` (`tähtien`) split across syllables (`täh-ti-en`) |
| Stops `/p t k/` | Dental `t̪` (`DENT_STOP`), full glottal opening during closure with a tight 12 ms VOT so stops release crisply without `[d]` voicing bleed or English `[tʰ]` breathiness |
| Lateral `l` | Clear anchored lateral `l̪` with smooth secondary tongue-body stabilization so back-to-front sweeps (`tuli`, `tulli`) don't turn into `[j]` |
| Quantity | Short stressed vowels `0.55×` vs doubled vowels `1.40×` (`~2.5×` contrast, matching Kuronen's ~50 % `V/V:` ratio); medial unstressed vowels stay unreduced (`1.0×`, `lämpöä`, `tähtönen`); geminates `2.2×` in Speak and consolidated into a single lengthened onset (`155–175 ms`) in Sing |
| Clusters | `ng` → long `[ŋː]`, `nk` → `[ŋk]`, coda `r` before a consonant (`turma`) lengthened so the trill strikes cleanly before the next closure, `l̪` after a voiceless stop (`Revontulet leiskuvat`) given 88 ms |
| Stress & intonation | Primary stress on the first syllable, secondary stress on odd non-final syllables, statement intonation |

## Not yet

- Unhyphenated compounds with an opening diphthong in a non-initial root (e.g. `lastenhuone`) split `uo` as `u-o` unless written with a hyphen (`lasten-huone`).
- Boundary gemination (*rajageminaatio*, e.g. *tule tänne* → *tulettänne*) is not modelled.
- No scramble-test material in Finnish yet.
