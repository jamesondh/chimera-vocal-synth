# Throat singing

`chimera throat` sings Tuvan overtone singing (khöömei, sygyt) and kargyraa through the human
airway. The melody comes from the vocal tract alone: there are no filters and no added partials.

## How it works

- **Khöömei and sygyt are tract tuning.** One harmonic of a pressed drone becomes the melody when a
  sharp tract resonance sits on it. `throat solve` searches the existing articulators (tongue body
  and tip, jaw, lips, tongue root, larynx) with the frequency-domain airway model. It wants the
  harmonic that lifts harmonic *n* over *n*±1 and *n*±2, and over everything else above about 450 Hz.
  - The target is scored as its minimum over ±0.2 f0. The FDTD is lossier than the frequency-domain
    model, and it blunts a needle-thin resonance into its neighbours.
  - The output is scored with a −10 dB/octave source roll-off, so F1 can't win on harmonics 4–5.
  - Every harmonic up to 4 kHz is scored.
  - All the notes of a melody are solved near one home posture (lips, jaw and larynx weigh 10× the
    tongue). Each note's own best posture is tried as the home, and the best whole melody wins.
  - Each style bounds the search to its mouth. Khöömei has the lips forward and a little open.
    Sygyt has the lips nearly closed and spread, the tongue tip raised, and also weighs a dark
    treble.
- **Low drones** need a narrow epilarynx (0.9) and full fold tension (1.0), and a whistle above
  about 1.1 kHz. Below about 60 Hz that means harmonics 24–36.
- **Kargyraa** is the ventricular folds closing on every other glottal cycle (Bailly, Henrich &
  Pelorson 2010).
  - `VentricularFolds` is an area waveform locked to the glottal cycle, like the parametric glottis.
  - It caps the minimum area of the pharynx node at the ventricular band, which sets the jet's
    Borda–Carnot loss and its turbulence.
  - Uncapped, the airway is bit-identical.
- **The breath ends the phrase.** `throat song` integrates the glottal volume velocity the airway
  computes against each breath's lung volume.
  - The last 12% of the air sags the pressure.
  - A phrase ends with the score, or when 2% of the air is left (`*` holds a note until then).
  - Each breath starts with an inhale: negative lung pressure through the open glottis, which makes
    its own turbulence noise. Then the drone scoops up from 0.6 f0, and a release lets the breath
    and the pitch fall away together.
  - The lungs refill by decree, not physically. The model's glottis can't inhale 3 L/s.

## Use

```sh
B="cargo run --release -q -p chimera-cli --"
$B throat solve 82 16 18 20 24 --style khoomei         # postures, lead over the neighbours
$B throat render out/khoomei 82 16:2 18:2 20:2 24:3     # one drone, a melody of harmonics
$B throat render out/kargyraa 57.5 9:4 10:4 12:4 --style kargyraa
$B throat song song.txt out/song --tele out/song.tele   # breath-driven, telemetry for ?show=
```

A song is one line per breath:

```
# style  drone(Hz)  harmonic:secs …                         options
khoomei  115        8:3 9:2 10:2 12:3 10:2 9:2 8:*          lungs=3.3   rest=2
kargyraa 57.5       9:5 10:5 12:5 10:*                      lungs=3.5   pressure=8
sygyt    172.5      8:5 9:3 10:4 9:*                        lungs=3.0
```

- `lungs=` is the breath's air in litres.
- `pressure=` is the lung pressure in cmH₂O. Softer singing costs less air.
- `rest=` is silence before the inhale, for lining up voices that are rendered separately.

The log gives each breath's start time and why it ended (`by the air` or `by the score`).

## Measured

Melody lead is the median, over 0.25 s frames, of the melody harmonic over its louder neighbour.
The same meter gives 9–11 dB on real khöömei, sygyt and ezengileer recordings.

| | rendered | real |
|---|---|---|
| khöömei, 136 / 82 / 65 / 49 Hz drone | 12 / ~10 / 9 / 7 dB | 9–11 dB |
| sygyt, 155 Hz | 13.6 dB, treble −17 dB | treble −11 to −30 dB |
| kargyraa, half-way partials against the harmonics | −1.4 dB | +0.2 / +1.1 dB |
| kargyraa vowel melody | 5.5 dB | 5.6 dB |
| airflow | 110–115 mL/s (kargyraa 80), so 3.5 L lasts ~30 s | phrases of 15–35 s |

## Not done

- No browser UI or wasm ABI for any of this yet.
- Kargyraa's melody moves mostly F2. Real singers move F1 and F2 together.
- A faint transient remains where the glottis opens for the gasp.
- Borbangnadyr and ezengileer have no style of their own.
