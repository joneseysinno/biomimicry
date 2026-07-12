//! Inspector helpers for cell state (pulled forward from later milestones).

use std::fmt::Write as _;

use super::Cell;
use super::lifecycle::{LEGAL_EDGES, LifecycleState, all_states, is_legal};

/// Human-readable profile dump: lifecycle, mode, surfaces, budgets, queue.
#[must_use]
pub fn profile(cell: &Cell) -> String {
    let p = cell.expression.profile();
    format!(
        "cell {id}\n  lifecycle: {life:?}\n  mode: {mode:?}\n  active_genes: {n}\n  \
         receptors: {r}\n  emissions: {e}\n  vetos: {v}\n  \
         budget_p1: {p1}/{c1}\n  budget_p2: {p2}/{c2}\n  queue_depth: {q}\n  stamp: {stamp}\n",
        id = cell.id.0,
        life = cell.lifecycle(),
        mode = cell.mode,
        n = cell.expression.len(),
        r = p.receptor_surface.len(),
        e = p.emission_surface.len(),
        v = p.veto_surface.len(),
        p1 = cell.energy.phase1.remaining_milli,
        c1 = cell.energy.phase1.capacity_milli,
        p2 = cell.energy.phase2.remaining_milli,
        c2 = cell.energy.phase2.capacity_milli,
        q = cell.pending.len(),
        stamp = cell.peek_stamp(),
    )
}

/// Graphviz DOT of the legal lifecycle-edge table.
#[must_use]
pub fn lifecycle_dot() -> String {
    let mut out = String::from("digraph lifecycle {\n  rankdir=LR;\n");
    for s in all_states() {
        let shape = if s == LifecycleState::Dead {
            "doublecircle"
        } else {
            "circle"
        };
        let _ = writeln!(out, "  \"{s:?}\" [shape={shape}];");
    }
    for &(from, to) in LEGAL_EDGES {
        debug_assert!(is_legal(from, to));
        let _ = writeln!(out, "  \"{from:?}\" -> \"{to:?}\";");
    }
    out.push_str("}\n");
    out
}
