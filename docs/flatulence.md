# Flatulence

The Bestiary's `flatulence` creature: the sound of gas passing the anus, from the anatomy and
physics of the anorectum. The brief was that every modelling choice is grounded in physics and
anatomy, nothing is added because it sounds better, and where data are missing the choice comes
from physical reasoning, never from listening. The code is
`crates/chimera/src/creatures/flatus.rs`; each constant there carries its source.

```
 abdominal push ─┐
  + rectal       ▼  tissue around the pocket (Rayleigh–Plesset)
  recoil     rectal gas pocket ── upper anal canal ── ▷ anal verge ◁ ── gluteal cleft ── air
             (FDTD tube: the gas   (opened by the     (subcutaneous      (soft-walled slot,
              law is the network)   gas, RAIR)          EAS ring:          radiating into
                                                        LipReed)           a baffle)
```

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

**Turbulence.** The engine's Reynolds-number noise sources, at their calibrated gains:

- At the valve's jet, the human voice's glottal aspiration source (gain 3.2e-6, band 300–3500 Hz).
  The mechanism is the same: a jet from a slit valve into a duct.
- In the cleft when it is narrow, the human narrowest-constriction source (gain 1.364e-6, band
  350–2600 Hz). It uses the human source's own area fade.

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
| Residual gap, pressed cheeks | 1 mm | not measured: skin and hair don't seal |
| Cleft opening when spread | right angle | the cheek surfaces at 90° |
| Push range and default | 0–80, 30 cmH₂O | rectal pressure during simulated evacuation 20 ± 3 mmHg (10th–90th 0.7–47 mmHg) (Noelting 2012) |
| Tone range and default | 2–150, 20 cmH₂O | resting 32–133 cmH₂O (median 67), squeeze 82–252 (Luft 2012); gas passes when it is relaxed below the push |
| Gas volume range | 10–300 mL | 476–1491 mL a day in 24-h collections on a bean-supplemented diet (Tomlin, Lowis & Read 1991, Gut 32:665), in some ten to twenty events |
| Push activation | attack 0.05 s, release 0.08 s | estimate from skeletal-muscle contraction times; not measured for straining |
| Turbulence | 3.2e-6 (300–3500 Hz); 1.364e-6 (350–2600 Hz) | the human voice's calibrated glottal aspiration and constriction sources, unchanged |
| Gas | air at body temperature | the engine's ρ and c; flatus (N₂, H₂, CO₂, CH₄) is not modelled separately |
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

These are the model's outputs. I couldn't listen to them, and no peer-reviewed acoustic
measurement of flatus was found to compare them with.

**Presets** (and the held voice at the defaults):

| | Duration | Level | f0 median (5–95 %) | Periodic | Centroid |
|---|---|---|---|---|---|
| Held, defaults (push 30, tone 20, gas 100, spread 0.2) | 0.40 s | −22.7 dBFS | 235 Hz (180–316) | 84 % | 915 Hz |
| Squeak | 0.17 s | −23.1 | 316 Hz (311–333) | 43 % | 1557 Hz |
| Rumble | 1.07 s | −24.9 | 164 Hz (133–173) | 100 % | 478 Hz |
| Staccato (five bursts, falling as the pocket empties) | 1.82 s | −30.6 | 199 Hz (154–264) | 57 % | 844 Hz |
| Slow leak | 1.40 s | −37.3 | 159 Hz (152–170) | 98 % | 684 Hz |

**One control at a time** (the others at the defaults):

