//! Regulatory rule: when \[conditions\] → express / suppress \[genes\].

use crate::cell::{Cell, Operation};
use crate::genesis::GeneId;
use crate::signal::SignalKind;

/// Mechanical condition for a Phase 1 regulatory rule (AND-combined).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RuleCondition {
    /// True if any queued [`Operation::Receive`] has this signal kind.
    SignalKind(SignalKind),
    /// True if `gene` is currently active in the cell.
    GeneActive(GeneId),
    /// True if `gene` is not currently active.
    GeneInactive(GeneId),
}

/// A Phase 1 regulatory rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegulatoryRule {
    /// Human-readable name.
    pub name: String,
    /// Conditions that must all hold (AND).
    pub conditions: Vec<RuleCondition>,
    /// Genes activated when the rule fires.
    pub express: Vec<GeneId>,
    /// Genes suppressed when the rule fires.
    pub suppress: Vec<GeneId>,
}

impl RegulatoryRule {
    /// Create a named rule with no conditions or effects yet.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            conditions: Vec::new(),
            express: Vec::new(),
            suppress: Vec::new(),
        }
    }

    /// Builder: add a condition.
    #[must_use]
    pub fn with_condition(mut self, condition: RuleCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Builder: genes to activate.
    #[must_use]
    pub fn with_express(mut self, genes: impl IntoIterator<Item = GeneId>) -> Self {
        self.express.extend(genes);
        self
    }

    /// Builder: genes to suppress.
    #[must_use]
    pub fn with_suppress(mut self, genes: impl IntoIterator<Item = GeneId>) -> Self {
        self.suppress.extend(genes);
        self
    }

    /// Whether every condition holds given the cell and queued Phase 1 ops.
    #[must_use]
    pub fn matches(&self, cell: &Cell, queued: &[Operation]) -> bool {
        self.conditions
            .iter()
            .all(|c| condition_holds(c, cell, queued))
    }
}

fn condition_holds(condition: &RuleCondition, cell: &Cell, queued: &[Operation]) -> bool {
    match condition {
        RuleCondition::SignalKind(kind) => {
            // Prefer queued Receives (same Phase 1 batch); fall back to the
            // cell's last matched inbound kind so rules can fire at the Phase 1
            // boundary after Receive already drained in Phase 2.
            queued.iter().any(|op| match op {
                Operation::Receive(sig) => sig.kind == *kind,
                _ => false,
            }) || cell.last_inbound_kind.as_ref() == Some(kind)
        }
        RuleCondition::GeneActive(gene) => cell.expression.is_active(*gene),
        RuleCondition::GeneInactive(gene) => !cell.expression.is_active(*gene),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::CellId;
    use crate::genesis::{compile, toy_dna};
    use crate::signal::{CausalStamp, Payload, Scope, Signal, SignalType};
    use std::sync::Arc;

    #[test]
    fn signal_kind_condition() {
        let genome = compile(&toy_dna()).unwrap();
        let cell = Cell::new(CellId(1), Arc::clone(&genome));
        let rule = RegulatoryRule::new("r")
            .with_condition(RuleCondition::SignalKind(SignalKind::new("trigger")));
        let sig = Signal::new(
            SignalType::Regulatory,
            "trigger",
            Scope::SelfCell,
            Payload::empty(),
            CellId(1),
            CausalStamp(0),
        );
        assert!(rule.matches(&cell, &[Operation::Receive(sig)]));
        assert!(!rule.matches(&cell, &[]));
    }
}
