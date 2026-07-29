//! Target 2: receptor downregulation / lateral inhibition.
//!
//! Sensing/compare are milli-ready. [`SignalFluxLoop::effect`] is an honest
//! **no-op brake** (records last error) until a future corrective-step design —
//! not wired into default [`crate::organism::Organism::settle`].

use crate::error::Result;
use crate::homeostasis::HomeostaticLoop;

/// Homeostatic loop over inbound signal flux.
#[derive(Debug, Clone, Default)]
pub struct SignalFluxLoop {
    /// Target flux (milli-units).
    pub target: i64,
    /// Measured flux (milli-units).
    pub current: i64,
    /// Last error passed to [`Self::effect`] (milli-units).
    pub last_error: i64,
}

impl SignalFluxLoop {
    /// Create a loop with the given set-point.
    #[must_use]
    pub fn new(target: i64) -> Self {
        Self {
            target,
            current: 0,
            last_error: 0,
        }
    }
}

impl HomeostaticLoop for SignalFluxLoop {
    type Measurement = i64;

    fn set_point(&self) -> Self::Measurement {
        self.target
    }

    fn sense(&self) -> Self::Measurement {
        self.current
    }

    fn compare(&self, measured: &Self::Measurement) -> i64 {
        self.target - *measured
    }

    fn effect(&mut self, error: i64) -> Result<()> {
        // Milli no-op: record error for inspection; do not mutate expression.
        self.last_error = error;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a5_effect_is_noop_ok() {
        let mut loop_ = SignalFluxLoop::new(1000);
        loop_.effect(42).expect("ok");
        assert_eq!(loop_.last_error, 42);
    }
}
