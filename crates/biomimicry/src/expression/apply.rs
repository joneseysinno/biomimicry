//! Write expression-state changes at the Phase 1 boundary.

use crate::cell::ExpressionState;
use crate::error::Result;
use crate::expression::RegulatoryRule;

/// Apply a fired rule to expression state at the Phase 1 boundary.
///
/// Phase 1 changes must not take effect mid-Phase-2.
///
/// # Errors
///
/// Returns an error if application violates expression invariants.
pub fn apply_at_boundary(_state: &mut ExpressionState, _rule: &RegulatoryRule) -> Result<()> {
    todo!("mutate expression state at Phase 1 boundary only")
}
