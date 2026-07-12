//! Reactive rule network evaluation.

use super::RegulatoryRule;

/// Network of Phase 1 regulatory rules.
#[derive(Debug, Clone, Default)]
pub struct RuleNetwork {
    /// Rules in evaluation order (determinism-owned).
    pub rules: Vec<RegulatoryRule>,
}

impl RuleNetwork {
    /// Create an empty network.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluate all matching rules; returns indices that fired.
    #[must_use]
    pub fn evaluate(&self) -> Vec<usize> {
        todo!("evaluate reactive rule network")
    }
}
