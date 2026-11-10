# Voice source and sibilant experiment (2026-11-09)

The eight-shuffle SenseVoice test used the same tune, texts and shuffle seeds in both voices. Values
below are mean scrambled error percentages, male / female. Differences around two points are within
the observed noise. Trial code was reverted unless stated otherwise.

| Render | Chinese syllables | Korean syllables | English words |
|---|---:|---:|---:|
| Baseline | 49.7 / 47.5 | 62.5 / 55.9 | 65.2 / 73.5 |
| Automatic phonation trial | 48.8 / 49.7 | 62.8 / 55.9 | 65.2 / 73.5 |
| Teeth-source spectrum trial | 49.7 / 46.6 | 60.3 / 56.2 | 67.4 / 72.9 |

The phonation trial put `fry=0.55` near each Mandarin tone-3 vowel's low point and `tension=0.95`
from Korean tense stops into the following vowel. Neither was a robust gain; both defaults were
reverted. The existing fry control pulls a 220 Hz note toward 48 Hz: at 0.55 it produces roughly
125 Hz before the tone's own pitch bend. That pitch cost likely swamped the creaky cue. The retained
`Segment.phonation` preset uses milder fry (0.25) and is opt-in; it has not been judged by ASR.

The teeth source is white noise passed through a first-order high-pass and two low-pass poles,
injected at the lower incisors when a narrow jet reaches them. In `/ˈɑCɑ/` (60 ms of frication,
male), baseline RMS / centroid were: /s/ −26.7 dB / 5.15 kHz, /ʃ/ −30.7 / 3.04, /ɕ/ −28.4 /
3.98, /ʂ/ −36.9 / 3.49. Female: −35.4 / 5.70, −40.3 / 3.20, −39.6 / 4.40, −45.1 / 3.23.
So the tract already separates the spectra, but the longer-cavity sibilants are weak, especially
in the female body.

The trial varied the teeth source's passband and gain with the measured constriction-to-incisor
distance, and extended the far-distance fade from 4.0 to 4.3 body-cm. It moved the male probe to
/s/ −24.4 dB / 5.02 kHz, /ʃ/ −26.8 / 2.85, /ɕ/ −25.3 / 3.81, /ʂ/ −28.6 / 2.93; the female
changes were similar. An earlier, wider fade overshot /ʂ/ by about 19 dB and was discarded before
the recognition run. The restrained trial did not clear the recognition noise and worsened male
English by 2.2 points, so it was reverted. A source with a front-cavity resonance and blade/jet
geometry needs a targeted judge before another gain or cutoff change.

This machine had no CSD or TSP dataset, so the singer guard and `chimera compare tsp` were not run.
The English scramble is the regression check here; one recognizer and one melody cannot establish
human listening quality.

The retained optional gesture path passed both golden bless runs (no fixture diff), the normal Rust
workspace tests and the no-allocation ABI test. In paired release bench runs, the median male 2×
load was 26.7% of one core at baseline and 27.6% with the new keys; individual runs varied by
several points. An IPA render with phonation unset was sample-identical to baseline. `bun run test`,
`bun run typecheck`, `bun run build`, and the desktop/mobile Chromium smoke test passed; the latter
reported no console errors using local software-rendering libraries.

## Two-mass folds: design and cost

`ParametricGlottis::next(dp)` already returns area to the interactive glottal port. A two-mass
oscillator could use that boundary and remain an opt-in voice mode. It needs two displacements and
velocities, collision/contact, aerodynamic force from the tract pressure and flow, a stable
integrator at the 96 kHz internal rate, and a tension controller that tracks each note's requested
f0. It must preserve the current model as the default and satisfy no allocation, deterministic
block-size-independent output, native/wasm sample parity, and safe behavior under closures and
large pressure changes.

Estimate: several days for an isolated oscillator and stability harness, then one to three weeks
for tract coupling, pitch tracking, body tuning and regression tests. The largest risks are pitch
drift on sung notes, collision instability and higher per-sample CPU cost. The opt-in gesture keys
address the immediate control gap at much lower cost; a two-mass prototype is not yet justified by
these recognition results.
