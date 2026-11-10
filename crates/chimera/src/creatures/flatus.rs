//! Flatulence: gas from the rectum passing the anus. Every parameter below is anatomy or physics
//! with its source or derivation; where measurements are missing the comment says so and gives the
//! physical reasoning instead. Nothing here was set by ear. See docs/flatulence.md.
//!
//! ```text
//!  abdominal push ─┐
//!   (+ rectal     ▼ (the tissue around the pocket: Rayleigh–Plesset, sample by sample)
//!    wall recoil) rectal gas pocket ── upper anal canal ── ▷ anal verge ◁ ── margin ── ▷ cheeks ◁ ── gluteal cleft ── air
//!                 (FDTD tube; its      (IAS zone, opened    (subcutaneous   (slot)    (their contact   (soft-walled slot
//!                  gas law is the       by the gas)          EAS ring:                 strip: LipReed   between the cheeks,
//!                  network's own)                            LipReed)                  of fat)          radiating)
//! ```
//!
//! - **Driving pressure.** The gas pocket is a tube of the airway network, so its gas law is the
//!   network's continuity equation, (V/ρc²)·dp/dt = ΣU (linearised adiabatic). Its wall is the
//!   surrounding tissue, treated as a liquid around a bubble: the pocket's equal-volume sphere
//!   obeys the Rayleigh–Plesset equation, R·R̈ + 1.5·Ṙ² = (p_gas − P_out)/ρ − 4μṘ/(ρR), where
//!   P_out is the abdominal push plus the rectal wall's elastic recoil V/C. The wall's motion
//!   (−dV/dt) is the flow it pushes into the gas, so the pressure holds while the tissue can
//!   follow, falls as gas leaves (the recoil term, and the gas law when the walls meet), and the
//!   event ends when the gas is gone. The same equation gives the pocket its resonance (the
//!   Minnaert frequency of a bubble in tissue, about 110 Hz at 100 mL).
//! - **Source.** The closed canal's distal ring is a pressure-controlled, outward-striking valve,
//!   like a brass player's lips (Fletcher 1993's (+,−) valve): the existing [`LipReed`]. Its mass is
//!   the tissue ring's, its stiffness the passive tissue's plus the sphincter muscle's short-range
//!   stiffness, which grows with its tone; its preload is the closing pressure. Pitch and
//!   squeak-or-rumble come out of those and the airflow; no frequency is set.
//! - **Turbulence.** The engine's Reynolds-number noise sources at their calibrated gains: the
//!   human glottal aspiration source at the valve's jet (same mechanism: a jet from a slit valve
//!   into a duct) and the human narrowest-constriction source in the cleft when it is narrow.
//! - **The cheeks.** Beside the verge the cheeks meet or nearly meet: a second pressure-opened
//!   valve, a LipReed whose mass and stiffness are the first thickness mode of the fat layer
//!   under the perianal skin (the ischioanal fossa's), about 23 Hz. Pressed, they touch and the gas
//!   has to part them; spread cheeks leave it open.
//! - **Mucus.** A wet canal's faces are joined by a liquid bridge after each contact: it seals
//!   the gap and pulls the faces together until they part by 0.27 mm (Bhattacharya & Siegmund's
//!   traction–separation law), so each opening starts late and all at once.
//! - **Radiation.** Through the gluteal cleft, a soft-walled wedge-shaped slot whose gap follows
//!   the cheek spread, into a baffled radiation load (the body surface around it).

use super::build::{CMH2O, TubeArea, TubeOpts, sections, tube};
use super::types::{CallDef, CallKey, CreatureDef, CreatureGroup, CreatureSim, GateTimes, ParamDef};
use crate::airway::network::{AirwayNetwork, NoiseSpec, PortSpec, Tube};
use crate::math::{PI, cbrt, js_max, js_min, sqrt, tan};
use crate::physics::WALL_SOFT;
use crate::sources::valves::LipReed;

// parameter indices (order of `def().params`)
const PUSH: usize = 0;
const TONE: usize = 1;
const GAS: usize = 2;
const SPREAD: usize = 3;
const SIZE: usize = 4;
const MUCUS: usize = 5;

/// Smallest and largest body scale (linear, 1 = adult). The tubes get their section counts at the
/// smallest, so every size keeps the FDTD's stability limit.
const SIZE_MIN: f64 = 0.6;
const SIZE_MAX: f64 = 1.25;

// ---------------------------------------------------------------- anatomy (adult, size 1)

