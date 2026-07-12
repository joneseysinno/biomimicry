//! Settle detector — has the organism come to rest?

/// Outcome of a convergence check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettleStatus {
    /// Still moving through state space.
    Transient,
    /// Settled into a basin.
    Converged,
    /// Timed out before settling.
    TimedOut,
}

/// Detect whether recent trajectory has settled.
#[must_use]
pub fn detect_convergence(_trajectory: &[f64], _epsilon: f64) -> SettleStatus {
    todo!("detect settle / convergence")
}
