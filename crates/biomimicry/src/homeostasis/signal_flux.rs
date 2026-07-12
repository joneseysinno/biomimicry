//! Target 2: receptor downregulation / lateral inhibition.

use crate::error::Result;
use crate::homeostasis::HomeostaticLoop;

/// Homeostatic loop over inbound signal flux.
#[derive(Debug, Clone)]
pub struct SignalFluxLoop {
    /// Target flux.
    pub target: f64,
    /// Measured flux.
    pub current: f64,
}

impl HomeostaticLoop for SignalFluxLoop {
    type Measurement = f64;

    fn set_point(&self) -> Self::Measurement {
        self.target
    }

    fn sense(&self) -> Self::Measurement {
        self.current
    }

    fn compare(&self, measured: &Self::Measurement) -> f64 {
        self.target - *measured
    }

    fn effect(&mut self, _error: f64) -> Result<()> {
        todo!("downregulate receptors / apply lateral inhibition")
    }
}
