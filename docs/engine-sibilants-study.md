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

## Score-first follow-up (2026-11-09, `sibilant-score`)

No sound change was retained. This run used the release CLI built from `d16f95d`,
explicit /Cɑ/ syllables on one 100 BPM note, and the unchanged eight-shuffle
SenseVoice sibilant probe (544 syllables, both bodies). The fricative window was
120–30 ms before the vowel beat; affricates used 80–10 ms. Levels are RMS dBFS;
peaks are the strongest 250 Hz spectral bin between 1 and 10 kHz. The reproduced
baseline confusion counts differ from the earlier study's counts (for example
c→ch 38 here versus 47 there), so all comparisons below use this run's own
baseline and identical shuffle seeds.

| Baseline / final | Male level / peak kHz | Female level / peak kHz |
|---|---:|---:|
| s | −29.2 / 3.375 | −37.5 / 3.625 |
| ʃ | −32.3 / 2.125 | −40.1 / 2.125 |
| ʂ | −38.8 / 2.125 | −45.3 / 2.125 |
| ɕ | −28.8 / 2.625 | −38.9 / 3.125 |
| f | −36.7 / 4.625 | −46.8 / 5.125 |
| ts | −27.1 / 3.625 | −35.4 / 3.875 |
| tʂ | −37.9 / 2.125 | −43.7 / 2.625 |
| tɕ | −27.1 / 2.625 | −37.1 / 3.125 |

The baseline sibilant probe scored 41.9/36.8% (male/female). Its named initial
confusions were c→ch 38, f→sh 8, s→sh 10, z→zh 5, q→t 16, and sh→ch 4.
The English Harvard baseline scored 66.2/68.1%; its phoneme confusions included
SH→F 4, S→F 10, DH→L 10, and 12 dropped SH phonemes.

| Trial (all reverted) | Acoustic effect | Sibilant probe male/female | c→ch / f→sh / s→sh / q→t |
|---|---|---:|---:|
| **Score:** retroflex jaw 0.35, calibrated | ʂ male +3.8 dB, female +0.2; tʂ male +6.9 | 40.8/36.4% | 38 / 10 / 10 / 20 |
| **Score:** retroflex area 0.15→0.10 cm² | ʂ −0.5/−0.3 dB | Not judged; level moved away from target | — |
| **Score:** affricate closure 55→40% | Release level nearly flat | 42.6/37.1% | 37 / 6 / 7 / 12; q→p 3→10 |
| **Score:** retroflex tip x 5.45→5.65 cm, calibrated | ʂ +10.8/+9.0 dB; tʂ +12.2/+10.3 | 38.6/36.8% | 39 / 10 / 4 / 22 |
| **Score:** /ts/ closure and release jaw 0.30 | ts peak 3.625→3.875 kHz male; level −0.5 dB | 40.4/37.9% | 37 / 7 / 10 / 18 |
| **Score:** /s/ tip x 6.30→6.50 cm, calibrated | s peak 3.375→4.375 kHz; ts peak likewise | 41.2/37.9% | 38 / 11 / 8 / 19 |
| **Engine:** long-jet injection +0.6 cm maximum | ʂ +2.9/+2.2 dB; short jets flat | 39.7/36.8% | 36 / 10 / 8 / 22 |
| **Engine:** long-jet injection +1.0 cm maximum | ʂ +5.0/+2.2 dB; short jets flat | 42.6/37.5% | 38 / 11 / 11 / 27 |
| **Score:** /ɕ/ tip x 5.85→5.95 cm, calibrated | tɕ male peak 2.625→3.625 kHz; female level +3.2 dB | 36.8/38.2% | 37 / 11 / 9 / 14 |

Other measured score checks: ʂ tip x 5.55 cm left it at −36.9/−45.2 dB,
while x 5.60 cm already jumped to −27.9/−36.4 dB. Raising voiceless
fricative abduction 0.30→0.40 lifted levels only 0.2–0.7 dB with no peak
movement. Raising aspirated affricate abduction 0.36→0.50 lifted releases
only 0.1–0.3 dB. For ʃ, jaw 0.20 and 0.50 gave −32.9/−40.6 and
−31.8/−42.4 dB respectively; area 0.25 and 0.15 cm² gave −33.6/−41.3
and −32.2/−39.9 dB. None met the level target in both bodies, so they were
reverted before recognition.

The same generated-table calibration was run after every area or tip-position
trial. It measures the minimum airway area, not telemetry's section means.
At x 5.55→5.60 cm the retroflex calibrated ttY jumps from 9.740 to 9.622,
and the dominant noise node moves forward; that makes output level sensitive
to a tiny score change. A source-position model that distributes turbulence
smoothly across adjacent branches is a better next experiment than another
phoneme-specific gain. It must be judged on c→ch and q→t, which stayed bad or
worsened in the gain trials.

The paired default-poem guard for the rejected +1.0 cm engine trial was:
Mandarin 30.9/30.9→29.1/30.3% and Korean 58.8/56.6→60.6/57.2%; Mandarin
sh→f stayed 6 and q→t rose 7→8. The /ɕ/ score trial's male real sibilant
line worsened 26.5→35.3% despite its male scrambled improvement. These are
counterevidence to keeping either change. The ASR runs are one melody and one
recognizer; CSD, TSP, Whisper, and listening were not run. Since every sound
edit was reverted, no golden re-bless, Rust sound tests, Bun tests, or bench
claim applies to this follow-up.
