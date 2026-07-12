//! Write expression-state changes at the Phase 1 boundary.
//!
//! Phase 1 changes must not take effect mid-Phase-2 (enforced by the scheduler).

use std::collections::BTreeSet;

use crate::cell::ExpressionState;
use crate::error::Result;
use crate::expression::RegulatoryRule;
use crate::genesis::GeneId;
use crate::metabolism::ExpressionDelta;

/// Resolve conflicts in a delta: **suppress wins** over activate for the same gene.
#[must_use]
pub fn resolve_conflicts(mut delta: ExpressionDelta) -> ExpressionDelta {
    let suppress: BTreeSet<GeneId> = delta.suppress.iter().copied().collect();
    delta.activate.retain(|g| !suppress.contains(g));
    // Dedup while preserving first-seen order within each list.
    let mut seen_a = BTreeSet::new();
    delta.activate.retain(|g| seen_a.insert(*g));
    let mut seen_s = BTreeSet::new();
    delta.suppress.retain(|g| seen_s.insert(*g));
    delta
}

/// Merge another rule's effects into an accumulating delta.
pub fn merge_rule_into(delta: &mut ExpressionDelta, rule: &RegulatoryRule) {
    delta.activate.extend(rule.express.iter().copied());
    delta.suppress.extend(rule.suppress.iter().copied());
}

/// Apply a resolved delta to expression state at the Phase 1 boundary.
pub fn apply_delta(state: &mut ExpressionState, delta: &ExpressionDelta) {
    let resolved = resolve_conflicts(delta.clone());
    for g in &resolved.activate {
        state.activate(*g);
    }
    for g in &resolved.suppress {
        state.suppress(*g);
    }
}

/// Apply a fired rule to expression state at the Phase 1 boundary.
///
/// # Errors
///
/// Currently infallible; reserved for future invariant checks.
pub fn apply_at_boundary(state: &mut ExpressionState, rule: &RegulatoryRule) -> Result<()> {
    let mut delta = ExpressionDelta::default();
    merge_rule_into(&mut delta, rule);
    apply_delta(state, &delta);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suppress_wins_conflict() {
        let g = GeneId(1);
        let delta = ExpressionDelta {
            activate: vec![g],
            suppress: vec![g],
        };
        let r = resolve_conflicts(delta);
        assert!(r.activate.is_empty());
        assert_eq!(r.suppress, vec![g]);
    }
}
