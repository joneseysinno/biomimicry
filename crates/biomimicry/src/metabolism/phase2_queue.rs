//! Ordered Phase 2 (operational) operation queue.
//!
//! Never mixed with Phase 1. Same ordering as Phase 1.

use crate::cell::CellId;
use crate::medium::ScheduledOp;
use crate::metabolism::phase1_queue::cmp_scheduled;

/// Phase 2 operational queue.
#[derive(Debug, Clone, Default)]
pub struct Phase2Queue {
    inner: Vec<ScheduledOp>,
}

impl Phase2Queue {
    /// Create an empty queue.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert and keep causal order.
    pub fn push(&mut self, op: ScheduledOp) {
        self.inner.push(op);
        self.sort();
    }

    /// Extend with many ops, then sort once.
    pub fn extend(&mut self, ops: impl IntoIterator<Item = ScheduledOp>) {
        self.inner.extend(ops);
        self.sort();
    }

    /// Pop the front op.
    pub fn pop(&mut self) -> Option<ScheduledOp> {
        if self.inner.is_empty() {
            None
        } else {
            Some(self.inner.remove(0))
        }
    }

    /// Number of pending ops.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Purge ops belonging to `cell` (die).
    pub fn purge_cell(&mut self, cell: CellId) {
        self.inner.retain(|op| op.cell != cell);
    }

    /// Sort by causal order key.
    pub fn sort(&mut self) {
        self.inner.sort_by(cmp_scheduled);
    }

    /// Iterate pending ops.
    pub fn iter(&self) -> impl Iterator<Item = &ScheduledOp> {
        self.inner.iter()
    }
}
