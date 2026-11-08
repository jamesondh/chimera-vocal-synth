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

## Tajweed

**Annotations.** The reader annotates the tajweed of the written text, each rule on a span of it,
with the rule names of [cpfair/quran-tajweed](https://github.com/cpfair/quran-tajweed) (riwāyat
Ḥafṣ, CC BY 4.0, built from ReciteQuran.com and the Dar al-Maarifah tajweed muṣḥaf): the madds,
ghunna, qalqala, nūn and mīm sākin and tanwīn, idghām of like and near letters, hamzat al-waṣl,
the sun lām, silent letters. `chimera tajweed score` reads every āya of the Qurʾān alone and scores
them against that reference (`tools/fetch-tajweed.sh` fetches it to `~/datasets`; it isn't in
git): **F1 99.8 %** over 60,057 reference spans (recall 100.0 %, precision 99.7 %; twelve rules at
100 %, the lowest ghunnah 99.4 % and silent 99.2 %). Getting there fixed the reader too: Uthmani
script leaves the sukūn off a letter that merges or hides (يُنفِقُونَ was "yunafiqūn").

The colouring follows the muṣḥaf: a written madd letter of two counts isn't coloured (only the
dagger alif and the ṣila), the lengthened madds are those written with the maddah, a hamzat
al-waṣl that starts a phrase is said and not coloured.

**Recitation** (`ArabicOptions::recite`, `render arabic --recite`, the page's Recite toggle):
Ḥafṣ by way of al-Shāṭibiyya, from the annotations: madd muttaṣil and munfaṣil 4 counts, lāzim 6,
ʿāriḍ 4 (2/4/6 a setting), natural 2; ghunna 2 counts; idghām with ghunna (into a nasalized ي or
و, the velum lowered: `Segment::nasalized`), iqlāb (n → m), ikhfāʾ (a nasal at the next letter's
place, held), ikhfāʾ and idghām shafawī; like and near letters merged; the disjoint letters read
as names (ʔalif lāːːːm mīːːːm); no word accents, the āya drifting down to a fall at its end.
`tests/tajweed.rs` checks the readings and counts, and in a render the velum open through a
ghunna and the lips shut through iqlāb. Whisper hears the recited Qurʾān lines at CER 0.0 %, as
it hears them spoken.

Not yet, and why:
- **Qalqala's echo.** The stop is released, but a bounce added as a short vowel, even 15 ms in
  pause, made Whisper hear another syllable (أَحَدْ → "أحوال"). It needs a bounce in the release
  itself: the stop releasing into a brief voiced burst without an oral vowel target.
- **Ikhfāʾ** is a full nasal closure at the next letter's place; in ikhfāʾ the tongue only
  approaches it. A partial closure with the velum open is the better model.
- **The recitation's melody**: a maqām instead of a falling line. `plan_song`'s pitched segments
  can carry it.
- **Tartīl's pace**: a 0.12 s count lost words to Whisper (Qurʾān CER 7.0 %); recitation keeps
  0.1 s. Slower recitation needs crisper transitions, not just longer segments.
- **Settings a reciter chooses**: madd munfaṣil 4 or 5, ʿāriḍ 2/4/6 per verse end, waqf signs as
  choices (ۖ ۗ ۚ ۛ), tafkhīm by degrees.

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
| reader, trills, narrower back fricatives | 63.0 % | 87.0 % | 14.7 % | 42.4 % |
| now (+ glottis closes by the release, back frication) | 63.0 % | 87.0 % | 12.8 % | 32.9 % |

Contrasts carried now: ʕ/ʔ, ħ/h, emphatic/plain and voicing 100 %; interdental and length 83 %;
ʕ/ʁ and q/k 75 %; ħ/χ and ش/س 50 %. Male voice 96 %, female 78 %. The training sentences that
come out exactly include the basmala, al-Fātiḥa's second verse, السلام عليكم, صباح الخير, أنا أحب
اللغة العربية and قل هو الله أحد.

"Heard" is low next to "carried": for single words Whisper's prior leans to the commoner word
(قلب over كلب, حب over هب, جمال over جمل), and the voice's contrasts are real but small.

Fixed in the shared engine (judged by both speech judges):

- **The glottis closes by a voiceless fricative's release.** It stayed open ~23 ms past it, so a
  short vowel after one was voiced late (/ɪ/ in "city" 0 %, the fatḥa in كَيْفَ حَالُكَ 37 %; now 53
  and 61 %). Opening it later as well voiced more, but tripped the /f/-click judge, so the
  opening is as it was.
- **A constriction just inside the mouth kept its frication.** Its noise source was dropped when
  its upstream branch fell on the pharynx–mouth junction: χ was 29 dB under the vowel, now 17.
- **A back obstacle source**: a jet from 5.5 cm or more behind the teeth (χ ʁ q ħ ʕ, not the
  velars) strikes the soft palate and the bend of the airway. ħ 7 dB louder.

English (tools/asr/speech_wer.py, small.en): training WER 38.74 → 37.42 % (no clear change);
held-out 44.33 → 48.0 %, all in the unpredictable sentences, which don't decide but are worth
watching. The singer judge (CSD, a 1.85 GB download) was not run.

Still limiting: ħ/χ (50 %), and the female voice's pairs (78 %).

Tried and reverted (Arabic-only knobs): a faster count (0.08 s; CER 14.4 → 19.9 %) and short
vowels at 0.8 of a count (CER 14.7 → 17.6 %): this voice needs the time.
