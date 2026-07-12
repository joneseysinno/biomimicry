//! Working (eventually-consistent) vs committed (gated) equilibria.

/// Which consistency equilibrium an observation belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Equilibrium {
    /// Eventually-consistent working state (may still be settling).
    #[default]
    Working,
    /// Checkpoint-gated committed state.
    Committed,
}

/// Gate that promotes working state to committed.
#[derive(Debug, Clone, Default)]
pub struct CommitmentGate {
    /// Whether the gate is currently open.
    pub open: bool,
}

impl CommitmentGate {
    /// Attempt to commit if the gate is open and preconditions hold.
    #[must_use]
    pub fn try_commit(&self) -> bool {
        todo!("gate working → committed equilibrium")
    }
}
