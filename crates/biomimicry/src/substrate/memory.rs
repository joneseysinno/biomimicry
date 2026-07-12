//! In-memory [`Store`] implementation — default for fast, deterministic tests.

use std::collections::BTreeMap;

use crate::causality::{CausalDag, CausalNode};
use crate::error::Result;
use crate::genesis::{Hyperedge, PrimitiveNode, PrimitiveNodeId};
use crate::substrate::{BranchId, SnapshotId, SnapshotMeta, Store};

/// Zero-dependency in-memory store (M0 default).
#[derive(Debug, Clone, Default)]
pub struct MemoryStore {
    nodes: BTreeMap<PrimitiveNodeId, PrimitiveNode>,
    hyperedges: Vec<Hyperedge>,
    causal: CausalDag,
    next_snapshot: u64,
    next_branch: u64,
}

impl MemoryStore {
    /// Create an empty in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Store for MemoryStore {
    fn clear_hypergraph(&mut self) -> Result<()> {
        self.nodes.clear();
        self.hyperedges.clear();
        Ok(())
    }

    fn put_node(&mut self, node: &PrimitiveNode) -> Result<()> {
        self.nodes.insert(node.id, node.clone());
        Ok(())
    }

    fn get_node(&self, id: PrimitiveNodeId) -> Result<Option<PrimitiveNode>> {
        Ok(self.nodes.get(&id).cloned())
    }

    fn iter_nodes(&self) -> Result<Vec<PrimitiveNode>> {
        Ok(self.nodes.values().cloned().collect())
    }

    fn put_hyperedge(&mut self, edge: &Hyperedge) -> Result<()> {
        self.hyperedges.push(edge.clone());
        Ok(())
    }

    fn iter_hyperedges(&self) -> Result<Vec<Hyperedge>> {
        Ok(self.hyperedges.clone())
    }

    fn append_causal(&mut self, node: CausalNode) -> Result<()> {
        self.causal.append(node);
        Ok(())
    }

    fn load_causal_dag(&self) -> Result<CausalDag> {
        Ok(self.causal.clone())
    }

    fn snapshot(&mut self, label: &str) -> Result<SnapshotMeta> {
        let id = SnapshotId(self.next_snapshot);
        self.next_snapshot = self.next_snapshot.saturating_add(1);
        Ok(SnapshotMeta {
            id,
            branch: BranchId(0),
            label: label.to_owned(),
        })
    }

    fn branch(&mut self, _from: SnapshotId, _label: &str) -> Result<BranchId> {
        let id = BranchId(self.next_branch);
        self.next_branch = self.next_branch.saturating_add(1);
        Ok(id)
    }

    fn restore(&mut self, _id: SnapshotId) -> Result<()> {
        // Scaffold: no snapshot payload retained yet; restore is a no-op success.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causality::{CausalEdgeKind, CausalStamp};
    use crate::genesis::Hypergraph;

    #[test]
    fn memory_store_round_trips_empty_hypergraph() {
        let mut store = MemoryStore::new();
        let hg = Hypergraph::new();
        store.save_hypergraph(&hg).expect("save");
        let loaded = store.load_hypergraph().expect("load");
        assert!(loaded.edges().next().is_none());
    }

    #[test]
    fn memory_store_issues_snapshot_ids() {
        let mut store = MemoryStore::new();
        let a = store.snapshot("a").expect("snap a");
        let b = store.snapshot("b").expect("snap b");
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn memory_store_appends_causal_nodes() {
        let mut store = MemoryStore::new();
        store
            .append_causal(CausalNode {
                stamp: CausalStamp(1),
                predecessors: Vec::new(),
                kind: CausalEdgeKind::Single,
            })
            .expect("append");
        let dag = store.load_causal_dag().expect("load dag");
        assert_eq!(dag.len(), 1);
    }
}
