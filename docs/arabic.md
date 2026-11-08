# Arabic, and the road to tajweed

CHIMERA reads vowelled Arabic and says it with the simulated airway: every letter is made where
tajweed places it (throat, tongue, lips), nothing is sampled. This page is the design, the
judge, what it measures today, and the plan for the next step: tajweed and Qurʾān recitation.

## What exists

- **The sounds** (`phonetics/inventory.rs`, `planner.rs`). Arabic needed little new anatomy: the
  inventory already had q, χ, ʁ, ħ, ʕ, ʔ, θ, ð. Added:
  - **Pharyngealization (ˤ)**, the emphatics ص ض ط ظ and the heavy rāʾ and lām, as a secondary
    articulation: the plain consonant with the tongue dorsum and root pulled back toward the upper
    pharynx. A vowel written with ˤ is the calibrated vowel moved back by a per-vowel amount.
    Measured on the tract's transfer function at the vowel's midpoint in a rendered word, F2 falls
    276 Hz (/saːb/→/sˤaˤːb/), 292 Hz (/tiːn/→/tˤiˤːn/) and 72 Hz (/suːq/→/sˤuˤːq/), the size of the
    drop reported for Arabic emphatics (Jongman, Herd & Al-Masri 2011).
  - **Unaspirated /q/ and /tˤ/** (tajweed: no breath, hams, in them).
  - **Trills that trill.** They used to seal into a stop (one bug for every language; see
    `tests/trill.rs`). Arabic rāʾ is a tap between a word's vowels, a short trill elsewhere.
  - **Back fricatives narrower than the open glottis** (χ 0.1, ʁ 0.12, ħ 0.12, ʕ 0.16 cm²): at
    0.2–0.3 cm² they were as wide as the glottis and sounded like [h].
- **The reader** (`phonetics/arabic.rs`): vowelled text → segments, letter by letter, with the
  rules of correct reading: shadda, tanwīn, madd letters and the dagger alif, hamza on its seats,
  hamzat al-waṣl, sun and moon letters, waqf at phrase ends, tafkhīm of the heavy letters, rāʾ by
  Ḥafṣ's rules, the lām of Allāh, hidden long vowels (هذا, ذلك, لكن, الله, الرحمن), the Uthmani
  marks, MSA stress. Readings are tested in `tests/arabic.rs`, written by hand from the rules.
- **The page** (`/arabic`): read vowelled text with its letters lit as they are said, presets
  (greetings, words, sentences, minimal pairs, al-Fātiḥa, al-Ikhlāṣ), and the letters by
  makhraj with their ṣifāt.
- **The CLI**: `chimera render arabic <out> "نص" [male|female|child] [--count s] [--no-waqf]`.

## Built for tajweed

Tajweed is a set of changes to lengths (counted in ḥarakāt) and to segments (merged, nasalized,
echoed), decided by the letters around them. The reader keeps exactly that:

- **Durations are counts.** A short vowel is one count, a long vowel two, a doubled nasal two (its
  ghunna); a count is a tempo (`ArabicOptions::count`, 0.1 s). Recitation speeds (taḥqīq, tadwīr,
  ḥadr) are counts of different lengths, and madd of 4 or 6 is a count change.
- **Every segment knows where it came from**: its span of the text (`sources`, code points), so a
  rule can be shown on the letters it applies to, as a muṣḥaf colours them.
- **The contexts are tagged already** (`arabic::rules`): madd muttaṣil, munfaṣil, lāzim and ʿāriḍ,
  the text's own madd sign, ghunna, nūn and mīm sākin (with the consonant after them), iqlāb marks,
  qalqala letters, tafkhīm and tarqīq, waqf, the article's assimilation, hamzat al-waṣl.

The next step, in order of what a teacher corrects first:

1. **Madd lengths** (Ḥafṣ ʿan ʿĀṣim by way of al-Shāṭibiyya): muttaṣil and munfaṣil 4–5,
   lāzim 6, ʿāriḍ lil-sukūn 2/4/6 (a setting), līn in pause. A count change on tagged segments.
2. **Nūn sākin and tanwīn**: iẓhār before the throat letters; idghām into ي ن م و (with ghunna, 2
   counts) and ل ر (without); iqlāb before ب (n → m, with ghunna); ikhfāʾ before the other fifteen
   (the tongue approaches the next letter's place with the velum open: a nasalized segment, which
   the velum articulator can already make). Mīm sākin: ikhfāʾ shafawī before ب, idghām before م.
3. **Qalqala**: ق ط ب ج د without a vowel are released with an echo (a brief voiced release, a
   small vowel-less burst); stronger at a pause.
4. **Waqf signs** (ۘ ۗ ۖ ۚ ۛ ۙ): where to stop, where to continue. The reader breaks phrases at ۝ and
   at the compulsory and preferred stops already.
5. **Tafkhīm degrees** (fatḥa with alif > fatḥa > ḍamma > sākin > kasra) as graded backing.
6. **The voice of recitation**: tartīl's tempo and a melody (maqām) instead of speech intonation;
   `plan_song`'s machinery (pitched segments) can carry it.

Judges for that step: a Qurʾān-trained ASR (Whisper fine-tunes on recitation exist) with forced
alignment to check lengths in counts, the rule tags against a tajweed-annotated muṣḥaf, and a
reciter's ear.

## The judge, and where it stands

`tools/asr/arabic_judge.py` (set up with `tools/asr/setup-arabic.sh`; Apple silicon, MLX) renders
with `chimera render arabic` in the male and female voices and asks Whisper large-v3-turbo:

- **Minimal pairs, forced choice** (`arabic-pairs.tsv`, 27 pairs × 2 voices): Whisper scores both
  spellings against the audio (teacher-forced log-probability). *Heard as intended*: the rendered
  word's spelling wins. *Contrast carried*: the audio moves the preference the right way with the
  language model's prior cancelled, [s(A|a) − s(B|a)] − [s(A|b) − s(B|b)] > 0.
- **Sentences** (`arabic-sentences.tsv`): character error rate of free transcription, after
  removing vowel marks and folding hamza seats; held-out sentences never decide.

The control is the macOS Arabic voice (Majed), a real synthesizer: it sets the ceiling.

| | heard | carried | CER, training | CER, held out |
|---|---|---|---|---|
| Majed (control) | 92.6 % | 96.3 % | 1.9 % | 1.0 % |
| before (reader, emphatics) | 62.0 % | 83.3 % | 24.0 % | — |
| now | 63.0 % | 87.0 % | 14.7 % | 42.4 % |

Contrasts carried now: ʕ/ʔ, ħ/h, interdental/sibilant, length, ش/س and voicing 100 %; emphatic
94 %; q/k, ħ/χ and ʕ/ʁ 50 %. The training sentences that come out exactly: the basmala, al-Fātiḥa's
second verse, السلام عليكم, صباح الخير, ذهب الولد إلى المدرسة (male).

"Heard" is low next to "carried": for single words Whisper's prior leans to the commoner word
(قلب over كلب, حب over هب, جمال over جمل), and the voice's contrasts are real but small. What still
limits it, measured:

- **Short vowels between voiceless consonants devoice in the male voice.** In كَيْفَ حَالُكَ the
  100 ms fatḥa between f and ħ voices for ~30 ms: the glottis re-adducts 30–40 ms after a voiceless
  fricative ends and opens ~45 ms before the next. Whisper hears "كيف سعي جوك". It is the shared
  planner's timing (English has it too), so fixing it should be judged by the English speech judge
  as well: a proposal, not done here.
- **ħ/χ and ʕ/ʁ.** Narrowing the constrictions helped (25 → 50 %), but the constriction noise
  source is calibrated for weak fricatives (f, θ); a uvular or pharyngeal jet has no obstacle
  source of its own. A place-aware noise source is the next experiment.
- **q/k**: half the pairs carry the contrast.

Tried and reverted (Arabic-only knobs): a faster count (0.08 s; CER 14.4 → 19.9 %) and short
vowels at 0.8 of a count (CER 14.7 → 17.6 %): this voice needs the time.
