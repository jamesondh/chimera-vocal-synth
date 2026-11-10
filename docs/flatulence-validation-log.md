# Flatulence validation log

Fitting the flatulence creature (docs/flatulence.md) to real recordings, and the model changes
that followed, one at a time, newest last. The tools are in `tools/flatus-fit/` (README there);
the recordings are listed in `tools/flatus-fit/manifest.json` and are not in the repository.

Round 1 (experiments 1–6): median distance 0.67 → 0.53, events close 12 → 34 of 59, recordings
reached 9 → 22 of 31 (with experiment 6's caveat on level). Round 2 (experiments 7–9, the two
sphincters): median 0.42, 47 of 59 events close, 25 of 31 recordings reached. Every number below comes from the same 59
events of 31 recordings, the same features and the
same feature scales (the real events' spread, stored with each fit as `scale`), so the runs
compare directly. A fit is **close** when its distance is at most 0.54, the median over the real
events of the distance to the nearest event of another recording. A recording is **reached**
when its best-fitting event is close.

The fit is a local search (a library of 2000–5000 renders, then Nelder–Mead from the two best
library points and, for events with 2–12 bursts, from a push train). Two searches of the same
model with different libraries moved single events by up to 0.3 and the counts by about 3 events
(experiment 5): read the counts and medians, not single events. Experiments are compared on the
same kind of search; the final numbers take each event's best of two searches.

## Baseline (commit 788bdd0)

- Median distance 0.67. Close: 12 of 59 events; 9 of 31 recordings reached.
- Largest misses (mean weighted z², and the mean signed miss real − model in feature spreads):
  bursts 0.60 (+0.38), log duration 0.57 (+0.50), log attack 0.56 (−0.82), periodic fraction
  0.56 (−0.19), log release 0.48 (+0.63), HNR 0.33 (−0.46), centroid 0.30, tilt 0.29.
- Read: the real events are longer, start more abruptly, decay more slowly, are less periodic
  and noisier, and break into more bursts than the model's. The model's events are clean
  brass-like tones of 0.1–0.5 s. 29 of 59 fits sit at the gas maximum (300 mL), 42 at the
  largest body: the fit pushes toward longer and lower events than the model can make.

## 1. Sphincter release onset (fit protocol, not kept)

- Hypothesis: real onsets are abrupt (median attack ≈ 10 ms, model ≈ 30 ms) because held gas
  is let go: the abdomen pressurises the pocket against a closed sphincter, which then relaxes.
- Change: a render option (`release=<t>` in `examples/flatus_render.rs`) holding the tone at
  its maximum until t, then relaxing it to the set tone.
- Result (single renders, not a full fit): the attack got longer, 0.03 → 0.08 s at the defaults,
  0.25 s at push 60. The oscillation still builds up over several cycles once the valve opens;
  how the pressure arrives doesn't matter. Not used in the fits. The abrupt onsets need a valve
  that opens all at once, not a different control course.

## 2. Mucus adhesion at the verge (not kept)

- Hypothesis: a wet canal sticks shut (mucus bridges between the faces) and opens in bursts.
- Change: Bhattacharya & Siegmund's (2014) traction–separation law (cohesive strength up to
  3 kPa, intestinal mucus; δ₀ 0.1 mm, δf 0.267 mm) pulling the faces of the LipReed together
  after contact; a control "mucus" 0–3 kPa.
- Result (sweep: mucus 0/1.5/3 × tone 2/8/20 × push 5–40): no bursts. Up to 2 kPa nothing
  changes (the oscillation's gap exceeds δf every cycle, as Bhattacharya & Siegmund found: no
  frequency change). At 3 kPa the event stops after 0.05–0.1 s: the faces part a little, inside
  the bridge's elastic range, and the valve settles into a silent leak. Reverted. (Experiment 5
  tries the bridge as a seal.)

## 3. The cheeks as a second valve: gluteal fat (kept, superseded by 4)

- Hypothesis: the docs' limit "the cheeks' own flapping … is not modelled". Gas leaving the
  verge must part the cheeks, whose surfaces meet (pressed) or nearly meet beside it: a second
  pressure-opened soft-tissue valve, like the lips in a raspberry, which should make the sound
  rougher and less periodic.
