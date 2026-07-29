//! Organism persistence — flush, checkpoint, restore, fork, diff.

use crate::causality::{CausalDag, CausalDiff, CommitmentGate, diff_dags, log_to_dag};
use crate::error::{BiomimicryError, Result};
use crate::organism::Organism;
use crate::substrate::{BranchId, SnapshotId, SnapshotMeta, Store};

impl<S: Store> Organism<S> {
    /// Open the commitment gate (allows checkpoint without `force`).
    pub fn open_commit_gate(&mut self) {
        self.commit_gate.open = true;
        self.effect_sink.commit_working();
    }

    /// Close the commitment gate.
    pub fn close_commit_gate(&mut self) {
        self.commit_gate.open = false;
    }

    /// Borrow the commitment gate.
    #[must_use]
    pub fn commit_gate(&self) -> &CommitmentGate {
        &self.commit_gate
    }

    /// Convert the scheduler event log into a DAG and replace the store causal half.
    ///
    /// # Errors
    ///
    /// Returns an error if the store write fails.
    pub fn flush_causal(&mut self) -> Result<()> {
        let dag = log_to_dag(&self.scheduler.log);
        self.store.replace_causal_dag(dag)
    }

    /// Flush causal state and take a named store snapshot.
    ///
    /// Requires an open commitment gate unless `force` is true.
    ///
    /// # Errors
    ///
    /// Returns [`BiomimicryError::CommitmentGateClosed`] when the gate is closed
    /// and `force` is false, or a store error.
    pub fn checkpoint(&mut self, label: &str, force: bool) -> Result<SnapshotMeta> {
        if !force && !self.commit_gate.try_commit() {
            return Err(BiomimicryError::CommitmentGateClosed);
        }
        self.effect_sink.commit_working();
        self.flush_causal()?;
        self.store
            .prepare_snapshot_log(Some(self.scheduler.log.clone()));
        self.store.snapshot(label)
    }

    /// Restore store state from a snapshot and refresh the in-memory event log.
    ///
    /// # Errors
    ///
    /// Returns an error if the snapshot is unknown or I/O fails.
    pub fn restore_checkpoint(&mut self, id: SnapshotId) -> Result<()> {
        self.store.restore(id)?;
        if let Some(log) = self.store.take_restored_event_log() {
            self.scheduler.log = log;
        }
        Ok(())
    }

    /// Fork a counterfactual branch from a snapshot (store working tip updates).
    ///
    /// # Errors
    ///
    /// Returns an error if the snapshot is unknown or I/O fails.
    pub fn fork_branch(&mut self, from: SnapshotId, label: &str) -> Result<BranchId> {
        let id = self.store.branch(from, label)?;
        if let Some(log) = self.store.take_restored_event_log() {
            self.scheduler.log = log;
        }
        Ok(id)
    }

    /// Diff two causal DAGs by stamp sets.
    #[must_use]
    pub fn diff_causal(a: &CausalDag, b: &CausalDag) -> CausalDiff {
        diff_dags(a, b)
    }

    /// Load the store's causal DAG view.
    ///
    /// # Errors
    ///
    /// Returns an error on store failure.
    pub fn load_causal_dag(&self) -> Result<CausalDag> {
        self.store.load_causal_dag()
    }
}
