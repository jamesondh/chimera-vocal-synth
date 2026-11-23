# Polish

Speak and Sing read Polish once you choose it: **PL**, next to EN and TR. It is one setting for
both tabs, kept on the device with Sing's words. Polish is written in Latin letters, so the lyrics
router can't tell it from English by script ([language-front-ends.md](language-front-ends.md));
English stays the default. Hangul, Cyrillic and Han characters in the same lyrics still go to their
own front ends. Toned pinyin doesn't: ó is both a Polish letter and a pinyin tone mark, and
pinyin's scan stops at ł or ź, so it took "pó" from "pół" and "późno".

`crates/chimera/src/phonetics/polish.rs` reads the spelling by rule (standard, Warsaw
pronunciation), with no dictionary, onto the existing inventory (no new sounds).

## Writing it

- Type it as you'd write it: ą ć ę ł ń ó ś ź ż.
- In Sing, a vowel is a note, and syllables are kept open: the next note takes the consonants
  between two vowels (do-bry, Szcze-brze-szy-nie), except that a sonorant before another consonant
  stays with the vowel before (Pol-ska, ręn-ka) and a doubled consonant is sung on both sides
  (An-na). `-` alone holds the previous vowel onto the next note, and hyphens split a word your way: the
  word is still read whole, so "rę-ka" keeps ręka's [ŋ], and only the break moves (proś-ba).
- A word with no vowel ("w", "z") is said and sung with the next word ("w do-mu").

## What it does

| | |
|---|---|
| Digraphs | one sound each: sz [ʃ], cz [tʃ], rz and ż [ʒ], dż [dʒ], ch and h [x], dz [dz]; rz is r and z in a few words (marznąć). The sz series is the inventory's postalveolars: its retroflexes are weak (ʂ and ʐ hiss 6–8 dB below ʃ and ʒ, and tʂ never quite closes), so they sounded mushy, and Polish sz is flat, not curled |
| i | before a vowel it only softens the consonant: si zi ci dzi ni are [ɕ ʑ tɕ dʑ ɲ] (się [ɕɛ], dzień [dʑɛɲ]); labials, l and ch are palatalized (pies [pʲɛs]); k and g front to [c ɟ] (kiedy [cɛdɨ]); t, d and r take a [j] in loans (radio [radjɔ]). Before a consonant or at the end it is also a vowel (nic [ɲits]) |
| ę ą | a vowel and a nasal at the next consonant's place before a stop or affricate (ręka [rɛŋka], ząb [zɔmp], pięć [pʲɛɲtɕ]); oral before l, ł and a nasal (wziął [vʑɔw]); nasalized before a fricative and at the end (mąż, są); a final ę is oral (się [ɕɛ]) |
| Voicing | a word's last obstruent devoices, also before a final sonorant (chleb [xlɛp], bóbr [bupɾ]); an obstruent takes the voicing of the next (wódka [vutka], prośba [prɔʑba]); w and rz devoice after a voiceless one and don't voice it (kwiat [kfʲat], przy [pʃɨ]) |
| Doubled letters | in speech one long consonant, 1.6 times its length (Anna [anːa]); in song it closes one note and opens the next |
| r | one tap [ɾ]; after t or d, in speech, a 50 ms [ə] before it (tramwaj [təɾamvaj]): the tap is at the stop's place, so without it the tap merged into the stop and the voice came back only in the vowel ("tamwaj"). Sung onsets stay clusters |
| y [ɨ] | a full vowel, 1.25 times the duration rules' /ɨ/ (theirs is English's reduced vowel, 110 ms) |
| Vowel length | in speech, the other vowels are shorter than the duration rules', which are English's and stretch a stressed vowel and a phrase's last: 0.6 times stressed, 0.75 unstressed, 0.55 a phrase's last. Set from a native speaker's recordings of four sentences, aligned with MFA's polish_mfa: their consonants matched the rules', their vowels were 1.3 to 1.85 times shorter |
| Stress | the penult, with the exceptions below |
| Intonation | a question rises, unless it starts with a question word (co, kto, gdzie, jak, dlaczego...), perhaps after a conjunction ("A gdzie?"), and then falls like a statement |

Stress exceptions:

1. Clitics take none: prepositions (na, do, w, z...), pronoun clitics (się, mi, go...),
   conjunctions and particles (i, a, że, czy).
2. The antepenult: past-tense -liśmy and -łyście (BYliśmy), the conditional -łbym, -łaby
   (ZRObiłbym, CHCIAłabym), loans in -yka and -ika and their cases (mateMAtyka, MUzyce) and a
   short list (uniWERsytet).
3. The conditional -libyśmy, -łybyście: one further back (CHCIElibyśmy).

## Not yet

- r is one tap [ɾ], as in Russian: the modelled trill held the tongue tip near the ridge and
  the voicing all but stopped. Russian's tap has the same trouble after т and д (три, друг); the
  fix for both would be in the planner, which would change Russian's sound. After s the tap is
  heard but faint (srebro). A nasal vowel is the oral one with the velum down, without the [w̃]
  glide some speakers add.
- Voicing stops at a word's edge, except for a vowelless word ("w Polsce" [fpɔlstsɛ]): "jak
  było" keeps its [k].
- A vowelless word needs a word after it: in Sing, one at the end of a line or before a comma
  ("idę z") isn't sung, and of two in a row ("w z domu") only the second is said. A
  vowelless abbreviation ("PKP") is read as letters joined to the next word. A monosyllabic preposition before a pronoun (DO mnie) isn't stressed.
- The -yka list matches stems, so "muzyka" is always music, never the genitive of "muzyk"
  (musician: muZYka). ł after a consonant isn't dropped (jabłko is [jabwkɔ], not [japkɔ]).
- Numbers are not read out; type them as words.
- The line font has no glyphs for ą ć ę ł ń ó ś ź ż, so the text box draws them in its fallback
  font (as it does Turkish ı ş ğ).
- No scramble test: its recogniser, SenseVoice, knows Chinese, English, Japanese, Korean and
  Cantonese, not Polish.
- A Polish speaker has listened to Speak (cz, sz, rz and r were changed by ear); stress, the
  nasal vowels and Sing haven't had a native check.
