# Russian

Speak and Sing accept Cyrillic. A word with a Cyrillic letter is read by
`crates/chimera/src/phonetics/russian.rs`; everything else stays English, and a line can mix both.
Sung lyrics reach it as a front end on the lyrics router ([language-front-ends.md](language-front-ends.md));
Speak calls the same reader word by word.
There are no new sounds: Russian is mapped onto the existing inventory.

## Writing it

- **Mark the stress** with a combining acute after the vowel: `молоко́`. ё is always stressed.
  An unmarked word is looked up in a short table of frequent words (`STRESS`); failing that, its
  **first vowel** is stressed. That guess is wrong for most longer words, so mark them.
- In Sing, one vowel is one note. `мо-ло-ко́` forces the breaks, `~` holds the previous vowel.
- Write numbers as words: digits still go through the English number expander.

## What it does

| | |
|---|---|
| Vowels | а о у э ы и; я е ё ю add /j/ at the start of a word, after a vowel and after ь ъ |
| Reduction (speech only) | unstressed о а → /ɐ/; е, and я before the end of the word → /ɪ/; а after ч щ → /ɪ/ |
| Hard-only consonants | ж ш ц: и and unstressed е after them are /ɨ/ |
| Voicing | a final obstruent devoices (дуб → /p/); voicing spreads backwards through a cluster (ло́дка → /tk/, сдать → /zd/); в doesn't start it (твой) |
| Prepositions | в, с, к, из, над, под… are unstressed and take their voicing from the next word (в лесу́ /v/, в ти́хом /f/, с горы́ /z/) |
| Spelling conventions | -ого, -его and его, сего́дня → /v/; что, коне́чно → /ʂ/; сч, зч, жч → щ; silent т д л в in стн, здн, лнц, вств; -тся, -ться → /tsa/ |
| Doubled letters | one consonant, half again as long between vowels (ва́нна), plain before a consonant (ру́сский) |

## What is approximate

- **Soft consonants** are the plain consonant and a 25 ms /j/ glide, not a palatalised
  articulation. л is the exception: dark /ɫ/ when hard, clear /l/ when soft.
- **щ** is a long /ʃ/ with that glide, and **ч** is /tʃ/ with it; the inventory has no
  alveolo-palatals.
- **р** is one tap /ɾ/. The trill /r/ held the tongue tip near the ridge for about 100 ms and the
  voicing nearly stopped, so рыба was heard as дыба.
- **Sung prepositions**: в, с, к have no vowel and so no note; they start the next word's first
  syllable (and take its voicing). A preposition with a vowel is sung as its own word and keeps
  its spelling's voicing (над ти́хим keeps /d/).
- Not done: stress for arbitrary unmarked words, voicing across other word boundaries, loanwords
  with hard consonants before е (те́ннис), numbers.

## Tests

`crates/chimera/tests/russian.rs` has one case per rule above.