- Change (`crates/chimera/src/creatures/flatus.rs`): the verge now opens into a 1.5 cm
  soft-walled slot (the anal margin), then a second LipReed valve (the cheeks' 1 cm contact
  strip; slit across the verge's width; rest gap the wedge's mean gap over the strip, so pressed
  cheeks leave the residual 1 mm and spread cheeks are wide open), then the cleft as before. The
  cheek is the first thickness mode of a fat layer of thickness L over a rigid base: f = c/4L,
  c = √(E/ρ), as a single mass with stiffness E/L and mass 4ρL/π² per area; damping from the
  model's tissue viscosity in extension, ζ = ω·3μ/2E. E = 3 × 7.5 kPa (shear modulus of
  subcutaneous fat, Geerligs et al. 2008, incompressible); L = 2 cm (gluteal subcutaneous fat on
  CT, 1.6 cm men, 2.5 cm women). f = 58 Hz, ζ = 0.05.
- Result: median 0.67 → 0.63; close 12 → 19 events; recordings reached 9 → 14. Periodic
  fraction and HNR misses fell (0.56 → 0.48, 0.33 → 0.23); the duration miss grew (0.57 → 0.78):
  the second valve passes the gas no slower.

## 4. The cheeks' layer is the ischioanal fat (kept)

- Hypothesis: the tissue beside the anal verge, where experiment 3's strip is, is not gluteal
  fat over muscle but the perianal skin over the ischioanal fossa, a fat-filled wedge about 5 cm
  deep (textbook figure: 5 cm vertical, 5 cm anteroposterior, 2.5 cm across). The anatomy was
  wrong in experiment 3; I checked it because experiment 3 still made no slow bursts, so the
  choice was prompted by the fit, but the value is the anatomy's, not a fitted one.
- Change: L = 5 cm. f = 23 Hz, ζ = 0.02.
- Result: median 0.63 → 0.58; close 19 → 25 events; recordings reached 14 → 18. Misses: bursts
  0.70 → 0.52, attack 0.57 → 0.37, periodic 0.48 → 0.33, release 0.44 → 0.31; duration 0.78 →
  0.62. 34 of 59 fits still at 300 mL.

## 5. Mucus as a seal (kept)

- Hypothesis: experiment 2's bridge, if it fills the gap while it holds (a liquid-filled gap passes
  no gas), makes each opening start late and all at once, and the longest rough events, the worst
  fits, need that.
- Change: as experiment 2, plus the valve passes nothing while the bridge holds. Control "mucus",
  0–3 kPa, default 0 (dry; the defaults and presets render as before).
- Result: a single search (library with mucus 0/1.5/3) 25 → 28 events close, recordings reached
  18 → 17, median 0.58 → 0.56. With mucus = 0 the model renders bit for bit as experiment 4, so
  experiment 4's fits are fits of this model too; each event's better of the two: 30 of 59 close,
  19 of 31 reached, median 0.54. Misses: duration 0.62 → 0.35, bursts 0.52 → 0.42. The fits used
  1.5–3 kPa on the long rough events (event wc_6fb2b93f.0, Frt755: 1.06 → 0.83; fs402569.1: 0.84 → 0.53).
- Search noise: the same events fitted by the two searches of the same model differ by up to 0.3
  (event wc_0c7c1082.43: 0.18 and 0.48). Single events are not to be read; counts within about
  ±3 events are noise.

## Fix: the cheeks started each render shut

- Writing the test for experiments 3–4, the cheeks' gap ranged below its rest value only at the
  start: LipReed starts at gap 0, so the cheeks were released from shut to their 1 mm rest gap
  and rang at 23 Hz (ζ 0.02) through the event. A spring released from where it never was is
  not physics. `LipReed::settle` now starts them at rest.
- Result, the same single search as experiment 5: 28 → 21 events close, 17 → 15 recordings
  reached, median 0.56 → 0.58. Misses: periodic fraction 0.41 → 0.57, bursts 0.42 → 0.55,
  duration 0.35 → 0.58. Part of experiments 3–5's gain was this artifact; the cheeks at rest are
  pushed open and barely move.

## 6. Pressed cheeks touch

- Hypothesis: pressed means in contact. The model kept a 1 mm gap between pressed cheeks
  ("skin and hair don't seal", not measured), so the gas met an open slit and only widened it.
  With the cheeks touching, the gas has to part them, and they fall shut again behind each puff:
  the raspberry.
