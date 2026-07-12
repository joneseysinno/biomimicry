//! PID / derivative term for oscillation control (milli-integer).

/// Damping / PID parameters for homeostatic loops (gains in milli-units).
///
/// Output of [`pid_term`] is in the same milli-error space as the loop error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DampingParams {
    /// Proportional gain (milli).
    pub kp: i64,
    /// Integral gain (milli).
    pub ki: i64,
    /// Derivative gain (milli).
    pub kd: i64,
}

impl Default for DampingParams {
    fn default() -> Self {
        Self {
            kp: 1000, // 1.0 in milli
            ki: 0,
            kd: 100, // 0.1 in milli
        }
    }
}

impl DampingParams {
    /// Undamped (no derivative) — used for oscillation demos.
    #[must_use]
    pub const fn undamped() -> Self {
        Self {
            kp: 1000,
            ki: 0,
            kd: 0,
        }
    }
}

/// Compute a PID contribution from error history (all milli-units).
///
/// Terms are scaled by `/1000` so `kp=1000` means unity proportional gain.
#[must_use]
pub fn pid_term(params: DampingParams, error: i64, integral: i64, derivative: i64) -> i64 {
    (params.kp * error) / 1000 + (params.ki * integral) / 1000 + (params.kd * derivative) / 1000
}