/// Length of the anal canal's high-pressure zone (cm): 3.6 ± 0.1 cm (10th–90th 2.8–4.4) in healthy
/// women by high-resolution manometry (Noelting et al. 2012, Am J Gastroenterol 107:1530).
const HPZ_LEN: f64 = 3.6;
/// Height of the vibrating closure (cm): below the intersphincteric groove, about 1 cm above the
/// anal verge, only the subcutaneous external sphincter surrounds the canal (standard anatomy; the
/// height is an anatomy-text figure, not a measurement series). Above it the internal sphincter
/// relaxes when the rectum distends (rectoanal inhibitory reflex), so gas enters the upper canal
/// and the remaining closure is this distal ring.
const VALVE_LEN: f64 = 1.0;
/// Diameter of the gas-filled upper canal (cm). No measurement of it exists. FLIP planimetry puts
/// the closed mid canal at about 5 mm (the probe's floor) and the upper canal at 20–24 mm when a
/// bag distends it (Luft et al. 2012, Tech Coloproctol 16:451); this is their geometric mean.
const UPPER_CANAL_D: f64 = 1.05;
/// Length of the closed lumen's slit (cm): the closed canal is a slit or star across its mucosal
/// core, about 1 cm wide (the core inside the sphincters; see RING_RADIUS). A slit of length ℓ
/// whose faces part by a gap h passes ℓ·h; a ring opened radially by Δr parts them by 2Δr. Wider
/// openings unfold the mucosal folds (the canal passes 2–4 cm stool), which a gas jet's sub-mm
/// gaps don't reach.
const SLIT_LEN: f64 = 1.0;
/// Mean radius of the sphincter ring that carries the hoop tension (cm): a mucosal core about 1 cm
/// across, the internal sphincter 2–3 mm and the external 5–8 mm thick (typical endoanal
/// ultrasound figures, not checked against one study here); the middle of the external sphincter
/// sits about 1.3 cm from the axis.
const RING_RADIUS: f64 = 1.3;
/// Thickness of tissue moving with the lip (cm): anoderm and submucosa (≈2 mm) and the
/// subcutaneous external sphincter (≈5 mm) — the ring that opens.
const LIP_THICKNESS: f64 = 0.7;
/// Soft-tissue density (g/cm³).
const TISSUE_DENSITY: f64 = 1.05;
/// Passive stiffness of the tissue around the lip, per unit area (dyn/cm³): the engine's measured
/// soft-tissue wall (Ishizaka, French & Flanagan 1975, cheek: skin, fat and a thin muscle sheet,
/// the closest measured tissue to the perianal skin, fat and sphincter).
const PASSIVE_STIFFNESS: f64 = WALL_SOFT.stiffness;
/// Short-range stiffness of active muscle in proportion to its force: K = γ·F/l_F with γ = 23.4,
/// fitted across six cat hindlimb muscles (Cui, Perreault, Maas & Sandercock 2008, J Biomech
/// 41:1945; after Rack & Westbury 1974). For a ring of hoop tension T = P_close·r opened by Δr
/// (fibre strain Δr/r), the closing pressure rises by γ·P_close·Δr/r: a radial stiffness
/// γ·P_close/r per unit area. Short-range stiffness holds for small stretches, of the order of the
/// lip's motion here (Δr/r ≈ 1–5 %); the sphincter's own γ is not measured.
const SRS_GAMMA: f64 = 23.4;
/// Damping ratio of the lip. No measurement for anal tissue; 0.1 is Ishizaka & Flanagan's (1972)
/// for the vibrating vocal-fold mass, the soft-tissue valve whose damping was fitted to data.
const LIP_ZETA: f64 = 0.1;
/// Rectal compliance (cm³ per dyn/cm²): 11 mL/mmHg, the mean barostat compliance of subjects with
/// normal anorectal function (11.4 ± 5.8 mL/mmHg; Colorectal Dis 2013, 15:715). Linear through
/// zero volume: the pressure–volume curve's flat start (minimal distending pressure) is ignored.
const RECTAL_COMPLIANCE: f64 = 11.0 / 1333.22;
/// Aspect ratio (length/diameter) of the gas pocket: the rectal ampulla is about 12 cm long and
/// 4–6 cm across when distended; the pocket keeps that shape as it shrinks.
const POCKET_ASPECT: f64 = 2.5;
/// The pocket's floor (cm³): what stays trapped in folds when the walls meet. Not measured; a
/// few mL. Below it there is no pocket for the wall to squeeze, so its last gas expands out.
const POCKET_FLOOR: f64 = 2.0;
/// Shear viscosity of the soft tissue around the pocket (poise): soft tissues measure of the order
/// of 1–3 Pa·s by shear-wave dispersion (liver, muscle; order of magnitude, not a pelvic
/// measurement); 2 Pa·s. It damps the pocket's pulsation (Rayleigh–Plesset's 4μṘ/R) lightly; ten
/// times more changes the measurements little (docs/flatulence.md).
const TISSUE_VISCOSITY: f64 = 20.0;
/// Depth of the gluteal cleft at the anus (cm). Measured 2.1 cm at the sacrococcygeal level in
/// people without pilonidal disease (Akinci et al. 2009, Colorectal Dis 11:753), deepening caudally;
/// at the anus it is not measured. 3 cm, flagged as an estimate.
const CLEFT_DEPTH: f64 = 3.0;
/// Residual gap of the cleft and the anal margin with pressed cheeks (cm): the channels that skin
/// folds and hair leave around the contact. Not measured; 1 mm. (Where the cheeks meet, beside the
/// verge, pressed cheeks touch: their valve's rest gap is the wedge's alone.)
const CLEFT_GAP: f64 = 0.1;
/// Half-angle of the cleft's wedge when fully spread (rad): the cheek surfaces at a right angle.
const CLEFT_SPREAD_MAX: f64 = PI / 4.0;
/// The cheeks as a second valve. Next to the anal verge the cheeks' surfaces meet (pressed) or
/// nearly meet; the gas pushes them apart and their tissue springs back, as at the lips of a
/// raspberry: a pressure-opened (+,−) valve (Fletcher 1993) of soft tissue, the same LipReed as
/// the sphincter's. Its moving mass is the fat under the perianal skin (below).
/// Length of the strip where the cheeks meet beside the verge, along the flow (cm). Not
/// measured; about the anal margin's width.
const CHEEK_LEN: f64 = 1.0;
/// The space between the verge and that strip (cm): a short soft-walled slot, at least two
/// stable sections at the smallest body size. Not measured.
const MARGIN_LEN: f64 = 1.5;
/// Thickness of the moving layer (cm). Beside the anal verge the skin covers the ischioanal
/// fossa, a fat-filled wedge about 5 cm deep from the perineal skin to the levator ani
/// (textbook figure; not an imaging series), not the gluteal muscle: so the fat's 5 cm. (Over
/// the gluteus maximus, further out, the subcutaneous fat is 1.6 ± 0.4 cm in men and
/// 2.5 ± 0.6 cm in women on CT.)
const CHEEK_LAYER: f64 = 5.0;
/// Young's modulus of subcutaneous fat (dyn/cm²): 3 × the shear modulus 7.5 kPa (Geerligs et al.
/// 2008, Biorheology 45:677, at 10 rad/s and 37 °C), incompressible.
const FAT_YOUNG: f64 = 3.0 * 7.5e4;

