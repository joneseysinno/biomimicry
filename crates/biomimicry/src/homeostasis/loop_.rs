//! HomeostaticLoop trait: set-point / sensor / comparator / effector + damping.

use crate::error::Result;

/// Negative-feedback control loop over some measured quantity `T`.
pub trait HomeostaticLoop {
    /// Measured quantity type.
    type Measurement;

    /// Desired set-point.
    fn set_point(&self) -> Self::Measurement;

    /// Sense the current value.
    fn sense(&self) -> Self::Measurement;

    /// Compare measurement to set-point (signed error).
    fn compare(&self, measured: &Self::Measurement) -> f64;

    /// Actuate effectors to reduce error.
    ///
    /// # Errors
    ///
    /// Returns an error if effectors cannot act.
    fn effect(&mut self, error: f64) -> Result<()>;

    /// Damping term (derivative / PID contribution).
    fn damping(&self, error: f64, previous_error: f64) -> f64 {
        let _ = (error, previous_error);
        0.0
    }

    /// One control step: sense → compare → damp → effect.
    ///
    /// # Errors
    ///
    /// Propagates effector errors.
    fn step(&mut self, previous_error: f64) -> Result<f64> {
        let measured = self.sense();
        let error = self.compare(&measured);
        let damped = error + self.damping(error, previous_error);
        self.effect(damped)?;
        Ok(error)
    }
}
