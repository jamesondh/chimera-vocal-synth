# Engine frication and nasal place study (2026-11-09)

Eight fixed scrambles, both voices, SenseVoice, with the probes in `tools/asr/probes/`.
The judge files were unchanged. Percentages are mean scrambled syllable errors (male /
female); pair counts combine both voices (544 sibilant or 480 nasal syllables). The
default poems used the same eight shuffles as a guard. Counts matter more than changes
of about two points in the aggregate score.

## Sibilants

The baseline `/ˈɑCɑ/` 60 ms frication windows already separate the tract spectra:
male /s/ −26.7 dB, 5.11 kHz centroid and /ʃ/ −30.9 dB, 3.37 kHz; female /s/
−35.2 dB, 5.73 kHz and /ʃ/ −40.6 dB, 3.35 kHz. In `/ˈɑCɑ/` affricates, the
male /ts/ release is −27.2 dB, 4.80 kHz and /tʂ/ is −38.8 dB, 3.17 kHz. The
short-jet source was too low in frequency despite that existing separation.

| Trial | Probe male / female | Relevant initial confusions | Verdict |
|---|---:|---|---|
| Baseline | 50.4 / 40.1% | c→ch 47, s→sh 13, z→zh 11, f→sh 11, q→t 20 | Reference |
| Extra wake source halfway to teeth | 49.3 / 43.8% | s→sh 10, q→t 25, new sh→q 8 | Reverted: /ʃ/ centroid rose to 4.66 kHz male and the probe worsened. |
| Short-jet low-pass lift only | 49.6 / 40.4% | c→ch 48, s→sh 14 | Reverted: /s/ gained 3–4 dB without improving recognition. |
| Short-jet source band | **47.4 / 39.3%** | c→ch 47, s→sh **7**, z→zh **8**, f→sh 14, q→t 21 | Kept: initial errors 199→188, final errors 123→114. |

The kept source uses the constriction-to-incisor distance in body-scaled cm, not a
phoneme label. For short jets the obstacle noise rolls off at up to 10 kHz and its
high-pass corner rises to 4.7 kHz; longer jets retain the 5 kHz / 1.2 kHz band.
The measured `/ts/` release centroid moves to 5.49 kHz male and 6.43 kHz female,
while `/tʂ/` stays at 3.17 and 3.06 kHz. A preliminary 0.8–1.8 cm distance
range changed almost nothing; the measured /s/ constriction is about 2.0 cm behind
the incisors, /ʃ/ about 2.6 cm, so the final smooth range is 1.8–2.4 cm.

The default-poem guard (scrambled error, male / female) was Mandarin 37.2 / 34.4%
→ 37.2 / 33.8%, Korean 62.5 / 55.9% → 61.6 / 55.6%, and English 65.2 /
73.5% → 65.9 / 74.1%. The English increases are small but both point upward;
the targeted gain is mostly s→sh and z→zh, with no gain for c→ch or f→sh.

## Nasals

The baseline nasal probe is 59.6 / 55.4%. Its main final confusions are in→ing
42, ing→in 17, eng→en 15, uang→uan 14, and ang→an 12. A geometry probe of
`/ˈɑnɑ/` and `/ˈɑŋɑ/` showed full oral closure for both; /n/ closed near oral
section 10, /ŋ/ near section 1, and the velum reached about 0.45 cm². The
baseline murmur spectra differ, particularly in the male 2–4 kHz band. A missing
closure is therefore unlikely to be the main fault.

| Trial | Probe male / female | Targeted -ng→-n counts (eng, uang, ang) | Verdict |
|---|---:|---:|---|
| Baseline | 59.6 / 55.4% | 15 + 14 + 12 = 41 | Reference |
| Nasal viscous factor 2.5→1.5 | Not judged | Murmur contrast barely changed | Reverted before ASR. |
| Velum target 0.45→0.8 cm² | 65.8 / 57.1% | 9 + 10 + 14 = 33 | Reverted: overall probe +4 points, sh→s 27→33. |
| Velum target 0.45→0.6 cm² | 62.1 / 55.8% | 15 + 11 + 14 = 40 | Reverted: no clear target gain, overall probe worse. |

The larger port helps some -ng→-n confusions but also changes unrelated
recognition. It does not justify a default change. Next, inspect the vowel-to-coda
formant trajectory and oral side-branch antiresonances for /n/ and /ŋ/ in both
bodies; also compare the recognizer's in→ing errors, which are as large as the
reverse place errors. A source adjustment should address c→ch and f→sh without
spending English's small guard margin.

## Validation and limits

The kept source passed two `CHIMERA_BLESS=1 cargo test --workspace --no-fail-fast`
runs (the first had the documented beatbox fixture ordering failure), then the
normal Rust workspace tests, `bun run test` (30 tests), and `bun run typecheck`.
Per the hill-climb workflow the blessed golden files were restored after checks;
they are intentionally out of date on this experiment branch. The paired release
bench was 25.2% baseline → 25.0% with the source at male 2× oversampling. CSD,
TSP, Whisper and human listening were not run for this targeted SenseVoice task.
