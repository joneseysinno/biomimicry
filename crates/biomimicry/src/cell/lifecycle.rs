//! Lifecycle states and the legal-edge transition table.
//!
//! "By construction" rejection: [`LifecycleState`] is mutated only through
//! [`crate::cell::Cell::try_transition`], which consults [`is_legal`].

/// Lifecycle state of a cell (Part II.3 / II.6 Layer 1).
///
/// Distinct from [`crate::cell::BehavioralMode::Differentiating`] — that is a
/// Layer-2 posture. A cell in lifecycle `Differentiating` is typically also in
/// mode `Differentiating`, but the two axes must not be conflated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LifecycleState {
    /// Not yet committed to a fate.
    #[default]
    Undifferentiated,
    /// Fate selection / identity change in progress.
    Differentiating,
    /// Actively receiving and expressing.
    Active,
    /// Alive but quiet (can return to Active).
    Quiescent,
    /// Terminal — absorbing; no outgoing edges.
    Dead,
}

/// Static legal-edge table (Part II.3, II.6).
pub const LEGAL_EDGES: &[(LifecycleState, LifecycleState)] = &[
    (
        LifecycleState::Undifferentiated,
        LifecycleState::Differentiating,
    ),
    (LifecycleState::Differentiating, LifecycleState::Active),
    (
        LifecycleState::Differentiating,
        LifecycleState::Undifferentiated,
    ),
    (LifecycleState::Active, LifecycleState::Quiescent),
    (LifecycleState::Active, LifecycleState::Dead),
    (LifecycleState::Active, LifecycleState::Differentiating),
    (LifecycleState::Quiescent, LifecycleState::Active),
    (LifecycleState::Quiescent, LifecycleState::Dead),
];

/// Whether `from → to` is a permitted lifecycle edge.
#[must_use]
pub fn is_legal(from: LifecycleState, to: LifecycleState) -> bool {
    LEGAL_EDGES.iter().any(|&(f, t)| f == from && t == to)
}

/// All lifecycle states (for exhaustive property tests).
#[must_use]
pub const fn all_states() -> [LifecycleState; 5] {
    [
        LifecycleState::Undifferentiated,
        LifecycleState::Differentiating,
        LifecycleState::Active,
        LifecycleState::Quiescent,
        LifecycleState::Dead,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_is_terminal() {
        for to in all_states() {
            assert!(!is_legal(LifecycleState::Dead, to));
        }
    }

    #[test]
    fn undifferentiated_cannot_skip_to_active() {
        assert!(!is_legal(
            LifecycleState::Undifferentiated,
            LifecycleState::Active
        ));
    }

    #[test]
    fn legal_edges_match_table() {
        for &(from, to) in LEGAL_EDGES {
            assert!(is_legal(from, to), "{from:?} → {to:?}");
        }
    }
}
