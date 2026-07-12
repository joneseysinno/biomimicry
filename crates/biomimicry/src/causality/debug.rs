//! Graphviz dump of a causal event log (proto-DAG).

use super::CausalEventLog;

/// Render the event log as a Graphviz digraph.
#[must_use]
pub fn causal_order_dot(log: &CausalEventLog) -> String {
    use std::fmt::Write as _;
    let mut out = String::from("digraph causal {\n  rankdir=LR;\n");
    for (i, ev) in log.events().iter().enumerate() {
        let _ = writeln!(
            out,
            "  \"e{i}\" [label=\"{tag}\\n{child:x}\\n{stamp}\"];",
            tag = ev.tag,
            child = ev.child.0,
            stamp = ev.stamp,
        );
        if let Some(parent) = ev.parent {
            let _ = writeln!(
                out,
                "  \"p{parent:x}\" -> \"e{i}\" [label=\"caused_by\"];",
                parent = parent.0,
            );
        }
    }
    out.push_str("}\n");
    out
}
