# Flatulence

The Bestiary's `flatulence` creature: the sound of gas passing the anus, from the anatomy and
physics of the anorectum. The brief was that every modelling choice is grounded in physics and
anatomy, nothing is added because it sounds better, and where data are missing the choice comes
from physical reasoning, never from listening. The code is
`crates/chimera/src/creatures/flatus.rs`; each constant there carries its source.

```
 abdominal push ─┐
  + rectal       ▼  tissue around the pocket (Rayleigh–Plesset)
  recoil     rectal gas pocket ── upper anal canal ── ▷ anal verge ◁ ── margin ── ▷ cheeks ◁ ── gluteal cleft ── air
             (FDTD tube: the gas   (opened by the     (subcutaneous    (slot)    (contact strip  (soft-walled slot,
              law is the network)   gas, RAIR)          EAS ring:                 over the        radiating into
                                                        LipReed;                  ischioanal fat:  a baffle)
                                                        mucus bridges)            LipReed)
```

The model was then fitted to 59 events of 31 real recordings; the cheeks' valve and the mucus
came out of that validation (below, "Validation against recordings", and
[flatulence-validation-log.md](flatulence-validation-log.md)).

## Method

**Driving pressure: the rectum as a finite gas reservoir.** The gas pocket is a tube of the airway
network, so its gas law is the network's own continuity equation, (V/ρc²)·dp/dt = ΣU, the
linearised adiabatic gas law. The pocket's wall is the tissue around it. At the pressures and time
scales here the pelvic contents move like a liquid, so the pocket is treated as a bubble: its
equal-volume sphere obeys the Rayleigh–Plesset equation

  R·R̈ + 1.5·Ṙ² = (p_gas − P_out)/ρ_tissue − 4·μ_tissue·Ṙ/(ρ_tissue·R),

integrated every internal sample. P_out, the pressure on the tissue far from the pocket, is the
abdominal push plus the rectal wall's elastic recoil V/C_rectum. The wall's motion (−dV/dt) enters
the network as flow into the gas. So:

- the pocket's pressure holds while the tissue can follow;
- it falls as the gas leaves, through the recoil term, and then through the gas law alone once the
  pocket reaches its floor (the walls meet);
- the event ends when the gas is gone. No envelope is imposed.

The same equation gives the pocket a resonance, the Minnaert frequency of a gas bubble in tissue:
about 113 Hz at 100 mL, 170 Hz at 30 mL. Tissue viscosity damps it lightly.

