//! `Store` trait — grn / causal persistence contract.
//!
//! The engine depends only on this trait. The default implementation is
//! [`crate::substrate::MemoryStore`]; the `infinite-db` backing lives in the
//! separate `biomimicry-substrate` crate.
//!
//! GRN-facing methods (`put_node` / `get_node` / `put_cistron` /
//! `iter_*` / `clear_grn`) are fleshed out in M1 so infinite-db is a
//! genuine drop-in at M7. The causal half waits for M7.

use crate::causality::{CausalDag, CausalEventLog, CausalNode};
use crate::error::Result;
use crate::genesis::{Cistron, Grn, PrimitiveNode, PrimitiveNodeId};
use crate::substrate::{BranchId, SnapshotId, SnapshotMeta};

/// Persistence contract for DNA GRN, genome artifacts, and causal logs.
pub trait Store {
    /// Remove all grn nodes and edges (prep for a full persist).
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure.
    fn clear_grn(&mut self) -> Result<()>;

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

    /// Append / upsert a cistron.
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure.
    fn put_cistron(&mut self, edge: &Cistron) -> Result<()>;

    /// Iterate all stored cistrons.
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure.
    fn iter_cistrons(&self) -> Result<Vec<Cistron>>;

    /// Load the working DNA GRN (convenience over fine-grained reads).
    ///
    /// # Errors
    ///
    /// Returns an error on I/O or corruption.
    fn load_grn(&self) -> Result<Grn>
    where
        Self: Sized,
    {
        Grn::load(self)
    }

    /// Persist the DNA GRN (convenience over fine-grained writes).
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure.
    fn save_grn(&mut self, grn: &Grn) -> Result<()>
    where
        Self: Sized,
    {
        grn.persist(self)
    }

    /// Append a causal event (Phase 1 or Phase 2 log).
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure.
    fn append_causal(&mut self, node: CausalNode) -> Result<()>;

    /// Replace the entire causal DAG (flush path).
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure.
    fn replace_causal_dag(&mut self, dag: CausalDag) -> Result<()>;

    /// Load the causal DAG.
    ///
    /// # Errors
    ///
    /// Returns an error on I/O or corruption.
    fn load_causal_dag(&self) -> Result<CausalDag>;

    /// Attach an event log to be retained by the next [`Self::snapshot`].
    fn prepare_snapshot_log(&mut self, log: Option<CausalEventLog>) {
        let _ = log;
    }

    /// Take an event log restored by the last [`Self::restore`] / [`Self::branch`].
    fn take_restored_event_log(&mut self) -> Option<CausalEventLog> {
        None
    }

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
