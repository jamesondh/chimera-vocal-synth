# Flatulence

The Bestiary's `flatulence` creature: the sound of gas passing the anus, from the anatomy and
physics of the anorectum. The brief was that every modelling choice is grounded in physics and
anatomy, nothing is added because it sounds better, and where data are missing the choice comes
from physical reasoning, never from listening. The code is
`crates/chimera/src/creatures/flatus.rs`; each constant there carries its source.

```
 abdominal push ─┐
  + rectal       ▼  tissue around the pocket (Rayleigh–Plesset)
  recoil     rectal gas pocket ── ▷ IAS zone ◁ ── junction ── ▷ distal ring ◁ ── margin ── ▷ cheeks ◁ ── gluteal cleft ── air
             (FDTD tube: the gas   (internal sphincter:          (subcutaneous     (slot)    (contact strip  (soft-walled slot,
              law is the network)   tone − rectoanal reflex;      external          over the        radiating into
                                    cushions; deep EAS. LipReed)  sphincter,        ischioanal fat:  a baffle)
                                                                  voluntary:        LipReed)
                                                                  LipReed; mucus)
```

The model was then fitted to 59 events of 31 real recordings; the cheeks' valve and the mucus
came out of that validation. A second round split the sphincter in two: the internal sphincter's
zone, whose involuntary tone the rectoanal reflex inhibits, and the external sphincter's ring,
whose voluntary command can change during an event and reaches the muscle through its activation
dynamics; and it took the closures' stiffness from the anal canal's measured compliance instead
of the face's (below, "Validation against recordings", and
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

**Two sphincters, two closures.** The anal canal is held shut by two muscles of different kinds
(Lestar, Penninckx & Kerremans 1989, Int J Colorectal Dis 4:118; 21 subjects, probes of 0.3–3 cm,
conscious and under anaesthesia with curarisation, and with a maximal rectoanal reflex): at rest
30 % of the closing pressure is the external sphincter's tonic activity (striated muscle), 45 %
the internal sphincter's nerve-driven and 10 % its myogenic tone (smooth muscle), and 15 % the
expansion of the haemorrhoidal plexuses, the anal cushions. They sit at different heights:

- **The internal sphincter's zone**, above the intersphincteric groove: the internal sphincter
  inside, the deep and superficial external sphincter outside, the cushions lining it (the
  internal cushions lie above the dentate line, so they belong to this zone, not to the verge).
  Its closing pressure is the internal sphincter's tone (control "Int. sphincter",
  default 37 cmH₂O: 55 % of the median resting pressure of 67 in Luft et al. 2012) less what the
  rectoanal reflex inhibits, plus the cushions' 10 cmH₂O (15 % of 67, constant: the plexus fills
  and empties over seconds), plus 17/32 of the external sphincter's pressure. That share comes from
  the reflex's manometry (Cheeney, Nguyen, Valestin & Rao 2012, Neurogastroenterol Motil
  24:e147; 10 volunteers): during a full reflex 28 mmHg stays at the internal sphincter's level
  (1.6 cm from the verge), of which the cushions' 11, against 32 mmHg at the external sphincter's
  peak at 0.9 cm.
- **The distal ring**, the last centimetre, where only the subcutaneous external sphincter
  surrounds the canal. Its closing pressure is the external sphincter's.

The gas reaches the canal from the rectum, opens the internal sphincter's zone when its pressure
exceeds that zone's closing pressure (a radial closure: the gas below it presses it open too, so
its load is the gas's gauge pressure above it), fills the space below and then pushes the distal
ring open. The space between the two closures is one 0.75 cm section of the airway network, the
shortest that stays stable at the smallest body size; anatomically the two abut.

**The rectoanal inhibitory reflex.** Distending the rectum inhibits the internal sphincter.
Cheeney et al. (2012) measured the anal pressure fall: 20 mmHg at 15 mL and its maximum, 47 mmHg,
from 71 mL on, fractions 0.43 and 1 of the internal sphincter's share. The model's nadir depth,
1 − exp(−V/27 mL), passes through both. After the nadir the sphincter recovers to a plateau that
depends on the volume: fully at 10 mL, partly at 40, to just above the nadir at 100 and not at all
at 160 mL ("linearly related to the volume"); the model's plateau is the nadir's depth times a
line from 0 at 10 mL to 1 at 115 mL. The reflex's latency is 5.1 s (Sangwan et al. 1995, Dis Colon
Rectum 38:1281), longer than most events, and it is the "sampling reflex" that lets the gas into
the upper canal to be sensed (Huizinga et al. 2021, Front Med 8:720558): the model takes it to have
acted before the event, so each event starts at the nadir for its volume. As the gas leaves, the
sphincter recovers towards the remaining volume's plateau at the measured speed, 2.98 mmHg/s
against a 28 mmHg relaxation in asymptomatic controls (Netinho et al. 2005, Arq Gastroenterol
42:19): 1/9.5 of the inhibition per second. So small volumes meet a half-toned internal sphincter
(at 20 mL it holds 18 of its 37 cmH₂O), large ones an open zone, and a long event's end meets a
sphincter closing again. An internal sphincter tone of 0 stands for the coloanal reflex, which
relaxes both sphincters completely when a colonic pressure wave brings gas into the rectum
(Huizinga et al. 2021); the model has no such wave of its own.