/// Mucus between the closed faces of the canal. Where the faces touch, a liquid bridge joins
/// them; while it holds it fills the gap, so no gas passes, and it pulls the parting faces
/// together by Bhattacharya & Siegmund's (2014, Biomech Model Mechanobiol 14:155)
/// traction–separation law for the liquid on colliding vocal folds: the traction rises linearly
/// to the cohesive strength σ₀ at a separation δ₀ = 0.1 mm and falls linearly to nothing at
/// δf = 0.267 mm, where the bridge breaks (their baseline). It forms again at the next contact.
/// σ₀ is a control (0: dry); its upper bound, 3 kPa, is the cohesive strength of intestinal
/// mucus (Mortazavi & Smart 1994), the bound they used.
const ADHESION_D0: f64 = 0.01;
const ADHESION_DF: f64 = 0.0267;
/// dyn/cm² per kPa
const KPA: f64 = 1.0e4;

// ---------------------------------------------------------------- turbulence (calibrated gains)

/// The human voice's glottal aspiration source: gain and band as calibrated there
/// (`voice/human_airway.rs`, `NoiseGains::glottis` and its `.band(3500, 300)`).
const JET_NOISE: (f64, f64, f64) = (3.2e-6, 3500.0, 300.0);
/// The human voice's narrowest-constriction source (`NoiseGains::constriction`, `.band(2600, 350)`).
const CONSTRICTION_NOISE: (f64, f64, f64) = (1.364e-6, 2600.0, 350.0);
/// Below this area (cm²) the cleft counts as a constriction: the human source's own fade
/// (`1 − smoothstep(0.35, 0.9, area)` at body scale 1).
const CONSTRICTION_FADE: (f64, f64) = (0.35, 0.9);

/// The radius of a sphere of volume `v`.
fn sphere_radius(v: f64) -> f64 {
    cbrt((3.0 * v) / (4.0 * PI))
}

