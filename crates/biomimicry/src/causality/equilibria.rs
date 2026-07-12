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
    /// Attempt to commit if the gate is open.
    ///
    /// M7: returns [`Self::open`] (no additional preconditions).
    #[must_use]
    pub fn try_commit(&self) -> bool {
        self.open
    }
}