**The external sphincter: voluntary, through muscle dynamics.** Its command (control "Ext.
sphincter", 0–250 cmH₂O, the pressure it makes once the muscle has followed: 0 fully
relaxed, 20–30 its tonic share at rest, up to 200 more squeezing, Luft et al. 2012) can change
during an event: a call's keyframes move it (calls "Clench" and "Quiver"), and the CLI and the
fitting renderer accept a course (`eas=t:v,t:v,…`). The command reaches the muscle's force through
first-order activation dynamics, da/dt = (u − a)/τ with τ = 10 ms rising and 40 ms falling, the
values of the musculoskeletal literature (Zajac 1989, Crit Rev Biomed Eng 17:359; Winters 1990)
for limb muscles of mixed fibres. The external anal sphincter is about 80 % slow (type I) fibres
(Konsten, Baeten, Havenith & Soeters 1993, Dis Colon Rectum 36:559), so its own constants are
probably longer: not measured. Isometric, the closing pressure follows the activation. So a
clench closes the ring within tens of milliseconds, never at once, and a modulated command
faster than about 4 Hz is smoothed away.

**Each closure is a pressure-opened soft-tissue valve**, like a brass player's lips (Fletcher's
1993 (+,−) valve), so the model uses the engine's `LipReed`, the single-mass valve the elephant's
trumpet uses. What differs between relaxed and contracted is derived:

- **Mass per unit face area, from incompressibility.** Soft tissue is incompressible at these
  rates, so the area A = ℓ·h the gas opens in the slit-shaped lumen (ℓ = 1 cm) must be pushed out
  through every contour around it: at distance r the tissue moves at Ȧ/2πr, and its kinetic
  energy per unit canal length is ½·ρ·ln(b/a)/(2π)·Ȧ², from the slit's conformal radius a = ℓ/4 to
  b, the closure's height H, beyond which the flow spreads in three dimensions. As LipReed's mass:
  m = ρ·ℓ·ln(4H/ℓ)/π, 0.46 g/cm² for the 1 cm distal ring, 0.67 for the 1.85 cm internal
  sphincter's zone. It is the same relaxed or contracted: the tissue has to move either way.
