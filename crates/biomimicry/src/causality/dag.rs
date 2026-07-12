//! Causal DAG build / traverse (single / conjunction / disjunction).

use super::CausalStamp;

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
#[derive(Debug, Clone)]
pub struct CausalNode {
    /// Stamp for this event.
    pub stamp: CausalStamp,
    /// Predecessor stamps.
    pub predecessors: Vec<CausalStamp>,
    /// How predecessors combine.
    pub kind: CausalEdgeKind,
}

/// Causal DAG over stamped events.
#[derive(Debug, Clone, Default)]
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

    /// Traverse ancestors of a stamp.
    pub fn ancestors(&self, _stamp: CausalStamp) -> Vec<CausalStamp> {
        todo!("traverse causal ancestors")
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