/// Pocket geometry for a gas volume: (length, area).
fn pocket_shape(v: f64) -> (f64, f64) {
    let d = cbrt((4.0 * v) / (PI * POCKET_ASPECT));
    let len = POCKET_ASPECT * d;
    (len, v / len)
}

pub struct Flatus {
    net: AirwayNetwork,
    tubes: Vec<(&'static str, Tube)>,
    rectum: Tube,
    canal: Tube,
    cleft: Tube,
    valve: usize,
    lip: LipReed,
    margin: Tube,
    cheek_port: usize,
    cheeks: LipReed,
    margin_areas: Vec<f64>,
    /// the cheeks start at rest at their first control step
    cheeks_settled: bool,
    /// gas pocket volume (cm³) and its equal-volume sphere's radius (cm) and wall speed (cm/s)
    volume: f64,
    radius: f64,
    radius_rate: f64,
    /// the push's pressure on the pocket (dyn/cm²), this control block
    push: f64,
    /// 1/rectal compliance at this body size (dyn/cm² per cm³)
    elastance: f64,
    dt: f64,
    /// a fill is pending (applied at the next control step)
    fill_pending: bool,
    was_pushing: bool,
    /// sums over the current control block: pocket pressure, valve flow, samples
    p_acc: f64,
    u_acc: f64,
    n_acc: f64,
    /// last block's means (telemetry)
    p_mean: f64,
    u_mean: f64,
    /// gas expelled since the last fill (cm³)
    expelled: f64,
    rect_areas: Vec<f64>,
    cleft_areas: Vec<f64>,
    canal_areas: Vec<f64>,
    noise: [NoiseSpec; 2],
    size: f64,
    /// mucus cohesive strength (dyn/cm²) and whether a liquid bridge joins the faces
    adhesion: f64,
    bonded: bool,
}

fn create(fs: f64) -> Box<dyn CreatureSim> {
    let mut net = AirwayNetwork::new(fs);
    let (len0, area0) = pocket_shape(100.0);
    let n_pocket = sections(fs, pocket_shape(POCKET_FLOOR).0, 2.0, 8.0, 1.0);
    // no yielding wall: the pocket's wall is the Rayleigh–Plesset tissue below
    let rectum = tube(&mut net, "rectum", fs, len0, TubeArea::Uniform(area0), TubeOpts { min: Some(n_pocket as f64), max: Some(n_pocket as f64), ..Default::default() });
    let canal_len = HPZ_LEN - VALVE_LEN;
    let n_canal = sections(fs, canal_len * SIZE_MIN, 2.0, 30.0, 1.0);
    let upper = PI * UPPER_CANAL_D * UPPER_CANAL_D / 4.0;
    let canal = tube(&mut net, "canal", fs, canal_len, TubeArea::Uniform(upper), TubeOpts { wall: Some(WALL_SOFT), min: Some(n_canal as f64), max: Some(n_canal as f64), ..Default::default() });
    let n_cleft = sections(fs, CLEFT_DEPTH * SIZE_MIN, 2.0, 30.0, 1.0);
    let cleft = tube(&mut net, "cleft", fs, CLEFT_DEPTH, TubeArea::Uniform(1.0), TubeOpts { wall: Some(WALL_SOFT), min: Some(n_cleft as f64), max: Some(n_cleft as f64), ..Default::default() });
    let n_margin = sections(fs, MARGIN_LEN * SIZE_MIN, 2.0, 30.0, 1.0);
    let margin = tube(&mut net, "margin", fs, MARGIN_LEN, TubeArea::Uniform(0.3), TubeOpts { wall: Some(WALL_SOFT), min: Some(n_margin as f64), max: Some(n_margin as f64), ..Default::default() });
    net.connect(rectum.last_node(), canal.first_node());
    // the distal ring: a slit duct of the lip's height whose gap the LipReed sets every sample;
    // Poiseuille slit resistance over its width, and Ishizaka & Flanagan's (1972) entrance and exit
    // kinetic coefficients (0.37 + 1.0), as at the glottis
    let valve = net.add_port(canal.last_node(), margin.first_node(), PortSpec::new(0.0, VALVE_LEN).kinetic(0.37, 1.0).slit(SLIT_LEN));
    // the cheeks' strip: a slit across the cleft whose gap the cheeks' LipReed sets every sample
    let cheek_port = net.add_port(margin.last_node(), cleft.first_node(), PortSpec::new(0.3, CHEEK_LEN).kinetic(0.37, 1.0).slit(1.0));
    net.add_radiation(cleft.last_node());
    net.finalize();
    // the pocket's wall motion enters as flow (sample by sample, below), so its geometry, which
    // follows at control rate, must not inject −dV/dt a second time; the other tubes change only
    // with body size and cheek spread, slowly
    net.enable_volume_source = false;
    let jet = NoiseSpec::new(valve, margin.branch(0), canal.branch(canal.n - 2), JET_NOISE.0).band(JET_NOISE.1, JET_NOISE.2);
    let mut s = Flatus {
        net,
        tubes: vec![("rectum", rectum), ("canal", canal), ("margin", margin), ("cleft", cleft)],
        rectum,
        canal,
        cleft,
        valve,
        lip: LipReed::new(fs),
        margin,
        cheek_port,
        cheeks: LipReed::new(fs),
        margin_areas: vec![0.3; margin.n],
        cheeks_settled: false,
        volume: 100.0,
        radius: sphere_radius(100.0),
        radius_rate: 0.0,
        push: 0.0,
        elastance: 1.0 / RECTAL_COMPLIANCE,
        dt: 1.0 / fs,
        fill_pending: true,
        was_pushing: false,
        p_acc: 0.0,
        u_acc: 0.0,
        n_acc: 0.0,
        p_mean: 0.0,
        u_mean: 0.0,
        expelled: 0.0,
        rect_areas: vec![area0; rectum.n],
        cleft_areas: vec![1.0; cleft.n],
        canal_areas: vec![upper; canal.n],
        noise: [jet; 2],
        size: 1.0,
        adhesion: 0.0,
        bonded: false,
    };
    s.lip.zeta = LIP_ZETA;
    Box::new(s)
}

impl Flatus {
    /// The rectal recoil pressure at volume `v` (dyn/cm²); compliance scales with body volume.
    fn recoil(&self, v: f64) -> f64 {
        v * self.elastance
    }

