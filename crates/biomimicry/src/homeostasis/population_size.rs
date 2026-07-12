//! Target 1: population-size homeostatic loop.
//!
//! Effect records pending recruit/cull counts; the organism applies them
//! (recruitment = M5 non-mitotic spawn; cull = `Die` via scheduler).

use crate::error::Result;
use crate::homeostasis::{DampingParams, HomeostaticLoop};

/// Homeostatic loop that holds population size near a set-point.
#[derive(Debug, Clone)]
pub struct PopulationSizeLoop {
    /// Desired living population size.
    pub target: usize,
    /// Current living population size (updated by the organism each tick).
    pub current: usize,
    /// Damping / PID parameters.
    pub damping: DampingParams,
    /// When true, effect overshoots by ±2 (undamped oscillation demo).
    pub aggressive: bool,
    /// Cells to recruit after the last effect (organism consumes).
    pub pending_recruit: usize,
    /// Cells to cull after the last effect (organism consumes).
    pub pending_cull: usize,
}

impl PopulationSizeLoop {
    /// Create a damped loop at `target`.
    #[must_use]
    pub fn new(target: usize) -> Self {
        Self {
            target,
            current: target,
            damping: DampingParams::default(),
            aggressive: false,
            pending_recruit: 0,
            pending_cull: 0,
        }
    }

    /// Undamped aggressive loop for limit-cycle demos.
    #[must_use]
    pub fn undamped(target: usize) -> Self {
        Self {
            target,
            current: target,
            damping: DampingParams::undamped(),
            aggressive: true,
            pending_recruit: 0,
            pending_cull: 0,
        }
    }

    /// Take and clear pending recruit count.
    pub fn take_recruit(&mut self) -> usize {
        std::mem::take(&mut self.pending_recruit)
    }

    /// Take and clear pending cull count.
    pub fn take_cull(&mut self) -> usize {
        std::mem::take(&mut self.pending_cull)
    }
}

impl HomeostaticLoop for PopulationSizeLoop {
    type Measurement = usize;

    fn set_point(&self) -> Self::Measurement {
        self.target
    }

    fn sense(&self) -> Self::Measurement {
        self.current
    }

    fn compare(&self, measured: &Self::Measurement) -> i64 {
        let target = i64::try_from(self.target).unwrap_or(i64::MAX);
        let measured = i64::try_from(*measured).unwrap_or(i64::MAX);
        (target - measured).saturating_mul(1000)
    }

    fn damping(&self, error: i64, previous_error: i64) -> i64 {
        let derivative = error - previous_error;
        (self.damping.kd * derivative) / 1000
    }

    fn effect(&mut self, error: i64) -> Result<()> {
        self.pending_recruit = 0;
        self.pending_cull = 0;
        if error == 0 {
            // Undamped kick: leave the set-point so the loop rings.
            if self.aggressive {
                self.pending_recruit = 2;
            }
            return Ok(());
        }
        // Convert milli-error to cell steps (at least 1 when nonzero).
        let steps = usize::try_from((error.abs() / 1000).max(1)).unwrap_or(1);
        let steps = if self.aggressive {
            steps.saturating_add(1).max(2)
        } else {
            // Damped: single-cell steps toward set-point.
            1.min(steps)
        };
        if error > 0 {
            self.pending_recruit = steps;
        } else {
            self.pending_cull = steps;
        }
        Ok(())
    }
}
