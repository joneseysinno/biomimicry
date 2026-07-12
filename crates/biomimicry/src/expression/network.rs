//! Reactive rule network evaluation.

use crate::cell::{Cell, Operation};
use crate::expression::RegulatoryRule;
use crate::expression::apply::{merge_rule_into, resolve_conflicts};
use crate::metabolism::ExpressionDelta;

/// Network of Phase 1 regulatory rules.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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

    /// Builder: append a rule.
    #[must_use]
    pub fn with_rule(mut self, rule: RegulatoryRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Indices of rules whose conditions hold (declaration order).
    #[must_use]
    pub fn firing_indices(&self, cell: &Cell, queued: &[Operation]) -> Vec<usize> {
        self.rules
            .iter()
            .enumerate()
            .filter_map(|(i, rule)| rule.matches(cell, queued).then_some(i))
            .collect()
    }

    /// Evaluate all matching rules into one conflict-resolved delta.
    ///
    /// Also folds any explicit `Express { gene, on }` ops from `queued` so that
    /// an empty network remains compatible with [`crate::metabolism::ExplicitRegulator`].
    #[must_use]
    pub fn evaluate(&self, cell: &Cell, queued: &[Operation]) -> ExpressionDelta {
        let mut delta = ExpressionDelta::default();
        for idx in self.firing_indices(cell, queued) {
            merge_rule_into(&mut delta, &self.rules[idx]);
        }
        // Compat: explicit Express ops still apply.
        for op in queued {
            if let Operation::Express { gene, on } = op {
                if *on {
                    delta.activate.push(*gene);
                } else {
                    delta.suppress.push(*gene);
                }
            }
        }
        resolve_conflicts(delta)
    }
}
