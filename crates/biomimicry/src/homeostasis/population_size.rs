//! Target 1: population-size homeostatic loop.

use crate::error::Result;
use crate::homeostasis::HomeostaticLoop;

/// Homeostatic loop that holds population size near a set-point.
#[derive(Debug, Clone)]
pub struct PopulationSizeLoop {
    /// Desired population size.
    pub target: usize,
    /// Current population size.
    pub current: usize,
}

impl HomeostaticLoop for PopulationSizeLoop {
    type Measurement = usize;

    fn set_point(&self) -> Self::Measurement {
        self.target
    }

    fn sense(&self) -> Self::Measurement {
        self.current
    }

    fn compare(&self, measured: &Self::Measurement) -> f64 {
        self.target as f64 - *measured as f64
    }

    fn effect(&mut self, _error: f64) -> Result<()> {
        todo!("grow or cull population toward set-point")
    }
}
