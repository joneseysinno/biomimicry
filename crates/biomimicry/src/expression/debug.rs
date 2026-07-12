//! Inspector / decision-trace helpers for the expression engine.

use std::fmt::Write;

use crate::cell::{Cell, Operation};
use crate::expression::RuleNetwork;

/// Human-readable trace of which rules would fire for `(cell, queued)`.
#[must_use]
pub fn decision_trace(network: &RuleNetwork, cell: &Cell, queued: &[Operation]) -> String {
    let mut out = String::from("expression decision\n");
    for (i, rule) in network.rules.iter().enumerate() {
        let fires = rule.matches(cell, queued);
        let _ = writeln!(
            out,
            "  [{i}] {name}: {status}",
            name = rule.name,
            status = if fires { "FIRE" } else { "skip" },
        );
    }
    out
}
