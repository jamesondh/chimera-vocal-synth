//! Ventricular (false) vocal folds as a valve phase-locked to the true folds: kargyraa.
//!
//! In kargyraa the ventricular folds, about a centimetre above the glottis, vibrate at half the
//! vocal folds' rate and close once every two glottal cycles (Bailly, Henrich & Pelorson 2010).
//! Every other glottal pulse meets a nearly shut valve and is choked, so the radiated sound
//! repeats every two cycles: its pitch is an octave below the folds', and the timbre is rough
//! (the jet through the narrow valve is turbulent).
//!
//! Like the parametric glottis, this is an area waveform, not a self-oscillating mass: the
//! valve's area follows the glottal cycle, so the subharmonic is exact. The airway does the rest
//! (the valve caps the minimum area of the tract node at the ventricular folds, which sets the
//! jet's Borda–Carnot loss and its turbulence).

use crate::math::{PI, cos, exp};

#[derive(Clone, Debug)]
pub struct VentricularFolds {
    /// target depth 0..1: how far the valve closes on its closing cycle
    pub depth: f64,
    depth_sm: f64,
    sm_coef: f64,
}

/// Centre of the closure within the two-cycle period (0..1): the second cycle's open phase
/// (glottal pulses rise over the first ~30 % of a cycle), so that cycle's pulse is the one choked.
const CLOSE_AT: f64 = 0.65;
/// Half-width of the closure (fraction of the two-cycle period).
const HALF_WIDTH: f64 = 0.25;
/// The narrowest the valve gets, as a fraction of its open area: it chokes the jet but never
/// seals (a seal stops the flow within a sample, a click).
const FLOOR: f64 = 0.01;

impl VentricularFolds {
    pub fn new(fs: f64) -> Self {
        VentricularFolds { depth: 0.0, depth_sm: 0.0, sm_coef: 1.0 - exp(-1.0 / (0.02 * fs)) }
    }

    /// Whether the valve is (still) acting: false once it has relaxed fully open.
    #[inline]
    pub fn active(&self) -> bool {
        self.depth > 0.0 || self.depth_sm > 1e-5
    }

    /// Area scale (FLOOR..1) for this sample, from the glottal cycle phase (0..1) and whether
    /// this is an odd glottal cycle.
    #[inline]
    pub fn next(&mut self, glottal_phase: f64, odd_cycle: bool) -> f64 {
        self.depth_sm += self.sm_coef * (self.depth - self.depth_sm);
        let phi = 0.5 * (glottal_phase + if odd_cycle { 1.0 } else { 0.0 });
        let d = (phi - CLOSE_AT).abs();
        let g = if d < HALF_WIDTH { 0.5 + 0.5 * cos(PI * d / HALF_WIDTH) } else { 0.0 };
        let s = 1.0 - self.depth_sm * g;
        if s < FLOOR { FLOOR } else { s }
    }
}
