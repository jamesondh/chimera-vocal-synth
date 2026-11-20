//! Domestic cat: meows from ordinary vocal-fold vibration (about 400–1200 Hz: Nicastro 2004),
//! their "au" made by the jaw moving F1 over a short tract (Ekström et al. 2024); hisses of breath
//! forced through a narrowed mouth; growls and snarls (Schötz 2015); and a purr, about 25 Hz, that
//! sounds on the out-breath and the longer in-breath (Schötz & Eklund 2011).
//!
//! How the purr is driven is still argued: laryngeal muscles closing the glottis at the purr rate,
//! the sound coming as it snaps open (Remmers & Gautier 1972), or the folds oscillating on their
//! own, like human creaky voice (Herbst et al. 2023). The ordinary glottis source only vibrates
//! under positive pressure, so the purr follows the first account: a valve gated at the purr rate
//! whatever the airflow's direction, which lets the in-breath purr too.
//!
//! The airway's lengths and the trachea are measured (the constants below say by whom); its
//! cross-sections, the nose and the purr's glottis are estimates. The airway is short, so the
//! creature asks for a 64 kHz internal rate on slow devices ([`CreatureDef::min_rate`]).

use super::build::{CMH2O, fill_areas};
use super::mammal::{MammalAirway, MammalSpec, mammal_airway};
use super::types::{CallDef, CallKey, CreatureDef, CreatureGroup, CreatureSim, Curve, GateTimes, ParamDef};
use crate::airway::network::{AirwayNetwork, NoiseSpec, Tube};
use crate::math::{js_max, js_min};

const PITCH: usize = 0;
const MOUTH: usize = 1;
const BRIGHT: usize = 2;
const ROUGH: usize = 3;
const PRESSURE: usize = 4;
const PURR: usize = 5;
const PURR_RATE: usize = 6;
const BREATH: usize = 7;
const HISS: usize = 8;

// The airway, from published measurements where there are any (see `docs/architecture.md`):
/// trachea: 125 mm long, lumen 8.9 × 8.0 mm (ellipse), CT of 15 cats of 4.9 kg (Zimmermann et al.
/// 2023, J Feline Med Surg)
const TRACHEA: (f64, f64) = (12.5, 0.56);
/// glottis to lips 80 mm in five adult cats (Weissengruber et al. 2002, J Anat, Table 2); the oral
/// part of the pharynx is under 1 cm, so the larynx sits high: 2 cm of pharynx, 6 cm of mouth
const PHARYNX_LEN: f64 = 2.0;
const ORAL_LEN: f64 = 6.0;
/// a uniform 2 cm² tube, as in Ekström et al.'s cat tube model (2024; an assumption there too:
/// no cross-sections of the cat tract are published)
const TRACT_AREA: f64 = 2.0;
/// the jaw shapes the front 3 cm: the buccolabial length is 2.8–3.3 cm in five cats (Ekström et
/// al. 2024, Table 2). Narrowed, the front 1 cm is 0.2 cm² (their model); open, it flares.
const LIP_LEN: f64 = 3.0;
const NARROW_LEN: f64 = 1.0;
const NARROW_AREA: f64 = 0.2;
/// how far the open mouth flares at the lips (×TRACT_AREA), set so the open tract's F1 matches
/// their model's 1305 Hz
const FLARE: f64 = 5.0;
/// No feline nasal tract has been described well enough to model (Ekström et al. 2024): an estimate.
const NASAL: (f64, f64) = (4.5, 0.3);
/// peak glottal area (cm²) of a meow: the folds' membranous length is 7.5 mm (Herbst et al. 2023),
/// about half a human's, so the peak area is about half a human's
const FOLD_AREA: f64 = 0.07;
/// The purr's glottis between closures (cm²): the cat breathes through it, so it is a breathing
/// opening, not a voice pulse. An estimate, about half the trachea's lumen.
const PURR_AREA: f64 = 0.25;
/// The hiss's noise band (Hz) and level (gain on the turbulence source at the narrowed mouth).
const HISS_LOWPASS: f64 = 1000.0;
const HISS_GAIN: f64 = 1.6e-3;