- Change: the cheeks' rest gap is the wedge's opening over their strip, without the 1 mm (0 when
  pressed). The 1 mm stays as the margin slot's gap (hair and skin channels around the verge), so
  the margin still holds gas.
- First look (single renders): pressed cheeks, tone 20, push 10, 100 mL: a 7.7 s train of about
  25 puffs a second, f0 72 Hz, centroid 220 Hz, 3 % periodic, the slow sputter no earlier version
  made. At push 30 it plays as before.
- Result, same single search (library with mucus 0/1.5/3): close 21 → 31 of 59, recordings
  reached 15 → 20 of 31, median 0.58 → 0.54; events at least as close as their own nearest other
  recording 20 → 32. Misses: periodic fraction 0.57 → 0.28, bursts 0.55 → under 0.2, duration
  0.58 → 0.32, attack 0.51 → 0.37. The sputtering "Harley Davidson" (fs523930) 1.22 → 0.53;
  Frt755 (wc_6fb2b93f) 1.06 → 0.36.
- Kept. Golden re-blessed; tests for the flapping and the sputter.
- A second search (library without mucus, mucus still free in Nelder–Mead): 27 close, 17
  recordings. Each event's better of the two (`merge.py`), the final numbers: **median 0.53;
  34 of 59 events close; 22 of 31 recordings reached; 32 events at least as close as their own
  nearest other recording.** Fits at the gas maximum 29 (baseline) → 16, at the largest body
  42 → 11.
- Caveat, level: the sputter regime is quiet. The best fits of fs523930 and fs135716.1 are 55 and
  36 dB below the default event's peak; requiring the fit to be within 30 dB of it leaves 32
  events and 20 recordings. Within one recording (same microphone and distance) the model's level
  differences between events miss the recordings' by 12 dB (median of 89 pairs), more than 20 dB
  in 23. The distance is level-free by design (the recordings are uncalibrated), so the fit never
  saw this.

## What still missed after round 1 (experiment 6)

Nine recordings are not reached: Authentic Fart (0.93), funny fart (0.76), 2026 Fart (0.74),
Bathroom Fart (0.69), Human fart.wav (0.86), Homemade fart (0.61), Close recording (0.59), FART - 3
(0.56), Fart.mp3 by TeddyFerguson (0.55). Largest mean misses over all events: attack 0.41 (real
faster), duration 0.31 and release 0.27 (real longer), the 500–1000 Hz band 0.23 (real more),
centroid 0.22 (real lower), 125–250 Hz 0.18 (real less).

- Clean, steady low tones (93–142 Hz, HNR up to 14 dB): the model's clean tones stay above about
  160 Hz because the lip's passive stiffness, borrowed from the face's cheek (Ishizaka et al.
  1975), keeps its natural frequency above 107 Hz. Lead: a measured passive stiffness of the
  anal verge, or the perianal skin folds as a heavier vibrator. Not changed without a measurement.
- Abrupt onsets: the LipReed's oscillation grows over several cycles (experiment 1 showed the
  control course doesn't change that). Lead: a closure that is held and then released (a
  sphincter letting go), e.g. the mucus bridge with a longer rupture length, if a source gives one.
- Long tails: some rooms reverberate (the pub toilet, the studio, bathrooms; the model has no
  room), and the flow is still too fast (16 fits at the gas maximum). Lead: the upper canal as a
  collapsible tube (flow limitation, Starling-resistor flutter; Bertram & Grotberg's reviews of
  flow in collapsible tubes) instead of an open tube.
- Level: see experiment 6's caveat. A fit that also matches the level differences between events
  of one recording would need a joint fit per recording; not done.

## 7. The lip's stiffness and mass from the anal canal, not the face (kept)

Second round (branch `flatus-tone`). The user's point: the sphincter behaves very differently
relaxed and contracted. Before, the lip's stiffness was a borrowed passive stiffness (Ishizaka's
facial cheek, 3·10⁵ dyn/cm³) plus the muscle's short-range stiffness γ·P/R; its mass 0.7 cm of
tissue moving as a plate.