- **Passive stiffness, from the canal's measured compliance.** Barostat distension of the anal
  canal with the balloon checked on MRI, 33 healthy women (Sharma, Feuerhak, Corner, Manduca &
  Bharucha 2021, Neurogastroenterol Motil 33:e13972): past the opening pressure the volume grew by
  0.30 (SD 0.18) mL/mmHg over the 3.2 cm canal, a cross-sectional compliance C_A = 0.094 cm² per
  mmHg. A slit passing A = ℓ·h, with h twice each face's displacement Δr, has
  k = dΔp/dΔr = 2ℓ/C_A = 2.8·10⁴ dyn/cm³ (SD range 1.8–7.1·10⁴), a tenth of the facial cheek's
  stiffness (Ishizaka et al. 1975) the model borrowed before. It is measured awake (the canal
  relaxes as it is distended, and the toned muscle's length–tension stiffens it) and quasi-
  statically; under anaesthesia with curarisation and a maximal reflex, passive elements bear load
  only beyond 1 cm of distension (Lestar et al. 1989), so at a gas jet's sub-mm gaps the passive
  part is if anything smaller. No study gave the canal's pressure–area curve under anaesthesia.
- **Active stiffness, from the muscle's short-range stiffness.** K = γ·F/l with γ = 23.4 (Cui,
  Perreault, Maas & Sandercock 2008). A ring of radius R holds the slit shut with hoop tension
  T = P·R. With incompressible tissue between them, the slit's opening dA widens the ring by
  dA/2πR, so dP/dA = (γ − 1)·P/(2πR²) (−1 from Laplace: the same tension on a wider ring presses
  less), and per unit face area k = (γ − 1)·P·ℓ/(πR²): 4.2·P for the external sphincter
  (R = 1.3 cm), 18.0·P for the internal one (R = 0.63 cm, the mucosal core's 0.5 cm and half the
  sphincter's 2–3 mm). The internal sphincter is smooth muscle, whose γ is not measured; the same
  γ is used (stiffness from attached cross-bridges, in proportion to force, in both). The earlier
  model's γ·P/R (18·P at 1.3 cm) treated the lumen as a circle of the ring's radius while its
  flow passed a 1 cm slit.
- **Preload:** the closing pressure; the rest gap is −2P/k, so a closure opens when the pressure
  on it exceeds its closing pressure.

So a relaxed closure is soft (2.8·10⁴ dyn/cm³ plus little) and its natural frequency low (39 Hz
for the distal ring with the external sphincter at 0, 78 Hz at 20 cmH₂O); a contracted one is
stiff (188 Hz at 150 cmH₂O). The valve's duct is a Poiseuille slit of the closure's height, with
Ishizaka & Flanagan's entrance and exit losses (0.37, 1.0), as at the glottis. Pitch is never
set; it comes out of the closures' masses and stiffnesses, the pressures, and the coupling to the
pocket and the canal. The valve sounds above its mechanical resonance, as outward-striking lips do
(Newton, Campbell & Gilbert 2008), and near the pocket's own resonance when the closure is soft.

**Mucus at the verge (control, 0 = dry).** The canal's faces can be wet. After each contact a
liquid bridge joins them. While it holds it fills the gap, so no gas passes, and it pulls the
parting faces together by Bhattacharya & Siegmund's (2014, Biomech Model Mechanobiol 14:155)
traction–separation law for the liquid on colliding vocal folds: the traction rises linearly to
the cohesive strength σ₀ at a separation δ₀ = 0.1 mm and falls to nothing at δf = 0.267 mm, where
the bridge breaks (their baseline values). σ₀ is the control, 0–3 kPa; 3 kPa is the cohesive
strength of intestinal mucus (Mortazavi & Smart 1994), the upper bound Bhattacharya & Siegmund
used. Each opening then starts late and all at once, and the openings come irregularly.

**Turbulence.** The engine's Reynolds-number noise sources, at their calibrated gains:

- At each closure's jet, the human voice's glottal aspiration source (gain 3.2e-6, band
  300–3500 Hz). The mechanism is the same: a jet from a slit valve into a duct.
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
| Ext. sphincter: the external sphincter's voluntary command (cmH₂O) | 0–250 | 20 | the command: the closing pressure the external sphincter makes once its activation has followed (10 ms up, 40 ms down); not smoothed, may change during an event |
| Int. sphincter: the internal sphincter's tone (cmH₂O) | 0–80 | 37 | its tone before the rectoanal reflex inhibits it; 0 is the coloanal reflex's full relaxation |
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

| Call | Push | External sphincter | Gas | Spread |
|---|---|---|---|---|
| Squeak | 50 | 40 | 20 | 0 |
| Rumble | 8 | 5 | 300 | 0.3 |
| Staccato | five 0.15 s pushes of 30 | 20 | 200 | 0.2 |
| Clench | 12 | 10, then 150 from 0.6 s | 300 | 0.3 |
| Quiver | 10 | alternating 5 and 35 every 0.125 s (4 squeezes a second) | 200 | 0.3 |
| Slow leak | 6 | 4 | 80 | 1 |

The internal sphincter's tone is the performer's in every call (default 37). The squeak's push
was 42 before the two sphincters; at 20 mL the reflex leaves half the internal sphincter's tone,
which held it shut, so it pushes 50; the slow leak's 4 became 6 for the same reason.

## Parameters and sources

Adult values (body size 1). "Not measured" means no measurement was found and the value comes
from the reasoning given.

| Parameter | Value | Source or derivation |
|---|---|---|
| Anal canal (high-pressure zone) length | 3.6 cm | 3.6 ± 0.1 cm (10th–90th 2.8–4.4), HR manometry, healthy women (Noelting et al. 2012, Am J Gastroenterol 107:1530) |
| Distal ring height | 1.0 cm | below the intersphincteric groove only the subcutaneous EAS surrounds the canal; anatomy-text figure; the EAS's manometric peak is at 0.9 cm (Cheeney et al. 2012) |
| Junction between the closures | 0.75 cm, 1.05 cm across | not anatomy: one FDTD section, the shortest stable at 44.1 kHz and size 0.6; diameter the geometric mean of the closed mid canal (~5 mm) and the FLIP-distended upper canal (20–24 mm) (Luft et al. 2012, Tech Coloproctol 16:451), not measured |
| Internal sphincter's zone height | 1.85 cm | the rest of the high-pressure zone (3.6 − 1.0 − 0.75) |
| Closed lumen slit length (valve width) | 1.0 cm | slit across the ~1 cm mucosal core; the sub-mm gaps of a gas jet don't unfold the folds |
| External sphincter ring mean radius | 1.3 cm | mucosal core ~1 cm, IAS 2–3 mm, EAS 5–8 mm (typical endoanal ultrasound figures, not checked against one study) |
| Internal sphincter ring mean radius | 0.63 cm | the core's 0.5 cm + half the IAS's 2–3 mm (same figures) |
| Tissue density | 1.05 g/cm³ | soft tissue |
| Closures' mass per face area | ρℓ·ln(4H/ℓ)/π: 0.46 g/cm² (ring), 0.67 (IAS zone) | incompressible tissue displaced by the slit's opening, a 2-D source out to the closure's height H (derivation above); was a 0.7 cm plate, 0.74 g/cm² |
| Passive stiffness | 2ℓ/C_A = 2.8·10⁴ dyn/cm³ (SD range 1.8–7.1·10⁴) | anal canal compliance 0.30 (SD 0.18) mL/mmHg over 3.2 cm, barostat with MRI, 33 healthy women (Sharma et al. 2021, Neurogastroenterol Motil 33:e13972); awake and quasi-static; was the facial cheek's 3·10⁵ (Ishizaka et al. 1975) |
| Active stiffness | (γ − 1)·P·ℓ/(πR²), γ = 23.4: 4.2·P (EAS), 18·P (IAS) | short-range stiffness K = γ·F/l_F (Cui, Perreault, Maas & Sandercock 2008, J Biomech 41:1945) of a ring around a slit, incompressible tissue between; smooth muscle's γ not measured, the same used |
| Resting pressure split | EAS 30 %, IAS 55 %, cushions 15 % | Lestar, Penninckx & Kerremans 1989 (Int J Colorectal Dis 4:118) |
| Anal cushions' pressure | 10 cmH₂O, constant, on the IAS zone | 15 % of the median resting 67 cmH₂O (Lestar 1989; Luft 2012); internal cushions lie above the dentate line |
| EAS share at the IAS zone | 17/32 of its pressure | residual 28 mmHg at 1.6 cm during a full reflex, less the cushions' 11, against 32 mmHg at 0.9 cm (Cheeney et al. 2012) |
| EAS activation dynamics | τ 10 ms rising, 40 ms falling | first-order activation (Zajac 1989, Crit Rev Biomed Eng 17:359; Winters 1990), limb muscles; EAS ~80 % type I (Konsten et al. 1993, Dis Colon Rectum 36:559): probably slower, not measured |
| Reflex depth (nadir) | 1 − exp(−V/27 mL) | through 43 % at 15 mL and full from 71 mL (Cheeney, Nguyen, Valestin & Rao 2012, Neurogastroenterol Motil 24:e147) |
| Reflex plateau | nadir × (V − 10)/(115 − 10), clamped 0–1 | recovery full at 10 mL, partial at 40, near the nadir at 100, none at 160 mL; "linearly related to the volume" (Cheeney 2012, one subject's traces and the group's statement) |
| Reflex timing | starts at the nadir; recovers at 1/9.5 of the inhibition per s | latency 5.1 s (Sangwan et al. 1995, Dis Colon Rectum 38:1281), taken as passed; recovery 2.98 mmHg/s against a 28 mmHg relaxation, asymptomatic controls (Netinho et al. 2005, Arq Gastroenterol 42:19) |
| Closures' damping ratio | 0.1 | not measured for anal tissue; Ishizaka & Flanagan 1972, vocal-fold mass |
| Valve kinetic losses | entrance 0.37, exit 1.0 | Ishizaka & Flanagan 1972 (as at the engine's glottis) |
| Rectal compliance | 11 mL/mmHg | barostat, normal anorectal function, 11.4 ± 5.8 (Colorectal Dis 2013, 15:715); linear through zero volume |
| Gas pocket shape | length/diameter 2.5 | rectal ampulla ~12 cm long, 4–6 cm across distended |
| Pocket floor | 2 mL | not measured: what stays trapped when the walls meet |
| Tissue viscosity (Rayleigh–Plesset) | 2 Pa·s | order of soft-tissue shear viscosity (1–3 Pa·s, shear-wave dispersion of liver and muscle); not a pelvic measurement |
| Cleft depth at the anus | 3.0 cm | 2.1 cm measured at the sacrococcygeal level (Akinci et al. 2009, Colorectal Dis 11:753), deeper caudally; at the anus not measured |
| Residual gap, pressed cheeks | 1 mm | not measured: skin folds and hair leave channels in the margin and the cleft; where the cheeks meet beside the verge they touch |
| Cleft opening when spread | right angle | the cheek surfaces at 90° |
| Push range and default | 0–80, 30 cmH₂O | rectal pressure during simulated evacuation 20 ± 3 mmHg (10th–90th 0.7–47 mmHg) (Noelting 2012) |
| External sphincter command range and default | 0–250, 20 cmH₂O | its tonic share of a resting pressure ~30 % (Lestar 1989), squeeze increment 25–200 (median 92), highest squeeze 252 cmH₂O (Luft 2012); gas passes when it is relaxed below the push |
| Internal sphincter tone range and default | 0–80, 37 cmH₂O | 55 % of the resting pressure (Lestar 1989) of 32–133 cmH₂O, median 67 (Luft 2012); 0 = relaxed by the coloanal reflex (Huizinga et al. 2021, Front Med 8:720558) |
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

These are the model's outputs after the second round (two sphincters, the closures' stiffness
from the anal canal; the defaults are dry, internal sphincter tone 37). I couldn't listen to
them. "Validation against recordings" below compares them with real flatus.

**Presets** (and the held voice at the defaults):

| | Duration | Level | f0 median (5–95 %) | Periodic | Centroid |
|---|---|---|---|---|---|
| Held, defaults (push 30, EAS 20, IAS 37, gas 100, spread 0.2, dry) | 0.23 s | −19.0 dBFS | 244 Hz* | 5 % | 1505 Hz |
| Squeak | 0.16 s | −19.1 | 284 Hz* | 8 % | 1615 Hz |
| Rumble | 0.50 s | −19.1 | 123 Hz* (117–127) | 13 % | 1288 Hz |
| Staccato | 1.81 s | −24.7 | – | 0 % | 1602 Hz |
| Clench (cut off at 0.6 s, gas left) | 0.51 s | −18.1 | 77 Hz* (60–128) | 21 % | 1005 Hz |
| Quiver | 1.11 s | −24.9 | 115 Hz* | 4 % | 1888 Hz |
| Slow leak | 1.14 s | −26.1 | 67 Hz (65–127) | 56 % | 2321 Hz |

**One control at a time** (the others at the defaults):

| Control | Values | Duration (s) | Level (dBFS) | f0 median (Hz) | Centroid (Hz) |
|---|---|---|---|---|---|
| Push (cmH₂O) | 10 / 15 / 20 / 30 / 45 / 60 / 80 | silent / 0.56 / 1.14 / 0.23 / 0.13 / 0.09 / 0.08 | – / −20.9 / −39.6 / −19.0 / −16.8 / −17.5 / −13.6 | – / 145 / 148 / 244* / 179* / – / – | – / 1951 / 1587 / 1505 / 1077 / 324 / – |
| External sphincter (cmH₂O) | 0 / 5 / 10 / 20 / 40 / 80 | 0.08 / 0.11 / 0.13 / 0.23 / 0.82 / silent | −23.9 / −28.4 / −28.0 / −19.0 / −37.6 / – | – / – / 142 / 244* / 143 / – | – / 3385 / 534 / 1505 / 829 / – |
| Internal sphincter tone (cmH₂O) | 0 / 20 / 37 / 60 / 80 | 0.19 / 0.21 / 0.23 / 0.47 / 0.92 | −17.5 / −18.6 / −19.0 / −21.0 / −23.3 | 197 / 205 / 244* / – / 177 | 1330 / 1418 / 1505 / 1754 / 1716 |
| Gas (mL) | 20 / 50 / 100 / 200 / 300 | silent / 1.05 / 0.23 / 0.27 / 0.31 | – / −21.7 / −19.0 / −16.2 / −14.7 | – / –† / 244* / 202* / 220* | – / 2212 / 1505 / 1371 / 1126 |
| Cheek spread | 0 / 0.25 / 0.5 / 0.75 / 1 | 0.27 / 0.22 / 0.25 / 0.26 / 0.29 | −21.7 / −19.0 / −19.4 / −18.7 / −19.1 | 166* / 220* / 216* / 222* / – | 1484 / 1477 / 1433 / 1554 / 1508 |
| Body size | 0.6 / 0.8 / 1 / 1.25 | 0.15 / 0.20 / 0.23 / 0.38 | −23.1 / −20.9 / −19.0 / −16.7 | – / – / 244* / – | 2544 / 1916 / 1505 / 1384 |
| Mucus (kPa) | 0 / 0.5 / 1 / 1.5 / 2 | 0.23 / 0.26 / 0.25 / 0.26 / 0.24 | −19.0 / −18.2 / −18.3 / −18.2 / −15.9 | 244* / 228* / 230* / 236* / – | 1505 / 1877 / 1936 / 1862 / 1853 |

\* Under 25 % of frames are periodic: the oscillation is irregular, and the median means little.
– No periodic frames. † The median lands on a 2 kHz artefact of YIN on a near-silent tail.

**Near threshold, the external sphincter relaxed or contracted** (internal sphincter relaxed,
`ias=0`, cheeks spread; the push a few cmH₂O above what opens the canal):

| External sphincter (cmH₂O) | Push | Duration | Level | f0 (periodic) | Closure's natural frequency |
|---|---|---|---|---|---|
| 5 | 4 | 2.00 s | −35.0 | 118 Hz (91 %) | 52 Hz |
| 10 | 11 | 0.81 s | −26.9 | 91 Hz (50 %) | 61 Hz |
| 20 | 17 | 1.47 s | −34.9 | 79 Hz (34 %) | 78 Hz |
| 40 | 31 | 0.91 s | −28.6 | 166 Hz (94 %) | 104 Hz |
| 80 | 71 | 0.36 s | −15.6 | 179 Hz (55 %) | 140 Hz |

How the sound follows the physics:

- **Relaxed or contracted is a different instrument.** Relaxed, the closures are soft (the canal's
  measured compliance) and, opened just past their closing pressure, hold clean low tones for
  seconds: 118 Hz, 91 % periodic, with the external sphincter at 5 cmH₂O. The old model's clean
  tones bottomed out near 160 Hz at the defaults (the previous round's fits did reach 122 Hz with
  a large body). Contracted, the short-range stiffness raises the closure's natural frequency
  (104 Hz at 40, 140 Hz at 80 cmH₂O) and the tone (166–179 Hz). The sounding pitch of a soft
  closure is not its natural frequency but near the gas pocket's own resonance (about 110 Hz at
  100 mL); it hardly moves across the passive stiffness's SD range (121 and 127 Hz at 0.63× and
  2.5× the stiffness).