    /// Start a new event: the pocket holds `v` cm³ at the pressure its walls put on it (the time
    /// between events, in which the colon refills the rectum, is skipped).
    fn fill(&mut self, v: f64, push: f64) {
        self.volume = js_max(POCKET_FLOOR, v);
        self.radius = sphere_radius(self.volume);
        self.radius_rate = 0.0;
        let (len, area) = pocket_shape(self.volume);
        self.rect_areas.fill(area);
        self.net.set_tube_targets(self.rectum, &self.rect_areas, Some(len), None);
        self.net.snap_to_targets();
        let p = push + self.recoil(self.volume);
        for k in self.rectum.first_node()..=self.canal.last_node() {
            self.net.p[k] = p;
        }
        self.expelled = 0.0;
    }
}

impl CreatureSim for Flatus {
    fn net(&self) -> Option<&AirwayNetwork> {
        Some(&self.net)
    }
    fn net_mut(&mut self) -> Option<&mut AirwayNetwork> {
        Some(&mut self.net)
    }
    fn tubes(&self) -> &[(&'static str, Tube)] {
        &self.tubes
    }
    /// A quarter of the human voice's output gain (5.5e-7): at the voice's scale the loudest
    /// events (a full valve closure at 50+ cmH₂O) reach the output limiter's knee, so the scale is
    /// 12 dB below it. Levels between events stay physical; 12 dB is added back to compare them
    /// with the voice.
    fn gain(&self) -> f64 {
        5.5e-7 / 4.0
    }
    fn control(&mut self, p: &[f64], gate: f64, _dt: f64) {
        let s = js_max(SIZE_MIN, js_min(SIZE_MAX, p[SIZE]));
        self.size = s;
        let push = p[PUSH] * CMH2O * gate;
        // block means of the last control period
        if self.n_acc > 0.0 {
            self.p_mean = self.p_acc / self.n_acc;
            self.u_mean = self.u_acc / self.n_acc;
            self.expelled += self.u_mean * self.n_acc / self.net.fs;
        }
        self.p_acc = 0.0;
        self.u_acc = 0.0;
        self.n_acc = 0.0;
        // a new push after rest starts a new event (the held voice); calls fill with "!fill"
        let pushing = gate > 0.01;
        if pushing && !self.was_pushing {
            self.fill_pending = true;
        }
        self.was_pushing = pushing;
        self.push = push;
        self.elastance = 1.0 / (RECTAL_COMPLIANCE * s * s * s);
        if self.fill_pending {
            self.fill_pending = false;
            self.fill(p[GAS] * s * s * s, push);
            self.p_mean = push + self.recoil(self.volume);
            self.u_mean = 0.0;
        }
        let (len, area) = pocket_shape(self.volume);
        self.rect_areas.fill(area);
        self.net.set_tube_targets(self.rectum, &self.rect_areas, Some(len), None);
        let upper = PI * UPPER_CANAL_D * UPPER_CANAL_D / 4.0 * s * s;
        self.canal_areas.fill(upper);
        self.net.set_tube_targets(self.canal, &self.canal_areas, Some((HPZ_LEN - VALVE_LEN) * s), None);
        // the cleft: a wedge between the cheeks, gap g(z) = g₀ + 2z·tan(θ/2) at depth z from the
        // anus, across the anal verge's width (the sphincter ring's outer diameter)
        let half = js_max(0.0, js_min(1.0, p[SPREAD])) * CLEFT_SPREAD_MAX;
        let depth = CLEFT_DEPTH * s;
        let across = 2.0 * (RING_RADIUS + LIP_THICKNESS / 2.0) * s;
        let n = self.cleft.n;
        let mut narrowest = (0usize, f64::INFINITY);
        for i in 0..n {
            let z = (i as f64 + 0.5) / n as f64 * depth;
            let a = (CLEFT_GAP * s + 2.0 * z * tan(half)) * across;
            self.cleft_areas[i] = a;
            if a < narrowest.1 {
                narrowest = (i, a);
            }
        }
        self.net.set_tube_targets(self.cleft, &self.cleft_areas, Some(depth), None);
        // the cheeks: at rest their strip's gap is the wedge's mean opening over it, so pressed
        // cheeks touch and the gas has to part them; the margin keeps the residual channels. Their
        // layer of fat
        // (over the levator ani) is a mass–spring whose one mass has the layer's first thickness mode,
        // f = c/4L with c = √(E/ρ): stiffness E/L and mass 4ρL/π² per unit area (the gap is
        // twice the surface's displacement). Damping: the layer's Kelvin–Voigt loss with the
        // tissue viscosity above (3μ in extension), ζ = ω·3μ/(2E).
        let strip = CHEEK_LEN * s;
        let rest = strip * tan(half);
        let layer = CHEEK_LAYER * s;
        let k_c = FAT_YOUNG / layer;
        let m_c = 4.0 * TISSUE_DENSITY * layer / (PI * PI);
        let w_c = sqrt(k_c / m_c);
        let ch = &mut self.cheeks;
        ch.f = w_c / (2.0 * PI);
        ch.zeta = w_c * 3.0 * TISSUE_VISCOSITY / (2.0 * FAT_YOUNG);
        ch.surface_per_mass = 2.0 / m_c;
        ch.h0 = rest;
        ch.width = across;
        if !self.cheeks_settled {
            ch.settle();
            self.cheeks_settled = true;
        }
        self.net.set_port_geometry(self.cheek_port, strip, across);
        self.margin_areas.fill((CLEFT_GAP * s + rest) * across);
        self.net.set_tube_targets(self.margin, &self.margin_areas, Some(MARGIN_LEN * s), None);
        // the lip: mass per unit area ρ·δ; stiffness passive + the sphincter's short-range
        // stiffness; at rest the tone presses the ring shut with the closing pressure
        let tone = p[TONE] * CMH2O;
        self.adhesion = js_max(0.0, p[MUCUS]) * KPA;
        let r = RING_RADIUS * s;
        let mass = TISSUE_DENSITY * LIP_THICKNESS * s;
        let k = PASSIVE_STIFFNESS + SRS_GAMMA * tone / r;
        let lip = &mut self.lip;
        lip.f = sqrt(k / mass) / (2.0 * PI);
        // the gap h is twice the ring's radial opening Δr, so h'' = (2/m)·Δp − ω²(h − h₀) and the
        // rest gap is twice the radial preload −P_close/k
        lip.surface_per_mass = 2.0 / mass;
        lip.h0 = -2.0 * tone / k;
        lip.width = SLIT_LEN * s;
        self.net.set_port_geometry(self.valve, VALVE_LEN * s, SLIT_LEN * s);
        // turbulence: the valve's jet, and the cleft where it is narrow
        let mut len = 1;
        let w = 1.0 - smooth(CONSTRICTION_FADE.0, CONSTRICTION_FADE.1, narrowest.1);
        if w > 0.0 {
            let i = narrowest.0.min(n - 2);
            self.noise[1] = NoiseSpec::new(self.cleft.branch(i), self.cleft.branch((i + 1).min(n - 2)), self.cleft.branch(i.saturating_sub(1)), CONSTRICTION_NOISE.0 * w).band(CONSTRICTION_NOISE.1, CONSTRICTION_NOISE.2);
            len = 2;
        }
        self.net.set_noise_sources(&self.noise[..len]);
    }
    #[inline]
    fn sample(&mut self) -> f64 {
        let p = &self.net.p;
        let dp = p[self.canal.last_node()] - p[self.margin.first_node()];
        let dpc = p[self.margin.last_node()] - p[self.cleft.first_node()];
        let mut pr = 0.0;
        for k in self.rectum.first_node()..=self.rectum.last_node() {
            pr += p[k];
        }
        let p_gas = pr / self.rectum.n as f64;
        self.p_acc += p_gas;
        // Rayleigh–Plesset for the pocket's wall (semi-implicit Euler); the walls meet at the floor
        let (r, rd) = (self.radius, self.radius_rate);
        let p_out = self.push + self.volume * self.elastance;
        let rdd = ((p_gas - p_out - 4.0 * TISSUE_VISCOSITY * rd / r) / TISSUE_DENSITY - 1.5 * rd * rd) / r;
        let mut rd = rd + rdd * self.dt;
        let mut r = r + rd * self.dt;
        let r_floor = sphere_radius(POCKET_FLOOR);
        if r < r_floor {
            r = r_floor;
            if rd < 0.0 {
                rd = 0.0;
            }
        }
        let v = 4.0 / 3.0 * PI * r * r * r;
        // the volume the wall gives up is pushed into the gas, shared by the pocket's sections
        let inflow = (self.volume - v) / self.dt / self.rectum.n as f64;
        for k in self.rectum.first_node()..=self.rectum.last_node() {
            self.net.ext_flow[k] += inflow;
        }
        self.volume = v;
        self.radius = r;
        self.radius_rate = rd;
        // the mucus bridge: formed at contact, broken past δf; while it holds it seals the gap and
        // pulls the parting faces together
        let traction = if self.bonded {
            let gap = self.lip.opening / self.lip.width;
            if gap <= ADHESION_D0 { self.adhesion * gap / ADHESION_D0 } else { self.adhesion * (ADHESION_DF - gap) / (ADHESION_DF - ADHESION_D0) }
        } else {
            0.0
        };
        let open = self.lip.next(dp - traction);
        let gap = open / self.lip.width;
        if open <= 0.0 {
            self.bonded = self.adhesion > 0.0;
        } else if gap >= ADHESION_DF {
            self.bonded = false;
        }
        self.net.set_port_area(self.valve, if self.bonded { 0.0 } else { open });
        let cheek_open = self.cheeks.next(dpc);
        self.net.set_port_area(self.cheek_port, cheek_open);
        let y = self.net.step();
        self.u_acc += self.net.u[self.valve];
        self.n_acc += 1.0;
        y
    }
    fn source_state(&self, out: &mut Vec<(&'static str, f64)>) {
        out.clear();
        // the glyph: the gap against a 1 mm reference
        out.push(("sphincter", js_min(1.0, self.lip.opening / (self.lip.width * 0.1))));
        out.push(("gap", self.lip.opening / self.lip.width));
        out.push(("cheeks", self.cheeks.opening / self.cheeks.width));
        out.push(("pressure", self.p_mean / CMH2O));
        out.push(("gas", self.volume));
        out.push(("flow", self.u_mean));
        out.push(("expelled", self.expelled));
    }
    fn trigger(&mut self, _event: usize, _amount: f64) {
        self.fill_pending = true;
    }
}

fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
    crate::math::smoothstep(e0, e1, x)
}

/// Five pushes of 0.15 s, 0.25 s apart, on one pocket of gas: the push (a parameter, smoothed like
/// the muscle) goes on and off while the gate stays open.
fn staccato() -> Vec<CallKey> {
    let k = CallKey::new;
    let mut keys = vec![k(0.0, &[("gate", 0.0), ("gas", 200.0), ("tone", 20.0), ("push", 0.0), ("spread", 0.2)]), k(0.1, &[("!fill", 1.0), ("gate", 0.0)]), k(0.12, &[("gate", 1.0)])];
    for i in 0..5 {
        let t = 0.15 + 0.4 * i as f64;
        keys.push(k(t, &[("push", 0.0)]));
        keys.push(k(t + 0.01, &[("push", 30.0)]));
        keys.push(k(t + 0.15, &[("push", 30.0)]));
        keys.push(k(t + 0.16, &[("push", 0.0)]));
    }
    keys.push(k(2.2, &[("gate", 1.0)]));
    keys.push(k(2.3, &[("gate", 0.0)]));
    keys.push(k(2.8, &[("gate", 0.0)]));
    keys
}

pub fn def() -> CreatureDef {
    let k = CallKey::new;
    CreatureDef {
        id: "flatulence",
        name: "Flatulence",
        latin: "Homo sapiens, flatus ani",
        group: CreatureGroup::Human,
        blurb: "Rectal gas pushed by the abdomen sets the anal sphincter's outer ring buzzing like a brass player's lips; the gas pocket empties as it plays, and the cleft between the cheeks shapes the sound.",
        role: "Squeaks to rumbles, from push, sphincter tone, gas and cheek spread",
        oversample: 2,
        // the push is the abdominal muscles: force builds and relaxes over about 0.1 s (twitch
        // contraction times of 50–100 ms; an estimate, not measured for straining)
        gate_times: Some(GateTimes { attack: 0.05, release: 0.08 }),
        params: vec![
            // intrarectal pressure added by straining: 20 ± 3 mmHg (10th–90th 0.7–47) during
            // simulated evacuation in healthy women under 50 (Noelting et al. 2012)
            ParamDef::new("push", "Abdominal push", 0.0, 80.0, 30.0).unit("cmH₂O").smooth(0.05),
            // the anal canal's closing pressure: resting 32–133 cmH₂O (median 67), squeeze up to
            // 252 (Luft et al. 2012); passing gas means relaxing below the push
            ParamDef::new("tone", "Sphincter tone", 2.0, 150.0, 20.0).unit("cmH₂O").smooth(0.05),
            // gas in the rectum at the start of an event: 476–1491 mL a day in 24-h collections
            // (Tomlin, Lowis & Read 1991, Gut 32:665), passed in some ten to twenty events
            ParamDef::new("gas", "Gas volume", 10.0, 300.0, 100.0).unit("mL").log().smooth(0.01),
            ParamDef::new("spread", "Cheek spread", 0.0, 1.0, 0.2).smooth(0.3),
            ParamDef::new("size", "Body size", SIZE_MIN, SIZE_MAX, 1.0).smooth(0.3),
            // the cohesive strength of the mucus between the canal's faces: 0 (dry) to 3 kPa,
            // intestinal mucus (Mortazavi & Smart 1994, via Bhattacharya & Siegmund 2014)
            ParamDef::new("mucus", "Mucus adhesion", 0.0, 3.0, 0.0).unit("kPa").smooth(0.05),
        ],
        // Every call sets its controls first, lets them settle (the gas control smooths over 10
        // ms), starts a new event with "!fill" at 0.1 s, then pushes, and holds its controls for
        // 0.5 s after the push ends while the pocket's pressure falls back: calls differ only in
        // the physical controls.
        calls: vec![
            CallDef::new(
                "squeak",
                "Squeak",
                "Little gas, a tight sphincter, cheeks together: the push only just opens it",
                None,
                vec![k(0.0, &[("gate", 0.0), ("gas", 20.0), ("tone", 40.0), ("push", 42.0), ("spread", 0.0)]), k(0.1, &[("!fill", 1.0), ("gate", 0.0)]), k(0.12, &[("gate", 1.0)]), k(1.3, &[("gate", 1.0)]), k(1.4, &[("gate", 0.0)]), k(1.9, &[("gate", 0.0)])],
            ),
            CallDef::new(
                "rumble",
                "Rumble",
                "Plenty of gas, a relaxed sphincter, a gentle steady push",
                None,
                vec![k(0.0, &[("gate", 0.0), ("gas", 300.0), ("tone", 5.0), ("push", 8.0), ("spread", 0.3)]), k(0.1, &[("!fill", 1.0), ("gate", 0.0)]), k(0.12, &[("gate", 1.0)]), k(3.1, &[("gate", 1.0)]), k(3.2, &[("gate", 0.0)]), k(3.7, &[("gate", 0.0)])],
            ),
            CallDef::new("staccato", "Staccato", "One pocket of gas let out in five pushes", None, staccato()),
            CallDef::new(
                "leak",
                "Slow leak",
                "Relaxed, cheeks spread, a gentle push",
                None,
                vec![k(0.0, &[("gate", 0.0), ("gas", 80.0), ("tone", 4.0), ("push", 4.0), ("spread", 1.0)]), k(0.1, &[("!fill", 1.0), ("gate", 0.0)]), k(0.12, &[("gate", 1.0)]), k(3.1, &[("gate", 1.0)]), k(3.2, &[("gate", 0.0)]), k(3.7, &[("gate", 0.0)])],
            ),
        ],
        events: vec!["fill"],
        create,
    }
    .resolved()
}
