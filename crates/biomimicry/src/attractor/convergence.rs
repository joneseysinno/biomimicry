//! Settle detector — has the organism come to rest?
//!
//! Convergence is discrete: identical expression fingerprints over a window.
//! Scheduler **drained** is not the same as attractor **converged**.

/// Outcome of a convergence check / settle run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettleStatus {
    /// Still moving through state space.
    Transient,
    /// Settled into a basin (last `window` fingerprints identical).
    Converged,
    /// Timed out before settling.
    TimedOut,
}

/// Default number of identical trailing fingerprints required for convergence.
pub const DEFAULT_CONVERGENCE_WINDOW: usize = 3;

/// Detect whether recent trajectory has settled.
///
/// Returns [`SettleStatus::Converged`] when the last `window` fingerprints are
/// identical (requires `trajectory.len() >= window`). Otherwise
/// [`SettleStatus::Transient`]. Callers map prolonged Transient + cap to
/// [`SettleStatus::TimedOut`].
#[must_use]
pub fn detect_convergence(trajectory: &[u128], window: usize) -> SettleStatus {
    if window == 0 || trajectory.len() < window {
        return SettleStatus::Transient;
    }
    let start = trajectory.len() - window;
    let first = trajectory[start];
    if trajectory[start..].iter().all(|&fp| fp == first) {
        SettleStatus::Converged
    } else {
        SettleStatus::Transient
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converges_when_window_identical() {
        assert_eq!(
            detect_convergence(&[1, 2, 2, 2], 3),
            SettleStatus::Converged
        );
        assert_eq!(
            detect_convergence(&[1, 2, 3, 3], 3),
            SettleStatus::Transient
        );
    }
}
