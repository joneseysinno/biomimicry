//! Regulatory rule: when \[conditions\] → express \[genes\].

use crate::genesis::GeneId;

/// A Phase 1 regulatory rule.
#[derive(Debug, Clone)]
pub struct RegulatoryRule {
    /// Human-readable name.
    pub name: String,
    /// Genes activated when the rule fires.
    pub express: Vec<GeneId>,
    /// Genes suppressed when the rule fires.
    pub suppress: Vec<GeneId>,
}

impl RegulatoryRule {
    /// Create a named rule.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            express: Vec::new(),
            suppress: Vec::new(),
        }
    }

    /// Whether the rule's conditions currently hold.
    #[must_use]
    pub fn matches(&self) -> bool {
        todo!("evaluate regulatory conditions")
    }
}
