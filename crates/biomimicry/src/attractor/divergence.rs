//! Divergence and limit-cycle detector.

/// Pathological dynamics detected on a trajectory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DivergenceKind {
    /// Unbounded drift.
    Diverging,
    /// Sustained oscillation (limit cycle / ringing).
    LimitCycle,
}

/// Detect divergence or limit-cycle behavior.
#[must_use]
pub fn detect_divergence(_trajectory: &[f64]) -> Option<DivergenceKind> {
    todo!("detect divergence or limit cycle")
}