- Physical basis, three corrections, each derived (code comments in `flatus.rs`):
  - **Passive stiffness from the anal canal's measured compliance.** Barostat distension with the
    balloon's volume checked on MRI in 33 healthy women: past the opening pressure, 0.30 (SD
    0.18) mL per mmHg over the 3.2 cm canal (Sharma, Feuerhak, Corner, Manduca & Bharucha 2021,
    Neurogastroenterol Motil 33:e13972), a cross-sectional compliance C_A = 0.094 cm²/mmHg. For a
    slit passing A = ℓ·h, k = 2ℓ/C_A = 2.8·10⁴ dyn/cm³ (SD range 1.8–7.1·10⁴): a tenth of the
    face's. Caveats: measured awake (the canal relaxes as it is distended, and the toned muscle's
    length–tension adds stiffness), quasi-static, and over 0.5–2.2 cm of diameter; Lestar et al.
    (1989) found passive elements bearing load only beyond 1 cm of distension (anaesthesia with
    curarisation and a maximal reflex), so at sub-mm gaps the passive part is if anything smaller.
    I found no anal FLIP or barostat study under anaesthesia with a pressure–area curve. The
    EndoFLIP studies I read (Luft 2012, Fynne 2013, Sørensen 2014) are of the awake canal and
    report segment diameters and distensibility, not a compliance in these units; the barostat–MRI
    study reports one and validates its volumes against images, so it was used.
  - **Active stiffness for the slit the flow actually passes.** The flow passes a 1 cm slit, but
    the active stiffness treated the lumen as a circle of the ring's radius. With incompressible
    tissue between the slit and the ring of radius R, the slit's opening dA widens the ring by
    dA/2πR; with short-range stiffness dT/T = γ·dR/R and Laplace's P = T/R,
    k = (γ − 1)·P·ℓ/(πR²): 4.2·P at R = 1.3 cm instead of 18·P.
  - **The moving mass from incompressibility.** The opened area must be pushed out through every
    contour around the slit (a two-dimensional source, velocity Ȧ/2πr) out to the ring's height,
    where the flow becomes three-dimensional: m = ρℓ·ln(4H/ℓ)/π = 0.46 g/cm² (was 0.74). It does
    not depend on tone: the tissue has to move either way. So relaxed and contracted differ in
    stiffness and preload, not in mass.
- Result (one search, library with mucus 0/1.5/3 and size 0.8/1.2, against the same kind of
  search of experiment 6): median 0.54 → 0.48; close 31 → 37 of 59; recordings reached 20 → 21.
  Misses: duration 0.32 → 0.39 (real longer), attack 0.37 → 0.28. The softer lip opens wider and
  empties the pocket faster (the default event 0.40 → 0.16 s); fits at the gas maximum 17 → 28.
- Kept (it is the measurement the model lacked), and carried into experiment 8 (committed with
  it, f4d2f85; this intermediate state is not a commit). The low-tone recordings: Bathroom Fart
  0.69 → 0.58, Human fart.wav 0.89 → 0.77, funny fart 0.76 → 0.74, 2026 Fart 0.76 → 0.80.

## 8. Two sphincters: the internal one's zone with the rectoanal reflex, the external one's ring with activation dynamics (kept)

- Physical basis (sources in `flatus.rs` and docs/flatulence.md):
  - The canal closes in two places. The internal sphincter's zone (above the intersphincteric
    groove): the IAS's tone, the anal cushions and the deep external sphincter. The distal ring:
    the subcutaneous external sphincter alone. Each is a LipReed with experiment 7's mass and
    stiffness; each muscle ring adds its own short-range stiffness (IAS at R = 0.63 cm).
  - The resting pressure split (Lestar, Penninckx & Kerremans 1989): EAS 30 %, IAS 55 %, cushions
    15 %. IAS tone a control (default 37 cmH₂O, 55 % of the median 67 of Luft 2012), cushions
    10 cmH₂O. The EAS's share at the IAS's level, 17/32, from Cheeney et al.'s (2012) residual
    pressures during a full reflex (28 mmHg at 1.6 cm less the cushions' 11, against 32 mmHg at
    0.9 cm).
  - The rectoanal reflex (Cheeney 2012, high-definition manometry): the nadir's depth
    1 − exp(−V/27 mL), through 43 % at 15 mL and full from 71 mL; the plateau after it rising
    linearly from none at 10 mL to the nadir at 115 mL. The latency (5.1 s; Sangwan 1995) is taken
    as passed: each event starts at the nadir (the gas has been sampled); the IAS recovers towards
    the remaining volume's plateau at 1/9.5 of its inhibition per second (2.98 mmHg/s against a
    28 mmHg relaxation in controls; Netinho 2005).
  - The external sphincter's command is a control ("eas", replacing "tone"), passed through
    first-order activation dynamics, 10 ms up and 40 ms down (Zajac 1989; Winters 1990). Not
    measured for the EAS, which is ~80 % slow fibres (Konsten 1993).
  - The IAS zone's load is the gas's gauge pressure above it (a radial closure: the gas below it
    presses it open, not shut); the distal ring keeps the pressure difference across it, as
    before. The 0.75 cm junction between them is one FDTD section, not anatomy.
