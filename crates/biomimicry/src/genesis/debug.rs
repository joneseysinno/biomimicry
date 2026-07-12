//! Graphviz dump for grn + genome (inspector seed).

use std::fmt::Write as _;

use super::{GeneOrigin, Genome, Grn};

/// Render a Graphviz DOT digraph of primitive nodes, cistrons, and complement
/// links (dashed). ~inspector ergonomics pulled forward from later milestones.
#[must_use]
pub fn to_dot(graph: &Grn, genome: &Genome) -> String {
    let mut out = String::from("digraph genesis {\n");
    out.push_str("  graph [rankdir=LR];\n");
    out.push_str("  node [shape=box];\n");

    for node in graph.nodes() {
        let _ = writeln!(
            out,
            "  \"n{id:x}\" [label=\"{prim:?}\\n{id:x}\"];",
            id = node.id.0,
            prim = node.primitive,
        );
    }

    for (i, edge) in graph.iter_cistrons().enumerate() {
        let ename = format!("e{i}_{}", edge.kind.as_str());
        let _ = writeln!(
            out,
            "  \"{ename}\" [shape=ellipse,label=\"{kind}\"];",
            kind = edge.kind.as_str(),
        );
        for ep in &edge.endpoints {
            let pol = match ep.polarity {
                super::EndpointPolarity::Positive => "+",
                super::EndpointPolarity::Negative => "−",
            };
            let _ = writeln!(
                out,
                "  \"{ename}\" -> \"n{nid:x}\" [label=\"{pol} {role}\"];",
                nid = ep.node.0,
                role = ep.role.as_str(),
            );
        }
    }

    for gene in genome.iter() {
        if let GeneOrigin::Complement(of) = gene.origin {
            let _ = writeln!(
                out,
                "  \"g{of:x}\" -> \"g{to:x}\" [style=dashed,label=\"complement\"];",
                of = of.0,
                to = gene.id.0,
            );
        }
        let _ = writeln!(
            out,
            "  \"g{id:x}\" [shape=diamond,label=\"gene\\n{id:x}\"];",
            id = gene.id.0,
        );
    }

    out.push_str("}\n");
    out
}