An earlier version of this model had the rectal wall relax toward the pressure balance at control
rate with a time constant. That time constant had no physical value, and the results depended on
it (twice the time constant doubled an event's duration), so it was replaced by the tissue's
inertia.

**Source: the anal verge as an outward-striking valve.** Gas entering the canal relaxes the
internal sphincter (the rectoanal inhibitory reflex) and opens the upper canal. The closure that
remains is the distal ring, where only the subcutaneous external sphincter surrounds the canal.
Gas pressure pushes that ring open, and the ring's tension closes it. This is Fletcher's (1993)
(+,−) pressure-controlled valve, like a brass player's lips, so the model reuses the engine's
`LipReed`, the single-mass outward-striking valve the elephant's trumpet uses. Its parameters are
derived, not chosen:

- **Mass per unit area:** ρ_tissue·δ, the tissue that moves with the lip.
- **Stiffness per unit area:** the passive soft tissue's (Ishizaka et al. 1975), plus the
  sphincter muscle's short-range stiffness. For a ring of hoop tension T = P_close·r, that is
  γ·P_close/r, with γ = 23.4 (Cui et al. 2008). So the stiffness grows with tone.
- **Natural frequency:** √(k/m)/2π, computed from those two.
- **Preload:** the closing pressure. The rest gap is −2·P_close/k (the gap is twice the radial
  opening), so the valve opens when the pressure across it exceeds the tone.
- **Width:** the closed lumen's slit, so the opening area is slit × gap.

The valve's duct is a Poiseuille slit of the ring's height. It uses Ishizaka & Flanagan's
entrance and exit loss coefficients, as at the glottis.

Pitch is never set. It comes out of the lip's mass and stiffness, the pressures, and the coupling
to the pocket and the canal. The valve sounds above its mechanical resonance, as outward-striking
lips do (Newton, Campbell & Gilbert 2008).

**Mucus at the verge (control, 0 = dry).** The canal's faces can be wet. After each contact a
liquid bridge joins them. While it holds it fills the gap, so no gas passes, and it pulls the
parting faces together by Bhattacharya & Siegmund's (2014, Biomech Model Mechanobiol 14:155)
traction–separation law for the liquid on colliding vocal folds: the traction rises linearly to
the cohesive strength σ₀ at a separation δ₀ = 0.1 mm and falls to nothing at δf = 0.267 mm, where
the bridge breaks (their baseline values). σ₀ is the control, 0–3 kPa; 3 kPa is the cohesive
strength of intestinal mucus (Mortazavi & Smart 1994), the upper bound Bhattacharya & Siegmund
used. Each opening then starts late and all at once, and the openings come irregularly.

**Turbulence.** The engine's Reynolds-number noise sources, at their calibrated gains:

- At the valve's jet, the human voice's glottal aspiration source (gain 3.2e-6, band 300–3500 Hz).
  The mechanism is the same: a jet from a slit valve into a duct.
- In the cleft when it is narrow, the human narrowest-constriction source (gain 1.364e-6, band
  350–2600 Hz). It uses the human source's own area fade.

**The cheeks: a second valve.** Gas leaving the verge enters the anal margin, a 1.5 cm
soft-walled slot, and must then pass the strip (1 cm along the flow) where the cheeks meet or
nearly meet. The gas pushes them apart and the tissue springs back: a second pressure-opened
soft-tissue valve, like the lips in a raspberry, modelled with the same LipReed. Its rest gap is
the cleft wedge's mean opening over the strip: pressed cheeks touch (no preload) and the gas has
to part them; spread cheeks leave it wide open. The margin keeps the 1 mm residual gap of skin
folds and hair, so it holds gas even when the cheeks are shut. Its
moving mass is the fat under the perianal skin, which beside the verge fills the ischioanal
fossa, about 5 cm deep (textbook figure). The fat layer's first thickness mode, f = c/4L with
c = √(E/ρ), sets a single mass: stiffness E/L and mass 4ρL/π² per area. E = 3 × 7.5 kPa, the
shear modulus of subcutaneous fat (Geerligs et al. 2008, Biorheology 45:677) of an
incompressible tissue. Damping is the layer's Kelvin–Voigt loss with the tissue viscosity of the
gas pocket (3μ in extension): ζ = ω·3μ/2E. So f ≈ 23 Hz and ζ ≈ 0.02 at body size 1. The cheeks
start each sound at rest.

**Radiation through the gluteal cleft.** The cleft is a soft-walled slot from the anal verge to
the body surface. Its walls are the engine's measured soft-tissue wall. Its gap at depth z is
g₀ + 2z·tan(θ/2):

- pressed cheeks: θ = 0, leaving the residual gap g₀;
- fully spread: the cheek surfaces at a right angle.

The gap runs across the verge's width. The cleft ends in the engine's baffled radiation load,
because the body surface around the outlet acts as a baffle.

**Physical controls.**

| Control | Range | Default | Basis |
|---|---|---|---|
| Abdominal push (cmH₂O) | 0–80 | 30 | intrarectal pressure added by straining |
| Sphincter tone (cmH₂O) | 2–150 | 20 | the canal's closing pressure |
| Gas volume (mL) | 10–300 | 100 | gas in the rectum at the event's start |
| Cheek spread | 0–1 | 0.2 | the cleft's opening angle |
| Body size | 0.6–1.25 | 1 | linear scale on every length |
| Mucus adhesion (kPa) | 0–3 | 0 | cohesive strength of the liquid between the canal's faces; 0 is dry |

Body size scales the lengths and thicknesses linearly and the volumes and compliance cubically.
Per-area stiffness and the pressures don't scale. The push is ramped by the breath gate, with a
0.05 s attack and a 0.08 s release (muscle activation, an estimate).

A new event starts with the set gas volume, at a call's `!fill` or at a new push after rest. The
time between events, in which the colon refills the rectum (about 30 mL/h awake; Tomlin et al.
1991), is skipped.

The presets differ only in these controls:

| Call | Push | Tone | Gas | Spread |
|---|---|---|---|---|
| Squeak | 42 | 40 | 20 | 0 |
| Rumble | 8 | 5 | 300 | 0.3 |
| Staccato | five 0.15 s pushes of 30 | 20 | 200 | 0.2 |
| Slow leak | 4 | 4 | 80 | 1 |

## Parameters and sources

Adult values (body size 1). "Not measured" means no measurement was found and the value comes
from the reasoning given.

| Parameter | Value | Source or derivation |
|---|---|---|
| Anal canal (high-pressure zone) length | 3.6 cm | 3.6 ± 0.1 cm (10th–90th 2.8–4.4), HR manometry, healthy women (Noelting et al. 2012, Am J Gastroenterol 107:1530) |
| Vibrating ring height | 1.0 cm | below the intersphincteric groove only the subcutaneous EAS surrounds the canal; anatomy-text figure |
| Upper canal diameter (gas-filled) | 1.05 cm | not measured; geometric mean of the closed mid canal (~5 mm) and the upper canal distended by a FLIP bag (20–24 mm) (Luft et al. 2012, Tech Coloproctol 16:451) |
| Closed lumen slit length (valve width) | 1.0 cm | slit across the ~1 cm mucosal core; the sub-mm gaps of a gas jet don't unfold the folds |
| Sphincter ring mean radius | 1.3 cm | mucosal core ~1 cm, IAS 2–3 mm, EAS 5–8 mm (typical endoanal ultrasound figures, not checked against one study) |
| Lip thickness (moving tissue) | 0.7 cm | anoderm and submucosa (~2 mm) + subcutaneous EAS (~5 mm) |
| Tissue density | 1.05 g/cm³ | soft tissue |
| Passive stiffness | 3·10⁵ dyn/cm³ | the engine's soft-tissue wall (Ishizaka, French & Flanagan 1975, cheek: skin, fat, thin muscle) |
| Active stiffness | γ·P_close/r, γ = 23.4 | muscle short-range stiffness K = γ·F/l_F (Cui, Perreault, Maas & Sandercock 2008, J Biomech 41:1945); a ring's hoop tension P_close·r |
| Lip damping ratio | 0.1 | not measured for anal tissue; Ishizaka & Flanagan 1972, vocal-fold mass |
| Valve kinetic losses | entrance 0.37, exit 1.0 | Ishizaka & Flanagan 1972 (as at the engine's glottis) |
| Rectal compliance | 11 mL/mmHg | barostat, normal anorectal function, 11.4 ± 5.8 (Colorectal Dis 2013, 15:715); linear through zero volume |
| Gas pocket shape | length/diameter 2.5 | rectal ampulla ~12 cm long, 4–6 cm across distended |
| Pocket floor | 2 mL | not measured: what stays trapped when the walls meet |
| Tissue viscosity (Rayleigh–Plesset) | 2 Pa·s | order of soft-tissue shear viscosity (1–3 Pa·s, shear-wave dispersion of liver and muscle); not a pelvic measurement |
| Cleft depth at the anus | 3.0 cm | 2.1 cm measured at the sacrococcygeal level (Akinci et al. 2009, Colorectal Dis 11:753), deeper caudally; at the anus not measured |
| Residual gap, pressed cheeks | 1 mm | not measured: skin folds and hair leave channels in the margin and the cleft; where the cheeks meet beside the verge they touch |
| Cleft opening when spread | right angle | the cheek surfaces at 90° |
| Push range and default | 0–80, 30 cmH₂O | rectal pressure during simulated evacuation 20 ± 3 mmHg (10th–90th 0.7–47 mmHg) (Noelting 2012) |
| Tone range and default | 2–150, 20 cmH₂O | resting 32–133 cmH₂O (median 67), squeeze 82–252 (Luft 2012); gas passes when it is relaxed below the push |
| Gas volume range | 10–300 mL | 476–1491 mL a day in 24-h collections on a bean-supplemented diet (Tomlin, Lowis & Read 1991, Gut 32:665), in some ten to twenty events |
| Push activation | attack 0.05 s, release 0.08 s | estimate from skeletal-muscle contraction times; not measured for straining |
| Turbulence | 3.2e-6 (300–3500 Hz); 1.364e-6 (350–2600 Hz) | the human voice's calibrated glottal aspiration and constriction sources, unchanged |
| Gas | air at body temperature | the engine's ρ and c; flatus (N₂, H₂, CO₂, CH₄) is not modelled separately |
| Anal margin (verge to the cheeks' strip) | 1.5 cm | not measured; at least two stable FDTD sections at the smallest body size |
| Cheeks' contact strip, along the flow | 1.0 cm | not measured; about the anal margin's width |
| Cheeks' moving layer | 5 cm | the ischioanal fossa's fat, a wedge about 5 cm deep (textbook figure, not an imaging series); over the gluteus, further out, subcutaneous fat is 1.6 ± 0.4 cm (men) and 2.5 ± 0.6 cm (women) on CT |
| Fat's Young's modulus | 22.5 kPa | 3 × the shear modulus 7.5 kPa of subcutaneous adipose tissue at 10 rad/s, 37 °C (Geerligs et al. 2008, Biorheology 45:677), incompressible |
| Cheeks' frequency and damping | 23 Hz, ζ 0.02 | the layer's first thickness mode c/4L; Kelvin–Voigt loss with the tissue viscosity above |
| Mucus traction–separation | δ₀ 0.1 mm, δf 0.267 mm, σ₀ ≤ 3 kPa | Bhattacharya & Siegmund 2014 (vocal-fold airway surface liquid; σ₀ bound from intestinal mucus, Mortazavi & Smart 1994) |
| Output gain | 5.5e-7 / 4 | the human voice's gain, 12 dB down so the loudest events stay under the limiter's knee |

## Measurements

All measurements were taken with `chimera render creature flatulence out/flatus hold=3` and with
sweeps of `chimera render creature flatulence out/sweep calls=none hold=4 <param>=<value>`, all at
48 kHz output (96 kHz internal). The WAVs are under `out/` (gitignored). Each sound was measured
over its span of 10 ms windows within 30 dB of its loudest:

- **duration:** that span;
- **level:** RMS in dBFS at the creature's gain (add 12 dB for the human voice's scale, where
  sustained male vowels measure −20 dBFS);
- **f0:** YIN median and 5th–95th percentiles over 40 ms frames, with the fraction of loud frames
  that are periodic (aperiodicity < 0.3);
- **centroid:** the spectral centroid.

These are the model's outputs, with the cheeks' valve and the mucus control (measured again
after both were added; the defaults are dry). I couldn't listen to them. "Validation against
recordings" below compares them with real flatus.

**Presets** (and the held voice at the defaults):

| | Duration | Level | f0 median (5–95 %) | Periodic | Centroid |
|---|---|---|---|---|---|
| Held, defaults (push 30, tone 20, gas 100, spread 0.2, dry) | 0.40 s | −21.0 dBFS | 239 Hz (178–317) | 46 % | 1386 Hz |
| Squeak | 0.17 s | −22.1 | 321 Hz (314–335) | 36 % | 1422 Hz |
| Rumble | 1.13 s | −26.4 | 165 Hz (128–182) | 95 % | 845 Hz |
| Staccato (five bursts, falling as the pocket empties) | 1.83 s | −30.0 | 210 Hz (174–240) | 38 % | 1329 Hz |
| Slow leak | 1.38 s | −36.4 | 159 Hz (153–170) | 95 % | 869 Hz |

**One control at a time** (the others at the defaults):

| Control | Values | Duration (s) | Level (dBFS) | f0 median (Hz) | Centroid (Hz) |
|---|---|---|---|---|---|
| Push (cmH₂O) | 15 / 20 / 30 / 45 / 60 / 80 | 0.79 / 0.56 / 0.40 / 0.19 / 0.14 / 0.11 | −30.1 / −26.2 / −21.0 / −15.6 / −13.2 / −11.2 | 186 / 205 / 239 / – / – / – | 995 / 1095 / 1386 / 1390 / 1421 / 1289 |
| Tone, push = tone + 15 | 5 / 10 / 20 / 40 / 60 | 0.30 / 0.30 / 0.28 / 0.17 / 0.12 | −21.4 / −20.2 / −18.9 / −14.6 / −12.3 | 179 / 203 / 305* / 311* / 368* | 1274 / 1334 / 1351 / 1648 / 1449 |
| Gas (mL) | 20 / 50 / 100 / 200 / 300 | 0.23 / 0.32 / 0.40 / 0.60 / 0.68 | −25.5 / −21.9 / −21.0 / −22.9 / −23.4 | 296 / 268 / 239 / 262 / 269 | 1690 / 1537 / 1386 / 1387 / 1389 |
| Cheek spread | 0 / 0.25 / 0.5 / 0.75 / 1 | 0.28 / 0.40 / 0.41 / 0.42 / 0.43 | −23.6 / −20.9 / −21.3 / −21.8 / −22.1 | 269 / 238 / 242 / 234 / 234 | 1250 / 1393 / 1327 / 1245 / 1214 |
| Body size | 0.6 / 0.8 / 1 / 1.25 | 0.25 / 0.33 / 0.40 / 0.49 | −23.9 / −21.5 / −21.0 / −19.9 | 349* / 336 / 239 / 201 | 2197 / 1816 / 1386 / 1118 |
| Mucus (kPa) | 0 / 0.5 / 1 / 1.5 / 2 | 0.40 / 0.44 / 0.43 / 0.42 / silent | −21.0 / −18.6 / −18.4 / −18.2 / – | 239 / 252* / 239* / 258* / – | 1386 / 1932 / 1941 / 1919 / – |

\* Under 25 % of frames are periodic: the oscillation is irregular, and the median means little.
– No periodic frames.

How the sound follows the physics:

- **Tone sets the pitch.** Its short-range stiffness raises the lip's natural frequency: 116, 150
  and 186 Hz at tones 5, 20 and 40 cmH₂O. The valve sounds above that (179, 203 and 305 Hz at
  tones 5, 10 and 20), on the outward-striking side, as brass players' lips do.
- **High tone becomes irregular.** With a push just above it, 46 % of frames are periodic at
  tones 5 and 10, under 12 % from tone 20 on.
- **Push sets the level and the duration.** More push opens the valve wider and empties the pocket
  faster: from push 15 to 80, 19 dB louder and 7 times shorter.
- **Push above about 45 cmH₂O breaks the oscillation**: no periodic frames, and the pocket's own
  Rayleigh–Plesset pulsation drives the valve hard.
- **At push 60 or more, the peaks reach the output limiter's soft knee.**
- **Gas sets the duration**, 0.23 s at 20 mL to 0.68 s at 300 mL.
- **Pressed cheeks are a second valve.** At spread 0 the gas has to part them: the event is
  shorter (0.28 against 0.43 s spread) and 1.5 dB quieter. Near threshold they sputter: pressed,
  tone 20, push 10, the 100 mL leave in puffs for 7.9 s, about 25 a second, f0 near 70 Hz and a
  centroid of 200 Hz, but at −85 dBFS RMS, some 60 dB below a normal event: a nearly silent
  sputter.
- **Body size scales the pitch about as 1/size** (349 Hz at 0.6, 201 Hz at 1.25), as the lip's
  mass and the ring's radius grow.
- **Mucus makes the openings late and irregular.** Already at 0.5 kPa the periodic fraction falls
  from 46 % to 12 %, and the sound is brighter (centroid 1386 → 1932 Hz) and 2.5 dB louder: each
  opening starts all at once. At 2 kPa and more the bridge holds the 22 cmH₂O by which the
  pocket exceeds the tone: the wet seal is continent, and nothing passes.
- **Events end when the gas is gone.** The pocket reaches its floor, its last gas expands out, and
  the pressure falls below the tone. A 100 mL event conserves gas: what crosses the valve is what
  left the pocket, within 8 mL (`tests/flatulence.rs`).
- **A tone above the push plus the recoil keeps the canal shut.** Tone 80 against push 30 is
  silent: continence.

**Sensitivity to the values that weren't measured.** These are held defaults with one constant
changed, through a temporary environment override that was not committed. They were measured
before the cheeks' valve was added and have not been repeated since:

| Constant (default) | Tried | f0 median (Hz) | Duration (s) |
|---|---|---|---|
| Lip damping (0.1) | 0.05 / 0.2 / 0.4 | 239 / 227 / 261 | 0.39 / 0.43 / 0.48 |
| Ring height (1.0 cm) | 0.6 / 1.6 | 240 / 242 | 0.38 / 0.43 |
| Upper canal diameter (1.05 cm) | 0.7 / 1.5 | 242 / 231 | 0.42 / 0.38 |
| Cleft depth (3 cm) | 2.0 / 4.5 | 235 / 237 | 0.40 / 0.40 |
| Lip thickness (0.7 cm) | 0.4 / 1.0 | 259 / 198 | 0.35 / 0.44 |
| Pocket floor (2 mL) | 1 / 5 | 236 / 232 | 0.41 / 0.36 |
| Tissue viscosity (2 Pa·s) | 0.6 / 20 | 236 / 228 | 0.40 / 0.43 |
| Slit length (1.0 cm) | 0.6 / 2.0 | 226 / 245 | 0.56 / 0.28 |
| Residual cleft gap (1 mm) | 0.5 / 2 mm | 238 / 233 | 0.40 / 0.41 |
| Passive stiffness (3·10⁵) | 1.5·10⁵ / 6·10⁵ | 215 / 263 | 0.36 / 0.51 |
| Short-range γ (23.4) | 12 / 40 | 212 / 260 | 0.35 / 0.48 |
| Rectal compliance (11 mL/mmHg) | 5 / 17 | 237 / 236 | 0.35 / 0.42 |

- **Pitch** stays within about ±15 % across these ranges. The lip's mass and stiffness move it
  most.
- **Duration** depends most on the slit length, the valve's width. It is the least certain
  geometry here.
- **Lip damping at 0.4** drops the level by 10 dB, and the oscillation becomes irregular.

## Validation against recordings

The model was fitted to real recordings: for each recorded event, the physical controls (push and
its timing, tone, gas, cheek spread, body size, mucus) that bring the model's render closest to
it. The tools are in `tools/flatus-fit/` (README there), every step is in
[flatulence-validation-log.md](flatulence-validation-log.md), and the recordings are listed, with
their licenses and the reasons for every exclusion, in `tools/flatus-fit/manifest.json`. They
are not in the repository.

**Data.** 31 recordings of human flatulence that their uploaders describe as real, from sources
reachable without login:

- 23 from Freesound (its public 128 kb/s MP3 previews): 20 CC0, 3 CC BY 3.0/4.0. 21 different
  uploaders. Two have natural room reverberation (a pub toilet, a studio), one was made into an
  upholstered couch, one is a 14 s series.
- 8 from Wikimedia Commons: CC BY-SA 3.0/4.0, CC BY 3.0 and one public-domain file. Among them a
  4 min 50 s compilation of "different kind of real farts" from the Finnish broadcaster Yle's 1986
  sound archive (also on Freesound, #425594), from which 12 events spread over its length were used.

Excluded, 37: anything made with the mouth, lips, tongue, an arm or the voice, whoopee cushions,
toy fart machines, synthesisers, files with added reverb or EQ and noise reduction, composites, a
duplicate, and files whose description doesn't say the sound is real (three Commons files; on
Freesound, files described only by tags, such as one uploader's dozens of "fart NN.mp3", were
not taken in the first place).

Each recording was cut into events (10 ms RMS above the noise floor + 15 dB and the loudest − 35 dB,
gaps under 0.2 s joined, events at least 15 dB above the floor and within 25 dB of the loudest),
at most 3 per recording (the loudest) and 12 from the compilation. Three events that were not
flatus by their spectrograms (a thump, a low thud, noise after the event) were dropped: 59 events.

**Features** (21 per event, none depending on the recording's level or gain):

- time: the duration (10 ms windows within 30 dB of the peak), attack (to within 6 dB of the
  peak), release (from the last window within 6 dB), sustain (fraction of the span within 10 dB of
  the peak), and bursts (peaks of a 5 ms envelope standing 6 dB above the dips around them, at
  least 30 ms apart, within 25 dB of the peak);
- pitch, on the periodic frames (YIN, 50 ms frames, 40–1000 Hz): median f0, its 10–90 % range,
  slope and frame-to-frame jitter, and the subharmonic-to-harmonic ratio (half harmonics against
  harmonics, after Sun 2002);
- periodicity: the fraction of loud frames that are periodic (aperiodicity < 0.3) and the median
  harmonics-to-noise ratio from the normalised autocorrelation at the period;
- spectrum, 62.5 Hz–8 kHz: the centroid, the tilt (dB per octave) and the seven octave-band levels
  relative to their total.

**Distance.** Each feature's difference is divided by its standard deviation over the 59 real
events (the pitch features' over the 18 mostly periodic ones), and the distance is the weighted
RMS of these, with weights summing to 3.0 for time, 3.2 for pitch, 1.8 for periodicity and 3.0
for the spectrum. The pitch terms fade out
when either sound is under 10–30 % periodic, where its pitch is undefined; the periodicity terms
carry that difference. The scales come from the real events only and are the same for every
version of the model.

**Close.** The yardstick is the spread among the real recordings: for each event, the distance to
the nearest event of another recording. Its median is 0.54 (quartiles 0.42 and 0.70). A fit is
close when it is within 0.54, as near to the recording as a typical other real recording is; a
recording is reached when one of its events is.

**Fit.** A library of 2000–5000 renders over a grid of the controls (push held until the gas is
gone), then scipy's Nelder–Mead from the two nearest library points over the controls and the push's
length, and, for events with 2–12 bursts, a train of pushes (fitted period and duty). About 150–200
renders per event. This is a local search: the same model searched from another library moves
single events by up to ±0.3 (log, experiment 5), so the counts below are what the search found,
a lower bound on what the model can do.

**Results.** Distances of the best fit, per recording (its best event); bold is close (≤ 0.54).
"Baseline" is the model before this validation (commit 788bdd0), "final" the model with the
cheeks' valve and the mucus control, each event's best of two searches:

| Recording | Baseline | Final | Best event: fitted controls | Largest misses (real − model, in spreads) |
|---|---|---|---|---|
| fs573745 (Authentic Fart) | 0.94 | 0.93 | 0: push 60, tone 26, gas 300, spread 0.00, size 1.01, mucus 0.04; push 0.90 s | log_attack -3.0, log_dur +1.3, log_release +1.9 |
| fs518699 (FART - 2) | 0.84 | **0.48** | 0: push 28, tone 42, gas 300, spread 0.00, size 1.23, mucus 1.37; push 0.56 s | b707 +1.9 |
| fs518701 (FART - 3) | 0.68 | 0.56 | 1: push 7, tone 2, gas 61, spread 1.00, size 0.79, mucus 3.00; push 0.54 s | periodic -1.1 |
| fs186025 (fart 2012-09-14.flac) | **0.51** | **0.47** | 0: push 65, tone 15, gas 300, spread 1.00, size 1.24, mucus 0.00; push 0.45 s | log_release +1.2 |
| fs121783 (FART.aif) | **0.52** | **0.50** | 0: push 15, tone 8, gas 27, spread 1.00, size 1.25, mucus 0.00; push 0.71 s | shr +1.2 |
| fs582215 (Fart_short_with_pressure_w…) | 0.70 | **0.54** | 0: push 41, tone 15, gas 300, spread 1.00, size 1.21, mucus 1.69; push 0.67 s | periodic +0.9, b177 -1.9, sustain +1.2 |
| fs135716 (Fart.wav) | 0.77 | **0.31** | 1: push 0, tone 17, gas 185, spread 0.00, size 1.25, mucus 0.32; push 0.52 s | – |
| fs185226 (funny fart) | 0.76 | 0.76 | 0: push 3, tone 4, gas 154, spread 0.00, size 1.20, mucus 3.00; 2 pushes of 0.25 s every 0.42 s | b354 +3.1, log_attack -1.6, f0 -0.8 |
| fs616782 (Fart.mp3) | 0.56 | **0.40** | 0: push 46, tone 4, gas 57, spread 0.00, size 1.20, mucus 0.00; push 0.20 s | log_release +1.3 |
| fs737546 (Flatulence) | **0.51** | **0.45** | 0: push 31, tone 9, gas 59, spread 0.00, size 1.23, mucus 1.55; push 0.25 s | centroid -0.7 |
| fs803697 (26 - fart) | 0.61 | **0.53** | 0: push 80, tone 2, gas 300, spread 0.00, size 0.61, mucus 0.26; push 0.27 s | log_dur +1.1, log_release +1.3 |
| fs874487 (Fart - Close recording) | 0.55 | 0.59 | 0: push 6, tone 2, gas 60, spread 0.00, size 1.20, mucus 3.00 | log_attack -1.6, periodic +0.9, log_release +1.3 |
| fs864457 (a fart) | **0.45** | **0.52** | 0: push 47, tone 7, gas 300, spread 1.00, size 0.85, mucus 1.36; push 0.37 s | periodic +0.8, b177 -1.7 |
| fs137008 (Homemade fart) | 0.80 | 0.61 | 0: push 34, tone 21, gas 300, spread 0.39, size 0.94, mucus 0.33; 4 pushes of 0.28 s every 0.49 s | b177 -2.5, log_attack -1.2, log_release +1.2 |
| fs854139 (Bathroom Fart) | 0.77 | 0.69 | 0: push 2, tone 14, gas 146, spread 0.50, size 1.20, mucus 1.72; push 0.30 s | b5657 -1.9, shr +1.2, b88 -1.7 |
| fs858198 (2026 Fart) | 0.64 | 0.74 | 0: push 3, tone 8, gas 60, spread 1.00, size 1.00, mucus 0.00; push 0.34 s | f0 +1.1, shr +1.7, b1414 -1.7 |
| fs456558 (Liquid Destiny Fart.wav) | 0.55 | **0.53** | 0: push 8, tone 6, gas 66, spread 0.01, size 1.13, mucus 0.07; 5 pushes of 0.14 s every 0.38 s | log_attack +2.0, centroid -0.9 |
| fs558387 (fart-series-fatulance-smal…) | **0.38** | **0.31** | 13: push 49, tone 15, gas 60, spread 0.00, size 1.20, mucus 3.00; push 0.22 s | – |
| fs400715 (Superbly Realistic Flatule…) | 0.57 | **0.45** | 0: push 12, tone 2, gas 23, spread 1.00, size 1.20, mucus 3.00; 5 pushes of 0.06 s every 0.11 s | log_attack -1.5, log_release +1.3 |
| fs402569 (Fart.mp3) | 0.62 | 0.55 | 0: push 60, tone 8, gas 152, spread 1.00, size 1.20, mucus 1.42; push 0.37 s | log_dur +0.9, sustain +1.2 |
| fs523930 (Harley Davidson) | 1.24 | **0.53** | 0: push 30, tone 40, gas 151, spread 0.00, size 1.20, mucus 0.00; push 6.88 s | b707 +1.9, bursts +1.0 |
| fs266450 (Chair Fart) | 0.63 | **0.46** | 0: push 66, tone 42, gas 62, spread 0.37, size 1.24, mucus 0.96; push 0.33 s | b177 -2.2 |
| fs608980 (Short but strong fart.ogg) | **0.50** | **0.46** | 1: push 42, tone 5, gas 300, spread 1.00, size 0.68, mucus 0.13; push 0.26 s | hnr -1.1 |
| wc_0c7c1082 (425594 ylearkisto farts-pi…) | **0.54** | **0.35** | 0: push 9, tone 3, gas 61, spread 0.00, size 1.20, mucus 1.29; push 0.59 s | – |
| wc_3dced03c (Flatulence.wav) | 0.55 | **0.52** | 0: push 16, tone 2, gas 59, spread 0.00, size 1.02, mucus 0.00; push 0.36 s | b2828 -1.6 |
| wc_adea4a8a (Flatulence de Clément.ogg) | 0.58 | **0.20** | 3: push 32, tone 40, gas 300, spread 0.01, size 1.20, mucus 1.42; 4 pushes of 0.07 s every 0.11 s | – |
| wc_e992050f (Flatus.wav) | **0.42** | **0.40** | 2: push 45, tone 13, gas 123, spread 1.00, size 1.20, mucus 1.30; push 0.34 s | – |
| wc_6fb2b93f (Frt755 1.ogg) | 1.09 | **0.36** | 0: push 80, tone 111, gas 300, spread 0.00, size 0.80, mucus 0.00; push 3.12 s | – |
| wc_359edc99 (Human fart.wav) | 0.88 | 0.86 | 0: push 47, tone 23, gas 11, spread 1.00, size 1.25, mucus 0.00; push 0.32 s | periodic +1.9, log_dur +1.4, b177 -1.8 |
| wc_709a446b (Menselijke wind drietraps.…) | **0.44** | **0.52** | 2: push 22, tone 15, gas 148, spread 0.02, size 0.80, mucus 1.71; push 0.95 s | centroid +0.9, hnr -0.8 |
| wc_69c63856 (Wet fart tummy rumbles.ogg) | 1.03 | **0.54** | 2: push 0, tone 4, gas 60, spread 0.00, size 1.25, mucus 0.00; push 1.80 s | b707 +1.8, log_release -1.3 |

| Model | Median distance | Events close (of 59) | Recordings reached (of 31) |
|---|---|---|---|
| Baseline | 0.67 | 12 | 9 |
| Final (one search / best of two) | 0.54 / 0.53 | 31 / 34 | 20 / 22 |

32 of the 59 events are now at least as close to the model as to their own nearest other
recording (baseline 18). Fits ending at the gas maximum fell from 29 to 16 and at the largest
body from 42 to 11: the model no longer strains at its ranges to make long, low events.

**Level.** The features ignore level, because the recordings are uncalibrated, and the fit uses
that freedom. Two of the reached recordings (fs135716, fs523930) are reached only by renders more
than 30 dB quieter than the creature's default event (the near-threshold sputter); with that as
a condition, 32 events and 20 recordings. Within a recording, where microphone and distance are
the same, the level differences between events are a test the fit never saw: the model's differ
from the recordings' by 12 dB (median over 89 pairs), and by more than 20 dB in 23.

**What still misses.** Nine recordings are not reached. By the features that miss:

- **Clean, steady tones**: Bathroom Fart (142 Hz, HNR 14 dB), Human fart.wav (132 Hz), funny
  fart (93 Hz) and 2026 Fart (254 Hz, HNR 17 dB; the final model fits it worse than the baseline,
  0.64 → 0.74). The model's tones are less steady, and its clean ones don't go below about
  160 Hz: at low tone the lip's stiffness is the passive tissue's (Ishizaka's 3·10⁵ dyn/cm³,
  measured on the face's cheek, not the anus), which puts its natural frequency at 107 Hz and the
  sound at about 1.5 times that. A softer or heavier vibrating lip (the perianal skin folds; the
  verge's passive stiffness was never measured) would go lower. Not changed: no measurement to
  set it from.
- **Abrupt onsets** (attack is the largest remaining miss: real events reach their peak within
  about 10 ms; the model's oscillation builds up over several cycles). The mucus bridge opens
  all at once, but the fits use it only partly. A valve that is held shut and then released, as
  a sphincter letting go, is not the single-mass LipReed's small-signal onset.
- **Long tails** (release and duration: Authentic Fart, Homemade fart). Several recordings were
  made in reverberant rooms (a pub toilet, a studio, bathrooms); the model has no room. And the
  flow is still fast (Limits).
- **Spectrum around 125–250 Hz and 500–1000 Hz**: the real events have less of the first and
  more of the second than the model's, consistently (mean misses −0.5 and +0.8 spreads). Phone
  and small microphones roll off the low band; it may be the recordings as much as the model.


## Limits

- **No peer-reviewed acoustic data.** The validation above uses freely licensed recordings of
  unknown microphones, distances, rooms and people; no peer-reviewed recording or spectral
  analysis of flatus was found.
- **The flow is too fast.** The model passes 100 mL in about 0.4 s at the default push (a mean of
  about 250 mL/s, peaks above 1 L/s). Against the recordings duration is still among the largest
  misses: the real events last longer for what they sound like (29 of 59 baseline fits ended at
  the largest gas volume, 16 now). No measurement of flatus flow rates was found. The likely
  causes are below (the upper canal, the valve's width).
- **The vibrating structure is one mass.** It has no mucosal wave, no asymmetric lips, and no
  localised opening of part of the ring. LipReed's contact floor (−0.5 mm) and maximum opening
  (8 mm) are its own.
- **The pocket is a sphere in liquid.** Rayleigh–Plesset treats the rectal pocket's surroundings
  as an unbounded liquid. The pelvis is bounded by bone (the sacrum) and by muscle, which would
  change the effective mass and damping. The pocket's acoustic shape is a 2.5:1 tube, while its
  wall dynamics use the equal-volume sphere.
- **The gas is the engine's air.** Flatus (mostly N₂, H₂, CO₂, CH₄) can be lighter than air when
  rich in H₂, which raises the speed of sound and the cavity resonances.
- **The rectal pressure–volume curve is linear through zero.** The flat start at low volumes
  (minimal distending pressure) is left out. With 300 mL the recoil alone (36 cmH₂O) exceeds a
  relaxed tone, and gas leaks without a push. That is physical in kind, but the threshold depends
  on this simplification.
- **The upper canal is always open** at its estimated diameter. Its opening and closing with the
  rectoanal reflex are not simulated, and neither is its collapse: a soft tube squeezed from
  outside limits the flow through it and can flutter (a Starling resistor), which would slow the
  event and could make the slow sputter below. Not tried.
- **The slow sputter is too quiet.** Five events (four recordings) are trains of distinct puffs at
  14–19 a second lasting 1.3–4.2 s. Pressed cheeks near threshold make such trains, but 30–70 dB
  below the model's ordinary events, while in the recordings the sputter is about as loud as the
  same person's other events. The features are level-free, so the fit doesn't see this (below).
- **The cheeks' valve is one mass on one strip.** Its strip length, the margin's length and the
  fat's depth beside the verge are estimates (above); pressed cheeks touch without a preload,
  because no contact pressure between the cheeks was found, and seated contact with a surface
  (which would press them and couple them to the seat) is not modelled. The cleft
  beyond the strip is a wedge-shaped slot with linear soft walls (Ishizaka's cheek tissue).
- **The mucus seal is all or nothing.** A bridge fills the whole slit until it breaks; partial
  bridges and gas blowing through the film are not modelled, so a strong bridge (2 kPa and up at
  the defaults) holds the gas indefinitely.
- **Refilling between events is skipped.** A new event starts full: at a call's `!fill`, or at a
  new push after rest in the held voice.
- **Cheek spread changes no flow.** The network's moving-wall volume source is off for this
  creature, because the pocket's wall motion enters as flow. Cheek spread and body size therefore
  change the cleft and canal without pushing air (they change slowly).
- **The wasm build is not checked bit for bit.** Native/wasm bit-identity is tested for the voice
  scenarios only, not for this creature. The e2e test plays it through the worklet.
