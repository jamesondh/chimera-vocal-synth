# Flatulence validation log

Fitting the flatulence creature (docs/flatulence.md) to real recordings, and the model changes
that followed, one at a time, newest last. The tools are in `tools/flatus-fit/` (README there);
the recordings are listed in `tools/flatus-fit/manifest.json` and are not in the repository.

Final result: median distance 0.67 → 0.53, events close 12 → 34 of 59, recordings reached
9 → 22 of 31 (experiment 6, with its caveat on level). Every number below comes from the same 59
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

## What still misses (final model)

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
