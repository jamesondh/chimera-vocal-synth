//! The limiter on the sum of several voices: the plugin's chords and the web app's
//! (`crates/chimera-plugin/src/core.rs`, `src/worklet/processor.ts`).
use crate::math::{exp, tanh};

/// The limiter on the sum of several voices. Each voice ends in one (`HumanVoice::render`: a loud
/// vowel sits at 0.75), so a chord's peaks add up to well over full scale: 1.5 for two voices at
/// full pressure, 2.4 for four. This is the same law on the sum: a peak envelope held for longer
/// than a glottal period, so a loud chord is turned down by a steady gain and not squared off, and
/// a soft clip for what rises faster. Below its ceiling it does nothing.
pub struct ChordLimiter {
    env: f64,
    hold: usize,
    hold_n: usize,
    release: f64,
}

impl ChordLimiter {
    const CEILING: f64 = 0.75;
    const KNEE: f64 = 0.85;

    pub fn new(sample_rate: f64) -> ChordLimiter {
        ChordLimiter { env: 0.0, hold: 0, hold_n: (0.025 * sample_rate).round() as usize, release: exp(-1.0 / (0.2 * sample_rate)) }
    }

    pub fn process(&mut self, out: &mut [f32]) {
        for o in out {
            let mut v = *o as f64;
            let av = v.abs();
            if av >= self.env {
                self.env = av;
                self.hold = self.hold_n;
            } else if self.hold > 0 {
                self.hold -= 1;
            } else {
                self.env *= self.release;
                // nothing is nothing: no denormals in a silence
                if self.env < 1e-20 {
                    self.env = 0.0;
                }
            }
            if self.env > Self::CEILING {
                v *= Self::CEILING / self.env;
            }
            let al = v.abs();
            if al > Self::KNEE {
                v = v.signum() * (Self::KNEE + (1.0 - Self::KNEE) * tanh((al - Self::KNEE) / (1.0 - Self::KNEE)));
            }
            *o = v as f32;
        }
    }

    /// Forget the level of what came before (one voice again: it passes unchanged at once).
    pub fn reset(&mut self) {
        self.env = 0.0;
        self.hold = 0;
    }
}
