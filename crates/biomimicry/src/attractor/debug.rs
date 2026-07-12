//! Inspector / decision-trace helpers for attractor dynamics.

use std::fmt::Write as _;

use crate::attractor::{Basin, Landscape, SettleStatus};
use crate::causality::{CausalEventLog, causal_order_dot};

/// Graphviz + trajectory dump for settle debugging.
#[must_use]
pub fn settle_trace(log: &CausalEventLog, trajectory: &[u128], status: SettleStatus) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "settle_status: {status:?}");
    let _ = writeln!(out, "trajectory_csv:");
    for (i, fp) in trajectory.iter().enumerate() {
        let _ = writeln!(out, "{i},{fp:032x}");
    }
    out.push('\n');
    out.push_str(&causal_order_dot(log));
    out
}

/// One-line basin / landscape summary.
#[must_use]
pub fn landscape_summary(landscape: &Landscape, basin: &Basin, key: u128) -> String {
    format!(
        "key={key:032x} potential={} in_basin={}",
        landscape.potential(key),
        basin.contains(key)
    )
}