| Control | Values | Duration (s) | Level (dBFS) | f0 median (Hz) | Centroid (Hz) |
|---|---|---|---|---|---|
| Push (cmH₂O) | 15 / 20 / 30 / 45 / 60 / 80 | 0.74 / 0.57 / 0.40 / 0.19 / 0.14 / 0.11 | −30.9 / −27.5 / −22.7 / −16.9 / −14.2 / −11.7 | 189 / 211 / 235 / 237 / 190* / 194* | 630 / 712 / 915 / 970 / 965 / 807 |
| Tone, push = tone + 15 | 5 / 10 / 20 / 40 / 60 | 0.30 / 0.31 / 0.29 / 0.27 / 0.25 | −22.6 / −21.7 / −20.2 / −19.0 / −18.4 | 172 / 195 / 237 / 305 / 247* | 748 / 802 / 997 / 1224 / 1059 |
| Gas (mL) | 20 / 50 / 100 / 200 / 300 | 0.24 / 0.33 / 0.40 / 0.55 / 0.64 | −29.2 / −24.4 / −22.7 / −22.6 / −23.9 | 266 / 239 / 235 / 234 / 237 | 1241 / 1013 / 915 / 872 / 978 |
| Cheek spread | 0 / 0.25 / 0.5 / 0.75 / 1 | 0.35 / 0.41 / 0.42 / 0.42 / 0.43 | −21.1 / −22.9 / −23.3 / −23.4 / −23.4 | 238 / 235 / 233 / 233 / 240 | 1106 / 905 / 926 / 1026 / 1295 |
| Body size | 0.6 / 0.8 / 1 / 1.25 | 0.24 / 0.33 / 0.40 / 0.50 | −25.5 / −23.9 / −22.7 / −21.4 | 345 / 279 / 235 / 197 | 1295 / 1060 / 915 / 873 |

\* Under 25 % of frames are periodic: the oscillation is irregular, and the median means little.

How the sound follows the physics:

- **Tone sets the pitch.** Its short-range stiffness raises the lip's natural frequency: 116, 150
  and 186 Hz at tones 5, 20 and 40 cmH₂O. The valve sounds about 1.5–1.6 times above that (172,
  237 and 305 Hz), on the outward-striking side, as brass players' lips do.
- **High tone becomes irregular.** At tone 60 or more with a push just above it, under a quarter
  of the frames are periodic.
- **Push sets the level and the duration.** More push opens the valve wider and empties the pocket
  faster: from push 15 to 80, 19 dB louder and 7 times shorter.
- **Push above about 45 cmH₂O breaks the oscillation.** The periodic fraction falls to 9–44 %, and
  the pocket's own Rayleigh–Plesset pulsation drives the valve hard.
- **At push 60 or more, the peaks reach the output limiter's soft knee.**
- **Gas sets the duration**, 0.24 s at 20 mL to 0.64 s at 300 mL, and lowers the pitch a little.
  The pocket's resonance falls as it grows.
- **Cheek spread changes the spectrum, not the pitch.** Pressed cheeks (spread 0) make the cleft a
  1 mm slot: the centroid rises and the narrowest-constriction noise turns on. Wide spread enlarges
  the outlet, whose radiation load favours high frequencies, and raises the centroid again. In between, the centroid is flattest.
- **Body size scales the pitch about as 1/size** (345 Hz at 0.6, 197 Hz at 1.25), as the lip's
  mass and the ring's radius grow.
- **Events end when the gas is gone.** The pocket reaches its floor, its last gas expands out, and
  the pressure falls below the tone. A 100 mL event conserves gas: what crosses the valve is what
  left the pocket, within 8 mL (`tests/flatulence.rs`).
- **A tone above the push plus the recoil keeps the canal shut.** Tone 80 against push 30 is
  silent: continence.

**Sensitivity to the values that weren't measured.** These are held defaults with one constant
changed, through a temporary environment override that was not committed:

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

## Limits

- **No acoustic data to validate against.** No peer-reviewed recording or spectral analysis of
  flatus was found. Informal sources (a blog's source-filter estimate of 100–200 Hz, a sound
  designer's 100–1000 Hz) are not a test. The model's 160–350 Hz falls inside them, but it was
  not fitted to them.
- **Flow and duration are unvalidated.** The model passes 100 mL in about 0.4 s at the default
  push: a mean of about 250 mL/s, with peaks above 1 L/s in the trace. No measurement of
  flatus flow rates was found.
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
  rectoanal reflex are not simulated.
- **The cleft is a wedge-shaped slot.** It has a fixed width across the verge and linear soft
  walls (Ishizaka's cheek tissue). The cheeks' own flapping, a second valve when they are pressed
  and the gap is small, is not modelled: the network's walls are linear and heavily damped.
- **Refilling between events is skipped.** A new event starts full: at a call's `!fill`, or at a
  new push after rest in the held voice.
- **Cheek spread changes no flow.** The network's moving-wall volume source is off for this
  creature, because the pocket's wall motion enters as flow. Cheek spread and body size therefore
  change the cleft and canal without pushing air (they change slowly).
- **The wasm build is not checked bit for bit.** Native/wasm bit-identity is tested for the voice
  scenarios only, not for this creature. The e2e test plays it through the worklet.
