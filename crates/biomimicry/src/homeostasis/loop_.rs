//! HomeostaticLoop trait: set-point / sensor / comparator / corrective step + damping.
//!
//! Errors and damping use **milli-integer** units (no floats on the core path).
//!
//! The [`Self::effect`] method name is historical; in prose this is the
//! **corrective step**. M11's [`crate::effector`] module is a different concept
//! (Phase 2 writes leaving the signal stream).

use crate::error::Result;

/// Negative-feedback control loop over some measured quantity `T`.
pub trait HomeostaticLoop {
    /// Measured quantity type.
    type Measurement;

    /// Desired set-point.
    fn set_point(&self) -> Self::Measurement;

    /// Sense the current value.
    fn sense(&self) -> Self::Measurement;

    /// Compare measurement to set-point (signed error in milli-units).
    fn compare(&self, measured: &Self::Measurement) -> i64;

    /// Actuate the corrective step to reduce error.
    ///
    /// # Errors
    ///
    /// Returns an error if the corrective step cannot act.
    fn effect(&mut self, error: i64) -> Result<()>;

    /// Damping term (derivative / PID contribution) in milli-units.
    fn damping(&self, error: i64, previous_error: i64) -> i64 {
        let _ = (error, previous_error);
        0
    }

    /// One control step: sense → compare → damp → corrective step.
    ///
    /// Returns the raw (pre-damping) error for the next step's derivative.
    ///
    /// # Errors
    ///
    /// Propagates corrective-step errors.
    fn step(&mut self, previous_error: i64) -> Result<i64> {
        let measured = self.sense();
        let error = self.compare(&measured);
        let damped = error + self.damping(error, previous_error);
        self.effect(damped)?;
        Ok(error)
    }
}
