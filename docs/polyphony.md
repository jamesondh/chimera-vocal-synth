# Polyphony: what a voice costs

A chord is one whole voice for each note, and a plugin runs all of its voices on one audio thread,
so the cost of one voice decides how many fit. This is the log of making a voice cheaper without
moving one sample of its sound.

## The rules

- **The judge** is `cargo run --release -q -p chimera --example bench`, last line: 6 male voices at
  48 kHz (2× oversampled), each saying the golden test's sentence 0.21 s after the one before, so
  every voice is moving all the time. It prints the best of five runs, as a percentage of one core
  in real time. Runs differ by about 0.3 points on a quiet machine. It measures the engines alone,
  at this rate. At 48 kHz the airway already steps at 96 kHz (2× oversampled), and a host at 96 kHz
  steps it at the same rate (1×); a host at 192 kHz steps it twice as often.
- **The guard** is the golden tests, with no re-bless, and the hash on the bench's line (the sum of
  the six voices), which must stay `0x4178d7c63e6cf860`. A change that moves one sample is a change
  to the sound: it goes through `docs/hill-climb/README.md`, not here.
- **The target** is 6 voices under 60% of one core on the machine below (10% a voice).
- **The loop:** profile, one hypothesis, change, bench and goldens, keep or revert, one line here.
  It stops at the target, or after three attempts in a row under 3% each, or when only changes to
  the sound are left.

Machine: Apple M-series (arm64), macOS, rustc 1.94.1, `--release` (fat LTO, one codegen unit).

## Where the time goes

Start, the commit that adds the bench's last line: **92.6%** for 6 voices, 15.4% a voice (15.3%
for the one-voice line).

A line profile of the 6-voice bench (2,500 stops under lldb, the innermost line of this repo's code):

| Part | Share |
|---|---|
| `AirwayNetwork::step`, node ramps: area, 1/A, 1/Amin, √A, volume (2 divisions and a root a node) | 22% |
| `step`, branches: the flow solve (a root and a division a branch) | 14% |
| `step`, lines the compiler gives no source line | 10% |
| `step`, node pressures (2 divisions a node) | 10% |
| `step`, the wall-movement source's low-pass | 8% |
| `step`, walls (a division a node) | 7% |
| `step`, the rest | 5% |
| Tract geometry, each control step (64 samples): `HumanVocalTract::compute`, `project_on` | 20% |
| `HumanVoice::render` itself | 3% |
| The glottis, the decimator, the score, the articulators | 1% |

The airway has 76 nodes and 75 branches and steps 96,000 times a second: 1.3 µs a step.

## Attempts

| # | Hypothesis and change | Before | After | Kept |
|---|---|---|---|---|