- Result (two searches, each with the course stages of experiment 9; here the constant-command
  stage): one search median 0.44, close 40 of 59, recordings 23 of 31 (experiment 7: 0.48, 37,
  21); each event's best of the two searches: median 0.43, close 44, recordings 24. Misses (one
  search): duration 0.39 → 0.25, attack 0.28 → 0.19, periodic 0.25 → 0.24; centroid 0.24 (real
  lower). Fits at the gas maximum 28 → 23 (best of two: 13). Bathroom Fart reached (0.54), funny
  fart 0.60, 2026 Fart 0.65, Human fart.wav 0.89 (one search). 32 of 59 best fits relax the
  external sphincter to ≤ 3 cmH₂O.
- Level (best of two, with courses): 45 events and 25 recordings close and within 30 dB of the
  default event's peak (round 1: 32 and 20); within-recording level differences off by a median
  12 dB (round 1: 12), more than 20 dB in 17 of 89 pairs (round 1: 23).
- Kept (commit f4d2f85).

## 9. The external sphincter's command as a course during the event (fit protocol; kept, adds little)

- Basis: the voluntary command can change during an event (clench to cut off, relax to let go,
  squeeze rhythmically), passed through the activation dynamics, so nothing changes faster than
  muscle can. Nothing shapes the sound itself.
- Change (`fit.py courses=1`): from each event's best constant fit, a ramp of the command (start
  and end value, start time, duration; 40 renders), then a modulation around it (0.5–8 Hz, depth
  up to 60 cmH₂O; 25 renders). The 8 Hz bound is the upper end of voluntary rhythmic movement
  (an estimate; not measured for the EAS); the activation dynamics attenuate the fast end anyway
  (−3 dB near 4 Hz for the 40 ms deactivation).
- Result (one search): constant → with courses: median 0.44 → 0.43; close 40 → 42; recordings
  23 → 24. The ramp improved 6 events by more than 0.03, the modulation 1 (Yle compilation
  event 0: 0.29 → 0.22 with ±15 cmH₂O at 4.2 Hz). Best of two searches: constant 44 close, 24
  recordings; up to a ramp 46, 24; with modulation 47, 25, median 0.43 → 0.42. The one recording
  the course reaches is the Harley Davidson sputter (fs523930: 0.69 constant, 0.48 with the
  command quivering ±24 cmH₂O at 3 Hz). Otherwise within the search noise (±3 events), with up
  to six more parameters per event: the level-free features barely ask for a changing
  contraction. Kept as an option of the fit; the model's controls are physical either way.

## What still misses (two sphincters)

Final: **median 0.42; 47 of 59 events close; 25 of 31 recordings reached; 45 events at least as
close as their own nearest other recording** (round 1: 0.53, 34, 22, 32). Not reached: Human
fart.wav (0.89), Authentic Fart (0.71), FART.aif (0.70; 0.50 in round 1), 2026 Fart (0.64),
funny fart (0.58), Wet fart tummy rumbles (0.55; 0.54 in round 1). Largest mean misses:
periodic fraction 0.24 (real more periodic), duration 0.21 (real longer), centroid 0.18 (real
lower), 2.8 kHz band 0.17 (real less).

- The near-threshold regime that holds clean tones is narrow; away from it the soft closures
  gape and the flow turns aperiodic. Human fart.wav (63 % periodic) still fits only aperiodic
  renders.
- Clean tones above about 200 Hz (2026 Fart, 254 Hz, HNR 17): the contracted closures play
  160–190 Hz cleanly; higher, irregularly.
- Leads, not tried: the external sphincter's slower (type I) activation constants; the closure as
  a collapsible tube (pressure along it, Bernoulli suction in the gap) instead of two lumped
  valves; a room for the reverberant recordings.