/// The purr cycle, after Remmers & Gautier (1972), whose tracheal pressures showed the glottis
/// closing once per cycle and the sound coming from its sudden reopening: open, then easing shut
/// (PURR_SHUT..PURR_HELD), held shut while pressure builds, then snapping open (PURR_SNAP, about
/// 0.5 ms at 25 Hz). The fractions are estimates: no glottal imaging of purring is published.
const PURR_SHUT: f64 = 0.45;
const PURR_HELD: f64 = 0.75;
const PURR_OPEN_AT: f64 = 0.9;
const PURR_SNAP: f64 = 0.0125;

pub struct Cat {
    a: MammalAirway,
    tubes: Vec<(&'static str, Tube)>,
    velum: usize,
    fs: f64,
    /// purr cycle phase 0..1, its increment per sample, and the meow ↔ purr mix
    purr_phase: f64,
    purr_inc: f64,
    purr_mix: f64,
    /// the purr valve's last area (for the source glyph)
    purr_area: f64,
    /// scale on every noise gain, fs/96000: the airway's turbulence noise is scaled by √(96000/fs)
    /// (`AirwayNetwork::compute_noise`), which looks inverted for keeping the noise power per Hz
    /// constant (that would need √(fs/96000)); theory puts the error at 1.5× in amplitude between
    /// 64 and 96 kHz. Measured on this cat's hiss (RMS amplitude): 0.034 at a 64 kHz internal rate
    /// against 0.017 at 96 kHz unscaled (2.0×; the noise feeds back on the flow's Reynolds number, so
    /// the response isn't linear), 0.0169 against 0.0167 with this scale. 88.2 kHz stays about 1.6×
    /// loud, from where the narrowed mouth falls on the section grid. The network is shared with
    /// the human voice, so it is left alone here.
    noise_rate: f64,
}

fn create(fs: f64) -> Box<dyn CreatureSim> {
    Box::new(build(fs))
}

fn build(fs: f64) -> Cat {
    let a = mammal_airway(
        fs,
        MammalSpec { trachea: TRACHEA, pharynx: (PHARYNX_LEN, TRACT_AREA), oral: (ORAL_LEN, TRACT_AREA), nasal: Some(NASAL), wall: None, wall_radiation: Some(0.3), coarse: None, nasal_radiates: None },
        |_, _| {},
    );
    let velum = a.velum.expect("the cat has a velum");
    let tubes = vec![("trachea", a.trachea), ("pharynx", a.pharynx), ("oral", a.oral)];
    let mut c = Cat { a, tubes, velum, fs, purr_phase: 0.0, purr_inc: 0.0, purr_mix: 0.0, purr_area: 0.0, noise_rate: fs / 96000.0 };
    let g = &mut c.a.glottis_src.params;
    g.jitter = 0.004;
    g.shimmer = 0.03;
    g.drift = 6.0;
    c
}

/// The mouth's area at position u (0 back … 1 lips) for a jaw opening 0..1: the jaw moves F1, which
/// carries the meow's vowel-like "au" (Ekström et al. 2024).
fn oral_area(u: f64, mouth: f64) -> f64 {
    let x = (1.0 - u) * ORAL_LEN; // cm back from the lips
    let narrowed = if x < NARROW_LEN { NARROW_AREA } else { TRACT_AREA };
    let s = js_max(0.0, 1.0 - x / LIP_LEN);
    let open = TRACT_AREA * (1.0 + FLARE * s * s);
    narrowed + (open - narrowed) * mouth
}

/// The glottis over one purr cycle (0 shut … 1 open). Polynomial: no transcendental per sample.
#[inline]
fn purr_pulse(phase: f64) -> f64 {
    if phase < PURR_SHUT {
        1.0
    } else if phase < PURR_HELD {
        let r = 1.0 - (phase - PURR_SHUT) / (PURR_HELD - PURR_SHUT);
        r * r * (3.0 - 2.0 * r)
    } else if phase < PURR_OPEN_AT {
        0.0
    } else {
        js_min(1.0, (phase - PURR_OPEN_AT) / PURR_SNAP)
    }
}

impl CreatureSim for Cat {
    fn net(&self) -> Option<&AirwayNetwork> {
        Some(&self.a.net)
    }
    fn net_mut(&mut self) -> Option<&mut AirwayNetwork> {
        Some(&mut self.a.net)
    }
    fn tubes(&self) -> &[(&'static str, Tube)] {
        &self.tubes
    }
    fn gain(&self) -> f64 {
        8.0e-8
    }
    fn control(&mut self, p: &[f64], gate: f64, _dt: f64) {
        let a = &mut self.a;
        let purr = p[PURR];
        let hiss = p[HISS];
        self.purr_mix = purr;
        self.purr_inc = p[PURR_RATE] / self.fs;
        // breath < 0 breathes in: the lungs pull, and only the purr valve still sounds
        a.net.set_fixed_pressure(a.lungs, p[PRESSURE] * CMH2O * gate * p[BREATH]);
        let g = &mut a.glottis_src.params;
        g.f0 = p[PITCH];
        g.amplitude = FOLD_AREA * (1.0 - hiss);
        g.abduction = 0.2 * hiss; // hissing: the folds part and the air goes through silent
        g.open_quotient = 0.6 - 0.25 * p[BRIGHT];
        g.closure_sharpness = 0.3 + 0.6 * p[BRIGHT];
        g.effort = 0.4 + 0.5 * p[BRIGHT];
        g.chink = 0.004;
        g.chaos = p[ROUGH];
        let mouth = p[MOUTH];
        // a purring cat breathes through its nose; a meow starts nasal ("m") while the jaw is shut
        // and the air moves to the mouth as it opens (Ekström et al. 2024)
        a.net.set_port_target(self.velum, 0.01 + 0.4 * purr + 0.12 * (1.0 - mouth) * (1.0 - purr));
        let ph_areas = fill_areas(a.pharynx, |_| TRACT_AREA, &mut a.phar_area);
        a.net.set_tube_targets(a.pharynx, ph_areas, None, None);
        let or_areas = fill_areas(a.oral, |u| oral_area(u, mouth), &mut a.oral_area);
        a.net.set_tube_targets(a.oral, or_areas, None, None);
        // breath noise at the glottis, and the hiss where the mouth is narrowest (breath forced
        // through the slightly open mouth: Moelk 1944), low-banded so its centre of gravity lands
        // near the 1.1 kHz of four cats' hisses (Schötz 2015)
        let glottal = NoiseSpec::new(a.glottis, a.pharynx.branch(1.min(a.pharynx.n - 2)), a.trachea.branch(a.trachea.n - 2), 1.5e-6 * self.noise_rate).band(4000.0, 250.0);
        let (mut mi, mut mv) = (0usize, f64::INFINITY);
        for i in 0..a.oral.n - 1 {
            if a.oral_area[i] < mv {
                mv = a.oral_area[i];
                mi = i;
            }
        }
        let oral = NoiseSpec::new(a.oral.branch(mi), a.oral.branch((a.oral.n - 2).min(mi + 1)), a.oral.branch(mi.saturating_sub(1)), (2e-6 + HISS_GAIN * hiss) * self.noise_rate).band(HISS_LOWPASS, 400.0);
        a.net.set_noise_sources(&[glottal, oral]);
    }
    #[inline]
    fn sample(&mut self) -> f64 {
        let a = &mut self.a;
        let dp = a.net.p[a.trachea.last_node()] - a.net.p[a.pharynx.first_node()];
        let folds = a.glottis_src.next(dp);
        self.purr_phase += self.purr_inc;
        if self.purr_phase >= 1.0 {
            self.purr_phase -= 1.0;
        }
        self.purr_area = 0.003 + PURR_AREA * purr_pulse(self.purr_phase);
        let mix = self.purr_mix;
        a.net.set_port_area(a.glottis, (1.0 - mix) * folds + mix * self.purr_area);
        a.net.step()
    }
    fn flush_tiny(&mut self) {
        self.a.glottis_src.flush_tiny();
    }
    fn source_state(&self, out: &mut Vec<(&'static str, f64)>) {
        out.clear();
        let mix = self.purr_mix;
        let area = (1.0 - mix) * self.a.glottis_src.area + mix * self.purr_area;
        out.push(("glottis", js_min(1.0, js_max(0.0, area / 0.08))));
    }
}

/// Purr phases: breathing out 585 ms, in 652 ms, the means of four cats (Schötz & Eklund 2011;
/// the in-breath was the longer in every cat).
const PURR_OUT: f64 = 0.585;
const PURR_IN: f64 = 0.652;
/// the in-breath's strength (breath param), set so it purrs about 2 dB below the out-breath, the
/// four cats' mean difference (Schötz & Eklund 2011)
const PURR_IN_BREATH: f64 = -0.8;

fn purr_keys() -> Vec<CallKey> {
    let k = CallKey::new;
    let mut keys = vec![k(0.0, &[("gate", 1.0), ("purr", 1.0), ("mouth", 0.0), ("hiss", 0.0), ("pressure", 9.0), ("breath", 1.0)])];
    let mut t = 0.0;
    for _ in 0..4 {
        if t > 0.0 {
            keys.push(k(t, &[("breath", 1.0)]).curve(Curve::Step));
        }
        keys.push(k(t + PURR_OUT, &[("breath", PURR_IN_BREATH)]).curve(Curve::Step));
        t += PURR_OUT + PURR_IN;
    }
    keys.push(k(t, &[("gate", 0.0)]).curve(Curve::Step));
    keys
}

pub fn def() -> CreatureDef {
    let k = CallKey::new;
    CreatureDef {
        id: "cat",
        name: "Domestic cat",
        latin: "Felis catus",
        group: CreatureGroup::Mammal,
        blurb: "Meows from vibrating vocal folds and a jaw that opens and closes; a 25 Hz purr that keeps going as it breathes in.",
        role: "Expressive leads, sub-bass purr pulses, noise sweeps",
        oversample: 2,
        min_rate: Some(64000.0),
        gate_times: Some(GateTimes { attack: 0.012, release: 0.05 }),
        params: vec![
            ParamDef::new("pitch", "Pitch", 40.0, 2400.0, 650.0).unit("Hz").log().smooth(0.015).pitch(),
            ParamDef::new("mouth", "Jaw opening", 0.0, 1.0, 0.6).smooth(0.03),
            ParamDef::new("bright", "Brightness", 0.0, 1.0, 0.5).smooth(0.02),
            ParamDef::new("rough", "Roughness", 0.0, 0.8, 0.03).smooth(0.05),
            ParamDef::new("pressure", "Breath pressure", 1.0, 20.0, 8.0).unit("cmH₂O").smooth(0.01),
            ParamDef::new("purr", "Meow ↔ purr", 0.0, 1.0, 0.0).smooth(0.04),
            ParamDef::new("purrRate", "Purr rate", 18.0, 35.0, 25.0).unit("Hz").smooth(0.05),
            ParamDef::new("breath", "Breath in ↔ out", -1.0, 1.0, 1.0).smooth(0.08),
            ParamDef::new("hiss", "Hiss", 0.0, 1.0, 0.0).smooth(0.03),
        ],
        calls: vec![
            CallDef::new(
                "meow",
                "Meow",
                "The jaw opens and closes over a rise and fall: i-a-u",
                Some(700.0),
                vec![
                    k(0.0, &[("gate", 1.0), ("pitch", 560.0), ("mouth", 0.05), ("purr", 0.0), ("breath", 1.0), ("hiss", 0.0)]),
                    k(0.18, &[("pitch", 760.0), ("mouth", 0.9)]).curve(Curve::Exp),
                    k(0.45, &[("pitch", 700.0), ("mouth", 0.8)]).curve(Curve::Exp),
                    k(0.72, &[("gate", 1.0), ("pitch", 480.0), ("mouth", 0.05)]).curve(Curve::Exp),
                    k(0.78, &[("gate", 0.0)]),
                ],
            ),
            CallDef::new(
                "mew",
                "Kitten mew",
                "Short and two to three times higher",
                Some(1600.0),
                vec![
                    k(0.0, &[("gate", 1.0), ("pitch", 1400.0), ("mouth", 0.1), ("purr", 0.0), ("breath", 1.0), ("hiss", 0.0), ("bright", 0.7)]),
                    k(0.1, &[("pitch", 1700.0), ("mouth", 0.75)]).curve(Curve::Exp),
                    k(0.32, &[("gate", 1.0), ("pitch", 1250.0), ("mouth", 0.1)]).curve(Curve::Exp),
                    k(0.36, &[("gate", 0.0)]),
                ],
            ),
            CallDef::new(
                "purr",
                "Purr",
                "Four breaths, purring out and in, mouth shut",
                None,
                purr_keys(),
            ),
            CallDef::new(
                "trill",
                "Trill",
                "A rolled, rising greeting with the mouth almost shut",
                Some(600.0),
                vec![
                    k(0.0, &[("gate", 1.0), ("pitch", 420.0), ("mouth", 0.08), ("purr", 0.0), ("breath", 1.0), ("hiss", 0.0), ("rough", 0.35)]),
                    k(0.22, &[("gate", 1.0), ("pitch", 640.0), ("mouth", 0.2)]).curve(Curve::Exp),
                    k(0.26, &[("gate", 0.0)]),
                ],
            ),
            CallDef::new(
                "hiss",
                "Hiss",
                "Air forced out through a narrowed mouth, no voice",
                None,
                vec![
                    k(0.0, &[("gate", 1.0), ("hiss", 1.0), ("purr", 0.0), ("breath", 1.0), ("mouth", 0.0), ("pressure", 14.0)]),
                    k(0.62, &[("gate", 1.0), ("pressure", 10.0)]),
                    k(0.7, &[("gate", 0.0)]),
                ],
            ),
            CallDef::new(
                "growl",
                "Growl",
                "Low, rough and long, on a slow breath out, mouth slightly open",
                Some(140.0),
                vec![
                    k(0.0, &[("gate", 1.0), ("pitch", 140.0), ("rough", 0.6), ("mouth", 0.25), ("bright", 0.8), ("purr", 0.0), ("breath", 1.0), ("hiss", 0.0), ("pressure", 10.0)]),
                    k(2.4, &[("gate", 1.0), ("pitch", 115.0)]),
                    k(2.5, &[("gate", 0.0)]),
                ],
            ),
            CallDef::new(
                "snarl",
                "Snarl",
                "Short, harsh and loud, mouth open on an \"a\"",
                Some(460.0),
                vec![
                    k(0.0, &[("gate", 1.0), ("pitch", 430.0), ("rough", 0.35), ("mouth", 0.85), ("bright", 0.9), ("purr", 0.0), ("breath", 1.0), ("hiss", 0.0), ("pressure", 14.0)]),
                    k(0.12, &[("pitch", 500.0)]).curve(Curve::Exp),
                    k(0.36, &[("gate", 1.0), ("pitch", 440.0)]).curve(Curve::Exp),
                    k(0.4, &[("gate", 0.0)]),
                ],
            ),
        ],
        events: vec![],
        create,
    }
    .resolved()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airway::frequency::{find_peaks, linspace, transfer_function};

    /// F1–F3 of the cat's tract (glottis to lips, nose shut, as in the tube model) for a jaw opening.
    fn formants(mouth: f64) -> Vec<f64> {
        let mut cat = build(96000.0);
        let mut p: Vec<f64> = def().params.iter().map(|q| q.default).collect();
        p[MOUTH] = mouth;
        cat.control(&p, 0.0, 0.001);
        cat.a.net.set_port_target(cat.velum, 0.0);
        let el = cat.a.net.linear_elements();
        let fr = linspace(50.0, 7000.0, 2800);
        let h = transfer_function(&el, cat.a.pharynx.first_node(), &fr, &[cat.a.glottis]);
        find_peaks(&fr, &h.mag, 300.0).into_iter().take(3).map(|p| p.freq).collect()
    }

    #[test]
    fn the_jaw_moves_f1_as_the_tube_model_predicts() {
        // Ekström et al. 2024, Table 3: an 8 cm cat tract narrowed at the lips has F1 602, F2 2675,
        // F3 5105 Hz; with the jaw open (flared) F1 is 1305 Hz. Only F1 is compared open: their
        // flare is a shortened-tube approximation, and F1 is what carries the "au".
        let (closed, open) = (formants(0.0), formants(1.0));
        for (got, want) in closed.iter().zip([602.0, 2675.0, 5105.0]) {
            assert!((got / want - 1.0).abs() < 0.06, "narrowed: {closed:?}, model 602/2675/5105 Hz");
        }
        assert!((open[0] / 1305.0 - 1.0).abs() < 0.03, "open F1 {:.0} Hz, model 1305", open[0]);
        let mid = formants(0.5)[0];
        assert!(closed[0] < mid && mid < open[0], "F1 rises as the jaw opens: {:.0} {mid:.0} {:.0}", closed[0], open[0]);
    }
}
