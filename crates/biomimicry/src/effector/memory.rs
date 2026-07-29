//! In-memory [`EffectorSink`] — M11 default.

use std::collections::BTreeMap;

use crate::effector::{EffectorId, EffectorSink};
use crate::error::Result;
use crate::signal::{CausalStamp, Value};

/// Organism-owned in-memory effector sink (`BTreeMap`, never `HashMap`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MemoryEffectorSink {
    working: BTreeMap<EffectorId, Value>,
    committed: BTreeMap<EffectorId, Value>,
    /// Last write stamp per effector (debug / causal).
    stamps: BTreeMap<EffectorId, CausalStamp>,
}

impl MemoryEffectorSink {
    /// Empty sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Promote working → committed (called when the commitment gate opens).
    pub fn commit_working(&mut self) {
        self.committed = self.working.clone();
    }

    /// Committed snapshot (unchanged by stimulations until checkpoint).
    #[must_use]
    pub fn committed_snapshot(&self) -> BTreeMap<EffectorId, Value> {
        self.committed.clone()
    }

    /// Diff of working against a prior snapshot.
    #[must_use]
    pub fn diff_from(&self, prior: &BTreeMap<EffectorId, Value>) -> BTreeMap<EffectorId, Value> {
        let mut out = BTreeMap::new();
        for (id, value) in &self.working {
            if prior.get(id) != Some(value) {
                out.insert(*id, value.clone());
            }
        }
        out
    }
}

impl EffectorSink for MemoryEffectorSink {
    fn write(&mut self, id: EffectorId, value: Value, stamp: CausalStamp) -> Result<()> {
        self.working.insert(id, value);
        self.stamps.insert(id, stamp);
        Ok(())
    }

    fn read(&self, id: EffectorId) -> Option<&Value> {
        self.working.get(&id)
    }

    fn snapshot(&self) -> BTreeMap<EffectorId, Value> {
        self.working.clone()
    }
}