- **Pushed well past the closing pressures, the canal gapes.** A relaxed closure is soft, so a
  push of 30 against an external sphincter at 20 opens it by millimetres: 100 mL leave in
  0.2 s, mostly aperiodic, and from push 45 on the peaks reach the output limiter's knee. The
  measured compliance implies this: 15 mmHg past the opening pressure, the barostat balloon took
  1.4 cm² of the canal (Sharma et al. 2021). Long and tonal events need pushes near threshold.
- **The internal sphincter gates the event by the gas volume.** At 20 mL the reflex leaves half
  its tone, which holds against push 30: continence (the squeak needs 50). At 100 mL and more it
  is nearly gone, and the zone opens at the cushions' 10 cmH₂O plus the external sphincter's
  share. A stronger internal sphincter (60–80 cmH₂O) slows the event (0.47–0.92 s): the zone
  opens less.
- **A clench cuts the flow off within muscle time.** The command jumps; the pressure follows with
  τ = 10 ms, and the ring shuts within about 20 ms, with gas left (`tests/flatulence.rs` checks
  4–60 ms; in the Clench call the flow is 38 mL/s at the command and 0 20 ms later, 41 of the
  300 mL left). A command modulated faster than about 4 Hz is
  smoothed by the 40 ms deactivation.
- **Gas sets the duration only near threshold**; well past it, 200–300 mL leave in 0.3 s.
- **Mucus** still makes the openings late and irregular (periodic 5 → 15 %, centroid 1505 →
  1877 Hz at 0.5 kPa) and holds a weak push: near threshold (push 8, external sphincter 5) 1 kPa
  holds the last 40 mL once the emptying rectum's recoil has fallen.
