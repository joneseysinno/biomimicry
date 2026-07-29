//! DOT visualisation of the block graph after linking.

use std::fmt::Write as _;

use crate::blocks::link::Linked;

/// Render a linked composition as Graphviz DOT.
///
/// Blocks are clusters, ports are nodes, wires are solid edges, bridges dashed.
#[must_use]
pub fn to_dot(linked: &Linked) -> String {
    let mut out = String::from("digraph blocks {\n  rankdir=LR;\n  compound=true;\n");
    for (i, g) in linked.ganglia.iter().enumerate() {
        let cluster = format!("cluster_{i}");
        let _ = writeln!(out, "  subgraph {cluster} {{");
        let _ = writeln!(out, "    label=\"{}\";", g.name);
        for p in &g.input_ports {
            let id = port_id(g.name.as_str(), p.kind.as_str(), "in");
            let _ = writeln!(
                out,
                "    \"{id}\" [label=\"{}\\nin\", shape=invhouse];",
                p.kind.local_name()
            );
        }
        for p in &g.output_ports {
            let id = port_id(g.name.as_str(), p.kind.as_str(), "out");
            let _ = writeln!(
                out,
                "    \"{id}\" [label=\"{}\\nout\", shape=house];",
                p.kind.local_name()
            );
        }
        out.push_str("  }\n");
    }
    for wire in &linked.wires {
        let from = port_id(
            wire.export_block.as_str(),
            &format!("{}::{}", wire.export_block, wire.export_kind.as_str()),
            "out",
        );
        let to = port_id(
            wire.import_block.as_str(),
            &format!("{}::{}", wire.import_block, wire.import_kind.as_str()),
            "in",
        );
        let _ = writeln!(out, "  \"{from}\" -> \"{to}\";");
    }
    for bridge in &linked.bridges {
        let _ = writeln!(
            out,
            "  \"bridge_{}\" [label=\"{}\", shape=diamond, style=dashed];",
            escape(&bridge.kind),
            escape(&bridge.kind)
        );
    }
    let _ = writeln!(out, "  label=\"genotype {}\";\n}}", linked.genotype);
    out
}

fn port_id(block: &str, kind: &str, dir: &str) -> String {
    escape(&format!("{block}/{kind}/{dir}"))
}

fn escape(s: &str) -> String {
    s.replace('\"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::fixture::{pipeline_blocks, pipeline_manifest};
    use crate::blocks::link;

    #[test]
    fn pipeline_dot_snapshot() {
        let linked = link::link(&pipeline_blocks(), &pipeline_manifest()).expect("link");
        let dot = to_dot(&linked);
        assert!(dot.contains("digraph"));
        assert!(dot.contains("sum"));
        assert!(dot.contains("scale"));
        assert!(dot.contains("sink"));
        assert!(dot.contains("bridge"));
        assert!(dot.contains("cluster_"));
        assert!(dot.contains("genotype"));
    }
}
