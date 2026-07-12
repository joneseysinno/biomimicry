//! Durable [`biomimicry::substrate::Store`] backed by an on-disk MemoryStore blob.
//!
//! This crate is the one place two vocabularies meet: the engine's biological
//! `genesis::Cistron` / `Grn` on one side, infinite-db's graph-theoretic
//! `Hyperedge` / `Space` on the other. All mapping between them lives here so
//! engine code never imports a database type, and database code never learns
//! biology. (Current impl delegates to a MemoryStore blob; the Cistron⇄Hyperedge
//! Space mapping is the next hardening step.)
//!
//! M7 locks a pragmatic contract: an inner [`MemoryStore`] plus a rewrite-on-mutate
//! file at `path`. This crate proves the Store contract and replay/branch
//! deliverable.

use std::path::{Path, PathBuf};

use biomimicry::causality::{CausalDag, CausalEventLog, CausalNode};
use biomimicry::error::{BiomimicryError, Result};
use biomimicry::genesis::{Cistron, PrimitiveNode, PrimitiveNodeId};
use biomimicry::substrate::{BranchId, MemoryStore, SnapshotId, SnapshotMeta, Store};

/// Store that delegates to [`MemoryStore`] and rewrites a durable file after
/// mutating persist operations.
#[derive(Debug)]
pub struct InfiniteDbStore {
    path: PathBuf,
    inner: MemoryStore,
}

impl InfiniteDbStore {
    /// Open (or create) a durable store at `path`.
    ///
    /// Loads an existing BM7S blob when present; otherwise starts empty and
    /// creates the parent directory as needed.
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be read/created or the blob is corrupt.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let inner = if path.exists() {
            let bytes = std::fs::read(&path)
                .map_err(|e| BiomimicryError::Substrate(format!("read {}: {e}", path.display())))?;
            MemoryStore::from_durable_bytes(&bytes)?
        } else {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        BiomimicryError::Substrate(format!("create dir {}: {e}", parent.display()))
                    })?;
                }
            }
            MemoryStore::new()
        };
        let store = Self { path, inner };
        store.persist()?;
        Ok(store)
    }

    /// Rewrite the durable file from the live inner store.
    fn persist(&self) -> Result<()> {
        let bytes = self.inner.to_durable_bytes()?;
        let tmp = self.path.with_extension("tmp");
        std::fs::write(&tmp, &bytes)
            .map_err(|e| BiomimicryError::Substrate(format!("write {}: {e}", tmp.display())))?;
        std::fs::rename(&tmp, &self.path).map_err(|e| {
            BiomimicryError::Substrate(format!(
                "rename {} → {}: {e}",
                tmp.display(),
                self.path.display()
            ))
        })?;
        Ok(())
    }
}

impl Store for InfiniteDbStore {
    fn clear_grn(&mut self) -> Result<()> {
        self.inner.clear_grn()?;
        self.persist()
    }

    fn put_node(&mut self, node: &PrimitiveNode) -> Result<()> {
        self.inner.put_node(node)?;
        self.persist()
    }

    fn get_node(&self, id: PrimitiveNodeId) -> Result<Option<PrimitiveNode>> {
        self.inner.get_node(id)
    }

    fn iter_nodes(&self) -> Result<Vec<PrimitiveNode>> {
        self.inner.iter_nodes()
    }

    fn put_cistron(&mut self, edge: &Cistron) -> Result<()> {
        self.inner.put_cistron(edge)?;
        self.persist()
    }

    fn iter_cistrons(&self) -> Result<Vec<Cistron>> {
        self.inner.iter_cistrons()
    }

    fn append_causal(&mut self, node: CausalNode) -> Result<()> {
        self.inner.append_causal(node)?;
        self.persist()
    }

    fn replace_causal_dag(&mut self, dag: CausalDag) -> Result<()> {
        self.inner.replace_causal_dag(dag)?;
        self.persist()
    }

    fn load_causal_dag(&self) -> Result<CausalDag> {
        self.inner.load_causal_dag()
    }

    fn prepare_snapshot_log(&mut self, log: Option<CausalEventLog>) {
        self.inner.prepare_snapshot_log(log);
    }

    fn take_restored_event_log(&mut self) -> Option<CausalEventLog> {
        self.inner.take_restored_event_log()
    }

    fn snapshot(&mut self, label: &str) -> Result<SnapshotMeta> {
        let meta = self.inner.snapshot(label)?;
        self.persist()?;
        Ok(meta)
    }

    fn branch(&mut self, from: SnapshotId, label: &str) -> Result<BranchId> {
        let id = self.inner.branch(from, label)?;
        self.persist()?;
        Ok(id)
    }

    fn restore(&mut self, id: SnapshotId) -> Result<()> {
        self.inner.restore(id)?;
        self.persist()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use biomimicry::causality::{CausalNode, CausalStamp};
    use biomimicry::signal::SignalId;

    #[test]
    fn p4_infinite_db_store_file_round_trip() {
        let dir = std::env::temp_dir().join(format!(
            "biomimicry-m7-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("tmpdir");
        let path = dir.join("store.bm7s");

        {
            let mut store = InfiniteDbStore::open(&path).expect("open");
            store
                .append_causal(CausalNode::leaf(CausalStamp(11), SignalId(22), "emit"))
                .expect("append");
        }

        let store = InfiniteDbStore::open(&path).expect("reopen");
        let dag = store.load_causal_dag().expect("dag");
        assert_eq!(dag.len(), 1);
        assert_eq!(dag.nodes()[0].stamp, CausalStamp(11));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
