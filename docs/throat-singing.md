# Make a throat-singing demo

Render a synthetic control track through CHIMERA, then fit tract postures for other overtone
targets. These steps need no recording or Python environment.

## Render the voice

Run this command from the repository root:

```sh
cargo run --release -p chimera-cli -- render track examples/bass-overtone/track.json out/bass-overtone/voice.wav male --tele out/bass-overtone/voice.tele
```

Play `out/bass-overtone/voice.wav`. The eight-second demo starts with regular pulses at 150 Hz.
It then attenuates alternate pulses by 85 percent, so the pulse pair repeats at 75 Hz.
Next, tract controls focus a harmonic near 1,500 Hz while the low voice continues.
The final second releases the breath.

To play the animated head, follow the [bass-overtone example](../examples/bass-overtone/README.md).
Keep generated audio and telemetry in the ignored `out/` and `public/shows/` folders.

## Fit overtone postures

The `overtone` example fits tongue, lip, jaw, and pharynx positions to target frequencies.
Run these commands from the repository root:

```sh
mkdir -p out/bass-overtone
cargo run --release -p chimera --example overtone -- 1500 2200 3000 > out/bass-overtone/postures.json
```

Supply target frequencies from 800 to 3,500 Hz. The example writes a JSON array with the fitted
`artic` values and calculated frequency response for each target.
Copy a result's `artic` object into the frames of a control track.
Set `tipFree` to `false` in those frames to use the independent tongue-tip targets.
See [Control tracks](control-tracks.md) for the format and rendering command.

The fit rejects near-closed average tube areas and penalizes competing high resonances.
Its frequency-domain response omits mean-flow losses, so check a rendered tone as well as the
fitted response. The voice engine generates the overtone through its airway.
It adds no whistle oscillator or output equalizer.

Run the rendered-audio regression:

```sh
cargo test -p chimera-cli render_track_can_focus_a_high_harmonic_while_retaining_the_low_voice
```

The test checks both the high harmonic and the continuing low voice.

## Model limits

`alternatingAttenuation` suppresses alternate glottal area pulses without changing their timing.
Its default is zero, which preserves the existing voice presets.
The engine's ventricular folds provide static narrowing only.

Two studies guide the model:

- [Lindestad et al., 2001](https://doi.org/10.1016/S0892-1997(01)00008-X) used high-speed imaging
  of bass throat singing. The ventricular folds closed at half the true-fold rate and reduced
  alternate airflow pulses. The engine approximates this effect with alternate glottal area
  attenuation. It does not model the second tissue oscillator or its pressure coupling.
  Area attenuation is not a calibrated measure of airflow attenuation.
- [Bergevin et al., 2020](https://elifesciences.org/articles/50476) studied Sygyt overtone focusing.
  Tongue and pharyngeal constrictions bring two vocal-tract resonances together.
  The high tone can change while the underlying pulse rate stays stable.

The demo tests a bass-style source approximation. It does not reproduce every throat-singing
technique or a specific performance.
