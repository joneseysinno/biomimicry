//! Target 3: detect ringing vs. converging.
//!
//! Sensing/compare are milli-ready. [`AttractorStabilityLoop::effect`] is an
//! honest **no-op brake** (records last error) until a future effector design —
//! not wired into default [`crate::organism::Organism::settle`].

use crate::error::Result;
use crate::homeostasis::HomeostaticLoop;

/// Homeostatic loop that stabilizes attractor dynamics (anti-ringing).
#[derive(Debug, Clone, Default)]
pub struct AttractorStabilityLoop {
    /// Target residual oscillation amplitude (milli-units).
    pub target_amplitude: i64,
    /// Measured residual amplitude (milli-units).
    pub current_amplitude: i64,
    /// Last error passed to [`Self::effect`] (milli-units).
    pub last_error: i64,
}

impl AttractorStabilityLoop {
    /// Create a loop with the given amplitude set-point.
    #[must_use]
    pub fn new(target_amplitude: i64) -> Self {
        Self {
            target_amplitude,
            current_amplitude: 0,
            last_error: 0,
        }
    }
}

impl HomeostaticLoop for AttractorStabilityLoop {
    type Measurement = i64;

    fn set_point(&self) -> Self::Measurement {
        self.target_amplitude
    }

    fn sense(&self) -> Self::Measurement {
        self.current_amplitude
    }

    fn compare(&self, measured: &Self::Measurement) -> i64 {
        self.target_amplitude - *measured
    }

    fn effect(&mut self, error: i64) -> Result<()> {
        self.last_error = error;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a5_effect_is_noop_ok() {
        let mut loop_ = AttractorStabilityLoop::new(0);
        loop_.effect(-7).expect("ok");
        assert_eq!(loop_.last_error, -7);
    }
}
