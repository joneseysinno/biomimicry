//! Causal DAG build / traverse (single / conjunction / disjunction).

use std::collections::{BTreeSet, VecDeque};

use super::CausalStamp;
use crate::signal::SignalId;

/// Edge combinator in the causal DAG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CausalEdgeKind {
    /// Single predecessor.
    Single,
    /// All predecessors required.
    Conjunction,
    /// Any predecessor sufficient.
    Disjunction,
}

/// Node in the causal DAG.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalNode {
    /// Stamp for this event.
    pub stamp: CausalStamp,
    /// Predecessor stamps.
    pub predecessors: Vec<CausalStamp>,
    /// How predecessors combine.
    pub kind: CausalEdgeKind,
    /// Signal identity (disambiguates stamp collisions from synthetic tags).
    pub signal_id: SignalId,
    /// Short tag (`deliver`, `emit`, `transduce`, …).
    pub tag: String,
}

impl CausalNode {
    /// Construct a node with empty predecessors and [`CausalEdgeKind::Single`].
    #[must_use]
    pub fn leaf(stamp: CausalStamp, signal_id: SignalId, tag: impl Into<String>) -> Self {
        Self {
            stamp,
            predecessors: Vec::new(),
            kind: CausalEdgeKind::Single,
            signal_id,
            tag: tag.into(),
        }
    }
}

/// Causal DAG over stamped events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CausalDag {
    nodes: Vec<CausalNode>,
}

impl CausalDag {
    /// Create an empty DAG.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a causal node.
    pub fn append(&mut self, node: CausalNode) {
        self.nodes.push(node);
    }

    /// Replace all nodes (flush / restore path).
    pub fn replace(&mut self, nodes: Vec<CausalNode>) {
        self.nodes = nodes;
    }

    /// Borrow all nodes in append order.
    #[must_use]
    pub fn nodes(&self) -> &[CausalNode] {
        &self.nodes
    }

    /// Traverse ancestors of a stamp (BFS over predecessors).
    ///
    /// Returns unique ancestor stamps in discovery order. When multiple nodes
    /// share a stamp, predecessors are ordered by `(stamp, signal_id)`.
    #[must_use]
    pub fn ancestors(&self, stamp: CausalStamp) -> Vec<CausalStamp> {
        let mut by_stamp: std::collections::BTreeMap<CausalStamp, Vec<&CausalNode>> =
            std::collections::BTreeMap::new();
        for n in &self.nodes {
            by_stamp.entry(n.stamp).or_default().push(n);
        }
        for list in by_stamp.values_mut() {
            list.sort_by_key(|n| n.signal_id.0);
        }

        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        let mut queue = VecDeque::new();

        let Some(starts) = by_stamp.get(&stamp) else {
            return out;
        };
        let mut seed_preds: Vec<CausalStamp> = starts
            .iter()
            .flat_map(|n| n.predecessors.iter().copied())
            .collect();
        seed_preds.sort_unstable();
        seed_preds.dedup();
        // Stable expansion order when several start nodes disagree: sort by stamp.
        for p in seed_preds {
            queue.push_back(p);
        }

        while let Some(cur) = queue.pop_front() {
            if !seen.insert(cur) {
                continue;
            }
            out.push(cur);
            let Some(nodes) = by_stamp.get(&cur) else {
                continue;
            };
            let mut next: Vec<CausalStamp> = nodes
                .iter()
                .flat_map(|n| n.predecessors.iter().copied())
                .collect();
            next.sort_unstable();
            next.dedup();
            for p in next {
                if !seen.contains(&p) {
                    queue.push_back(p);
                }
            }
        }
        out
    }

    /// Number of nodes currently in the DAG.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the DAG has no nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a3_ancestors_contains_parent_stamps() {
        let mut dag = CausalDag::new();
        dag.append(CausalNode::leaf(CausalStamp(1), SignalId(10), "root"));
        dag.append(CausalNode {
            stamp: CausalStamp(2),
            predecessors: vec![CausalStamp(1)],
            kind: CausalEdgeKind::Single,
            signal_id: SignalId(20),
            tag: "child".into(),
        });
        dag.append(CausalNode {
            stamp: CausalStamp(3),
            predecessors: vec![CausalStamp(2)],
            kind: CausalEdgeKind::Single,
            signal_id: SignalId(30),
            tag: "leaf".into(),
        });
        let anc = dag.ancestors(CausalStamp(3));
        assert_eq!(anc, vec![CausalStamp(2), CausalStamp(1)]);
    }
}