- **Events end when the gas is gone** (or when a closure shuts on the rest). A 100 mL event
  conserves gas: what crosses the anus is what left the pocket, within 8 mL.
- **An external sphincter above the push plus the recoil keeps the canal shut**: 80 against push
  30 is silent: continence.
- **Pressed cheeks sputter near threshold** (external sphincter 10, push 8, internal sphincter
  relaxed): the 100 mL leave over 3.3 s in puffs ended by the cheeks falling shut, more than 15 a
  second.

**Sensitivity to the values that weren't measured.** The table below is from the first round's
model (one sphincter, the face's stiffness) and has not been repeated for the second round,
except the passive stiffness above.

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
its timing, the sphincters' tone, gas, cheek spread, body size, mucus) that bring the model's render closest to
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
renders per event. In the second round the controls are push, the external sphincter's command,
the internal sphincter's tone, gas, cheek spread, body size and mucus, and after the constant fit
the command may become a course (`fit.py courses=1`): a ramp (held at e0, then linearly to e1 over
a fitted time from a fitted start: clench to cut off, relax to let go; 40 renders), then a
sinusoidal modulation around it (0.5–8 Hz, up to 60 cmH₂O; 25 renders). The creature passes the
command through the muscle's activation dynamics, so the sphincter follows it no faster than
muscle can; nothing shapes the sound itself. Each stage's best is kept, so what the course adds
can be read apart. This is a local search: the same model searched from another library moves
single events by up to ±0.3 (log, experiment 5), so the counts below are what the search found,
a lower bound on what the model can do.

**Results.** Distances of the best fit, per recording (its best event); bold is close (≤ 0.54).
"Baseline" is the model before any validation (commit 788bdd0), "Round 1" the model with the
cheeks' valve and the mucus control (commit c258ac2), "Now" the two-sphincter model with the
closures' stiffness from the anal canal, its external sphincter's command allowed a course; each
of the last two is each event's best of two searches:

| Recording | Baseline | Round 1 | Now | Best event: fitted controls | Largest misses (real − model, in spreads) |
|---|---|---|---|---|---|
| fs573745 (Authentic Fart) | 0.94 | 0.93 | 0.71 | 0: push 3, eas 2, ias 19, gas 200, spread 0.00, size 0.72, mucus 0.00; push 0.88 s | b177 -2.3, sustain +1.5, b2828 -1.7 |
| fs518699 (FART - 2) | 0.84 | **0.48** | **0.29** | 0: push 5, eas 3, ias 9, gas 218, spread 0.00, size 1.12, mucus 0.00; push 0.50 s | – |
| fs518701 (FART - 3) | 0.68 | 0.56 | **0.31** | 0: push 35, eas 2, ias 45, gas 28, spread 0.07, size 1.25, mucus 0.00; push 0.61 s; eas 1 → 4 from 0.30 s over 0.17 s | – |
| fs186025 (fart 2012-09-14.flac) | **0.51** | **0.47** | **0.39** | 0: push 12, eas 2, ias 8, gas 205, spread 0.32, size 1.25, mucus 0.48; push 0.52 s; eas 2 → 2 from 0.20 s over 0.11 s | – |
| fs121783 (FART.aif) | **0.52** | **0.50** | 0.70 | 0: push 28, eas 33, ias 77, gas 300, spread 1.00, size 0.78, mucus 0.00; push 0.66 s; eas 33 → 47 from 0.37 s over 0.10 s | b177 -2.1, b2828 -2.1, b707 +1.9 |
| fs582215 (Fart_short_with_pressure_w…) | 0.70 | **0.54** | **0.52** | 0: push 51, eas 49, ias 31, gas 203, spread 1.00, size 1.25, mucus 0.00; push 0.79 s; eas 49 → 49 from 0.29 s over 0.10 s | – |
| fs135716 (Fart.wav) | 0.77 | **0.31** | **0.29** | 1: push 5, eas 19, ias 62, gas 235, spread 0.01, size 1.25, mucus 0.00; push 0.44 s; eas 19 → 15 from 0.37 s over 0.06 s | – |
| fs185226 (funny fart) | 0.76 | 0.76 | 0.58 | 0: push 4, eas 2, ias 15, gas 197, spread 0.00, size 0.73, mucus 0.00; push 0.94 s | log_dur +0.9, periodic +0.9, sustain +1.3 |
| fs616782 (Fart.mp3) | 0.56 | **0.40** | **0.28** | 0: push 43, eas 0, ias 71, gas 50, spread 0.00, size 1.21, mucus 0.36; 2 pushes of 0.06 s every 0.10 s; eas 0 → 0 from 0.14 s over 0.23 s | – |
| fs737546 (Flatulence) | **0.51** | **0.45** | **0.27** | 0: push 8, eas 0, ias 71, gas 217, spread 0.01, size 0.82, mucus 1.44; push 0.22 s | – |
| fs803697 (26 - fart) | 0.61 | **0.53** | **0.33** | 0: push 3, eas 2, ias 43, gas 200, spread 0.00, size 0.70, mucus 0.00; push 0.24 s; eas 2 → 3 from 0.12 s over 0.11 s | – |
| fs874487 (Fart - Close recording) | 0.55 | 0.59 | **0.43** | 0: push 8, eas 35, ias 39, gas 300, spread 0.00, size 0.85, mucus 0.00; push 0.23 s; eas 35 → 35 from 0.09 s over 0.10 s | periodic +1.0, sustain +1.2 |
| fs864457 (a fart) | **0.45** | **0.52** | **0.43** | 0: push 25, eas 15, ias 71, gas 300, spread 1.00, size 0.82, mucus 0.00; push 0.34 s; eas 15 → 31 from 0.16 s over 0.10 s | periodic +0.8 |
| fs137008 (Homemade fart) | 0.80 | 0.61 | **0.33** | 0: push 18, eas 1, ias 31, gas 48, spread 0.10, size 0.75, mucus 1.31; 4 pushes of 0.26 s every 0.44 s; eas 1 → 1 from 0.89 s over 0.12 s | – |
| fs854139 (Bathroom Fart) | 0.77 | 0.69 | **0.54** | 0: push 4, eas 15, ias 0, gas 300, spread 0.00, size 0.87, mucus 0.00; push 0.32 s | f0_slope -1.6, b88 -1.6 |
| fs858198 (2026 Fart) | 0.64 | 0.74 | 0.64 | 0: push 4, eas 28, ias 60, gas 206, spread 1.00, size 0.72, mucus 0.00; push 0.27 s; eas 27 → 29 from 0.12 s over 0.13 s | f0 +1.3, shr +1.2 |
| fs456558 (Liquid Destiny Fart.wav) | 0.55 | **0.53** | **0.50** | 0: push 15, eas 15, ias 39, gas 300, spread 0.00, size 0.93, mucus 0.00; push 1.81 s; eas 14 → 30 from 0.87 s over 0.10 s | log_attack +1.5, log_dur +0.8 |
| fs558387 (fart-series-fatulance-smal…) | **0.38** | **0.31** | **0.24** | 13: push 24, eas 0, ias 32, gas 49, spread 0.00, size 0.91, mucus 1.69; push 0.20 s; eas 0 → 1 from 0.09 s over 0.11 s | – |
| fs400715 (Superbly Realistic Flatule…) | 0.57 | **0.45** | **0.37** | 0: push 3, eas 10, ias 65, gas 180, spread 0.01, size 0.73, mucus 0.46; push 0.44 s; eas 10 → 8 from 0.28 s over 0.13 s | b354 -1.7 |
| fs402569 (Fart.mp3) | 0.62 | 0.55 | **0.38** | 0: push 43, eas 34, ias 0, gas 148, spread 1.00, size 1.20, mucus 1.57; push 0.39 s; eas 35 → 32 from 0.18 s over 0.10 s | – |
| fs523930 (Harley Davidson) | 1.24 | **0.53** | **0.48** | 0: push 6, eas 2, ias 14, gas 85, spread 0.00, size 1.25, mucus 0.00; push 4.69 s; eas 2 → 2 from 2.12 s over 0.10 s, ±24 at 3.0 Hz | bursts +1.2, log_dur +1.0 |
| fs266450 (Chair Fart) | 0.63 | **0.46** | **0.24** | 0: push 50, eas 50, ias 10, gas 27, spread 1.00, size 1.25, mucus 0.00; push 0.45 s | – |
| fs608980 (Short but strong fart.ogg) | **0.50** | **0.46** | **0.43** | 1: push 50, eas 25, ias 50, gas 200, spread 0.20, size 1.00, mucus 0.00; eas 25 → 51 from 0.12 s over 0.10 s | log_attack -1.5 |
| wc_0c7c1082 (425594 ylearkisto farts-pi…) | **0.54** | **0.35** | **0.17** | 5: push 38, eas 14, ias 71, gas 51, spread 0.00, size 1.24, mucus 0.21; 8 pushes of 0.05 s every 0.10 s; eas 11 → 14 from 0.48 s over 0.16 s | – |
| wc_3dced03c (Flatulence.wav) | 0.55 | **0.52** | **0.42** | 0: push 5, eas 1, ias 56, gas 220, spread 0.00, size 0.69, mucus 0.00; push 0.48 s; eas 1 → 3 from 0.17 s over 0.10 s | b707 +1.6, b2828 -1.6 |
| wc_adea4a8a (Flatulence de Clément.ogg) | 0.58 | **0.20** | **0.13** | 3: push 4, eas 0, ias 40, gas 300, spread 0.04, size 0.76, mucus 1.30; push 0.50 s; eas 0 → 0 from 0.21 s over 0.10 s | – |
| wc_e992050f (Flatus.wav) | **0.42** | **0.40** | **0.33** | 2: push 3, eas 2, ias 53, gas 212, spread 0.00, size 0.67, mucus 0.00; push 0.48 s; eas 2 → 4 from 0.16 s over 0.09 s | – |
| wc_6fb2b93f (Frt755 1.ogg) | 1.09 | **0.36** | **0.34** | 0: push 4, eas 1, ias 6, gas 237, spread 0.00, size 1.25, mucus 0.37; push 3.10 s | – |
| wc_359edc99 (Human fart.wav) | 0.88 | 0.86 | 0.89 | 0: push 60, eas 35, ias 0, gas 50, spread 1.00, size 1.20, mucus 0.00; eas 38 → 35 from 0.15 s over 0.10 s | periodic +2.0, log_dur +1.1, hnr +1.0 |
| wc_709a446b (Menselijke wind drietraps.…) | **0.44** | **0.52** | **0.46** | 2: push 21, eas 7, ias 47, gas 76, spread 0.26, size 0.69, mucus 0.17; push 0.99 s; eas 7 → 7 from 0.35 s over 0.10 s | sustain -1.4, b177 +1.6 |
| wc_69c63856 (Wet fart tummy rumbles.ogg) | 1.03 | **0.54** | 0.55 | 2: push 4, eas 0, ias 0, gas 51, spread 0.00, size 1.20, mucus 0.00; push 1.71 s | log_release -1.4, b2828 -1.9, centroid -0.8 |

| Model | Median distance | Events close (of 59) | Recordings reached (of 31) |
|---|---|---|---|
| Baseline | 0.67 | 12 | 9 |
| Round 1 (one search / best of two) | 0.54 / 0.53 | 31 / 34 | 20 / 22 |
| Closures' stiffness and mass from the anal canal only (one search) | 0.48 | 37 | 21 |
| Two sphincters, constant command (one search / best of two) | 0.44 / 0.43 | 40 / 44 | 23 / 24 |
| Two sphincters, command up to a ramp (best of two) | 0.42 | 46 | 24 |
| Two sphincters, command with ramp and modulation (one search / best of two) | 0.43 / 0.42 | 42 / 47 | 24 / 25 |

45 of the 59 events are now at least as close to the model as to their own nearest other
recording (round 1: 32). The largest mean misses moved from attack, duration and release (round
1) to the periodic fraction (real more periodic, +0.22 spreads), duration (real longer) and the
centroid (real lower). Fits at the gas maximum 16 → 13; at the largest body 11 → 15. 32 of the 59
fits relax the external sphincter to 3 cmH₂O or less, as people do to pass gas (and as the
coloanal reflex does; Huizinga et al. 2021); the fitted internal sphincter tone has a median of
38 cmH₂O, the population's 37.

**What the two sphincters and the course each bought.** The closures' stiffness and mass from the
anal canal alone took one search from 31 to 37 close events (20 → 21 recordings). The two
sphincters, with a constant command, took it to 40 (23); the second search and the merge to 44
(24). The command's course added 3 events and 1 recording on top (44 → 47, 24 → 25): within the
search noise (about ±3 events), and bought with up to six more parameters per event. Only 10 of
the 59 best fits use a course that moves the command by more than 3 cmH₂O, and 2 a modulation.
One of those is the case for it: the Harley Davidson sputter (a 4.7 s train of puffs) fits at
0.69 with a constant command and at 0.48, reached, with the command quivering ±24 cmH₂O at 3 Hz,
slow enough for the muscle to follow. Otherwise the level-free features hardly ask
for a changing contraction; that the model can clench, release and quiver is physics it now has,
not something these recordings were shown to need. A clench cutting an event off, or a
deliberate quiver, would show in the recordings' envelopes, and few of them have one.

**The low clean tones** that motivated the round: Bathroom Fart (142 Hz, HNR 14 dB) 0.69 → 0.54
(reached; fitted 134 Hz, HNR 12.6, external sphincter 15 cmH₂O, internal 0, push 4 against 300
mL), funny fart (93 Hz) 0.76 → 0.58 (the fit's pitch is not the recording's: 44 Hz, 10 %
periodic, where the recording is 34 %), 2026 Fart (254 Hz, HNR 17) 0.74 → 0.64 (fitted 150 Hz,
HNR 16.6: the clean tone is there, its pitch is not), Human fart.wav (132 Hz, 63 % periodic)
0.86 → 0.89 (the fits stay aperiodic). In round 1 the fitted Bathroom Fart was already at 122 Hz
(HNR 14), with a large body; what the relaxed closure adds is clean tones near 120 Hz at an
ordinary body size and a slow push.

**Level.** The features ignore level, because the recordings are uncalibrated, and the fit uses
that freedom. Taking only fits within 30 dB of the creature's default event's peak, round 1 had
32 events and 20 recordings close, now 45 and 25 (7 of 59 fits are more than 30 dB down, median
12 dB down). Within a recording, where microphone and distance are the same, the level
differences between events are a test the fit never saw: the model's differ from the
recordings' by a median 12 dB over 89 pairs (round 1: 12), by more than 20 dB in 17 (round 1: 23).

**What still misses.** Six recordings are not reached: Human fart.wav (0.89), Authentic Fart
(0.71), FART.aif (0.70; reached in round 1, 0.50, now its fits are brighter), 2026 Fart (0.64),
funny fart (0.58) and Wet fart tummy rumbles (0.55; 0.54 in round 1). By the features that miss:

- **Periodicity** (real more periodic, the largest mean miss): Human fart.wav, FART.aif, the
  Close recording. The near-threshold regime that holds clean tones is narrow (a few cmH₂O of
  push), and away from it the soft closures gape and the flow turns aperiodic.
- **High clean tones**: 2026 Fart's 254 Hz at HNR 17. Contracted closures play 160–190 Hz in the
  sweeps; tones above about 200 Hz come out irregular.
- **Duration and tails** (real longer): Authentic Fart, funny fart, Wet fart tummy rumbles. Some
  rooms reverberate (the model has no room); and the soft closures pass gas fast unless the push
  is near threshold.
- **Spectrum**: centroid (real lower, −0.30 spreads) and the 2.8 kHz band (real less, −0.70):
  consistent across events; recording chains (phones, small microphones) may share in it.


## Limits

- **No peer-reviewed acoustic data.** The validation above uses freely licensed recordings of
  unknown microphones, distances, rooms and people; no peer-reviewed recording or spectral
  analysis of flatus was found.
- **The flow is fast unless the push is near threshold.** With the canal's measured compliance
  the closures are soft and gape when pushed well past their closing pressures: 100 mL in 0.2 s
  at the defaults (peaks above 2 L/s). Long, tonal events come from pushes just above threshold,
  which the fits find; duration is still among the largest misses (real longer). No measurement
  of flatus flow rates was found. The compliance is the awake, quasi-static one (above).
- **Each closure is one mass.** No mucosal wave (no body–cover split: the incompressible tissue
  moves as one), no asymmetric lips, no localised opening of part of the ring. LipReed's contact
  floor (−0.5 mm) and maximum opening (8 mm) are its own. The two closures are a lumped picture of
  the canal's pressure profile (Cheeney et al. 2012); the 0.75 cm junction between them is
  numerical. The internal sphincter's zone is loaded by the gas above it, the distal ring by the
  pressure difference across it (the earlier model's (+,−) valve); a full collapsible-tube
  treatment (pressure along the closure, Bernoulli suction in the gap) is not made.
- **Muscle constants borrowed.** The external sphincter's activation time constants are limb
  muscles' (10/40 ms); the sphincter is mostly slow fibres, so it probably tightens and lets go
  more slowly. The smooth internal sphincter's short-range stiffness uses the striated γ. The
  muscles' length–tension at these sub-mm strains is taken as flat.
- **The rectoanal reflex is reduced to its nadir, its volume plateau and its recovery speed.**
  Its 5 s latency is assumed passed at every event's start; a new distension during an event
  (none happens: the pocket only empties) would need it. The plateau curve is read from one
  subject's traces and the group's statement; recovery is from 13 controls (Netinho 2005).
  The excitatory reflex (the external sphincter's reflex contraction on distension) and the
  coloanal reflex are not simulated (the latter only as an internal sphincter tone of 0).
- **The cushions are a constant pressure.** Their filling with straining (venous congestion) and
  emptying over seconds are not modelled.
- **The pocket is a sphere in liquid.** Rayleigh–Plesset treats the rectal pocket's surroundings
  as an unbounded liquid. The pelvis is bounded by bone (the sacrum) and by muscle, which would
  change the effective mass and damping. The pocket's acoustic shape is a 2.5:1 tube, while its
  wall dynamics use the equal-volume sphere.
- **The gas is the engine's air.** Flatus (mostly N₂, H₂, CO₂, CH₄) can be lighter than air when
  rich in H₂, which raises the speed of sound and the cavity resonances.
- **The rectal pressure–volume curve is linear through zero.** The flat start at low volumes
  (minimal distending pressure) is left out. With 300 mL the recoil alone (36 cmH₂O) exceeds a
  relaxed sphincter's closing pressure, and gas leaks without a push. That is physical in kind, but the threshold depends
  on this simplification.
- **Level is not fitted.** Five events (four recordings) are trains of distinct puffs at 14–19 a
  second lasting 1.3–4.2 s. In round 1 the model made them only with pressed cheeks near
  threshold, 30–70 dB below its ordinary events; now the best fits of the two sputters are 15–16
  dB below the default event (the Harley Davidson with the external sphincter quivering at
  3 Hz), but seven fits are still more than 30 dB down, and within a recording the model's level
  differences miss the recordings' by a median 12 dB. The features are level-free, so the fit
  doesn't see this (Validation, "Level").
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
