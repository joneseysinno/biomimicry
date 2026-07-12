//! `Store` trait — hypergraph / causal persistence contract.
//!
//! The engine depends only on this trait. The default implementation is
//! [`crate::substrate::MemoryStore`]; the `infinite-db` backing lives in the
//! separate `biomimicry-substrate` crate.
//!
//! Hypergraph-facing methods (`put_node` / `get_node` / `put_hyperedge` /
//! `iter_*` / `clear_hypergraph`) are fleshed out in M1 so infinite-db is a
//! genuine drop-in at M7. The causal half waits for M7.

use crate::causality::{CausalDag, CausalNode};
use crate::error::Result;
use crate::genesis::{Hyperedge, Hypergraph, PrimitiveNode, PrimitiveNodeId};
use crate::substrate::{BranchId, SnapshotId, SnapshotMeta};

/// Persistence contract for DNA hypergraph, genome artifacts, and causal logs.
pub trait Store {
    /// Remove all hypergraph nodes and edges (prep for a full persist).
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure.
    fn clear_hypergraph(&mut self) -> Result<()>;

    /// Upsert a primitive node.
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure.
    fn put_node(&mut self, node: &PrimitiveNode) -> Result<()>;

    /// Fetch a primitive node by id.
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure.
    fn get_node(&self, id: PrimitiveNodeId) -> Result<Option<PrimitiveNode>>;

    /// Iterate all stored primitive nodes.
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure.
    fn iter_nodes(&self) -> Result<Vec<PrimitiveNode>>;

    /// Append / upsert a hyperedge.
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure.
    fn put_hyperedge(&mut self, edge: &Hyperedge) -> Result<()>;

    /// Iterate all stored hyperedges.
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure.
    fn iter_hyperedges(&self) -> Result<Vec<Hyperedge>>;

    /// Load the working DNA hypergraph (convenience over fine-grained reads).
    ///
    /// # Errors
    ///
    /// Returns an error on I/O or corruption.
    fn load_hypergraph(&self) -> Result<Hypergraph>
    where
        Self: Sized,
    {
        Hypergraph::load(self)
    }

    /// Persist the DNA hypergraph (convenience over fine-grained writes).
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure.
    fn save_hypergraph(&mut self, hypergraph: &Hypergraph) -> Result<()>
    where
        Self: Sized,
    {
        hypergraph.persist(self)
    }

    /// Append a causal event (Phase 1 or Phase 2 log).
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure.
    fn append_causal(&mut self, node: CausalNode) -> Result<()>;

    /// Load the causal DAG.
    ///
    /// # Errors
    ///
    /// Returns an error on I/O or corruption.
    fn load_causal_dag(&self) -> Result<CausalDag>;

    /// Take a named snapshot of current store state.
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure.
    fn snapshot(&mut self, label: &str) -> Result<SnapshotMeta>;

    /// Branch from a snapshot for counterfactual replay.
    ///
    /// # Errors
    ///
    /// Returns an error if the snapshot is unknown or I/O fails.
    fn branch(&mut self, from: SnapshotId, label: &str) -> Result<BranchId>;

    /// Restore store state from a snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error if the snapshot is unknown or I/O fails.
    fn restore(&mut self, id: SnapshotId) -> Result<()>;
}
