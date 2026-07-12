//! PID / derivative term for oscillation control.

/// Damping / PID parameters for homeostatic loops.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DampingParams {
    /// Proportional gain.
    pub kp: f64,
    /// Integral gain.
    pub ki: f64,
    /// Derivative gain.
    pub kd: f64,
}

impl Default for DampingParams {
    fn default() -> Self {
        Self {
            kp: 1.0,
            ki: 0.0,
            kd: 0.1,
        }
    }
}

/// Compute a PID contribution from error history.
#[must_use]
pub fn pid_term(params: DampingParams, error: f64, integral: f64, derivative: f64) -> f64 {
    params.kp * error + params.ki * integral + params.kd * derivative
}
