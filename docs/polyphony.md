# Polyphony: what a voice costs

The plugin plays chords with one whole voice for each note (`docs/plugin.md`, "Voices"). All the
voices of a plugin run on one audio thread, so the cost of one voice decides how many fit. This is
the log of making a voice cheaper without moving one sample of its sound.

## The rules

- **The judge** is `cargo run --release -q -p chimera --example bench`, last line: 6 male voices at
  48 kHz (2× oversampled), each saying the golden test's sentence 0.21 s after the one before, so
  every voice is moving all the time. It prints the best of five runs, as a percentage of one core
  in real time. Runs differ by about 0.3 points on a quiet machine. It measures the engines alone,
  at this rate: the plugin adds its expression glides and the limiter on the sum (not measured).
  At 48 kHz the airway already steps at 96 kHz (2× oversampled), and a host at 96 kHz steps it at
  the same rate (1×); a host at 192 kHz steps it twice as often.
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
| 1 | The node ramps are bound by their 2 divisions and a root, one node at a time, because the loop also holds the rare squeeze of a closing node's air into its neighbours. Split: a plain loop for the arithmetic (the compiler does two nodes at a time), then the squeeze in node order, only on a sample that has one, with each neighbour's area from before or after its ramp as the old loop saw it. | 92.6% | 80.8% | yes |
| 2 | The wall-movement source's low-pass is three multiply-adds a node, but it indexed four `Vec`s through `self`, so each access was checked and the loop ran one node at a time. The same statements on slices cut once. | 80.8% | 75.5% | yes |
| 3 | The wall loop has a division a node behind a "has this node a wall" test, and a sum that must stay in node order. A plain loop for each run of wall nodes (found once, when the network is built), then the sum by itself. | 75.5% | 66.0% | yes |
| 4 | The node pressures are 2 divisions a node, one node at a time, because the loop also holds the junction floor, the sealed cells (which read their neighbours) and the radiating nodes (whose sum has an order). Split: a plain loop for each node's compliance, the floor for the junctions only (a list made once), a plain loop for the pressure step; then, on a sample with no sealed cell (nearly all of them), a plain loop that adds the step and the radiating nodes in node order. A sample with a sealed cell takes the old loop, node by node. | 66.0% | 59.1% | yes |

## Result

Commit of attempt 4: **59.1%** for 6 voices (four runs: 58.9% to 59.2%), 9.9% a voice, from 92.6%.
The one-voice line went from 15.3% to 9.9%. The hash is the same and the golden tests pass with no
re-bless (392 Rust tests). The target is met, so the loop stops here.

## What was not done

Each of these is a proposal, not work in progress. None was measured.

- **The branch loop** (14% of the start profile, about a fifth of what is left). Its flow solve has a
  root and a division a branch behind a sign test. The same split as attempts 1 to 4 fits, and the
  result stays bit-exact: the arithmetic in a plain loop, then the sums into the nodes in branch order.
- **The tract geometry** (20% of the start profile, about a third of what is left). It runs each
  control step (64 samples) for each voice. Voices with one mouth pose, as the notes of a chord
  would have, could share one geometry. That is exact only while their bodies are the same and
  their breath does not move the geometry; this needs a check before any work.
- **Voices side by side in one SIMD loop.** The plain loops already use both lanes of a NEON
  register for one voice, so two voices in the same register give nothing more on this machine.
- **Memory layout.** One voice's airway state is about 76 nodes of a dozen `f64` arrays, near 10 kB:
  it fits the L1 cache. No miss showed in the profile, so no gain is expected.
- **A voice with no note is rendered like one that sings.** With Voices at 6 and one finger down,
  the plugin pays for 6 voices. To stop a voice when its sound is below a threshold cuts its tail,
  so it changes samples (only with more than one voice). It is the largest gain in real use and it
  is a change to the sound: it was left out by the rules above.
