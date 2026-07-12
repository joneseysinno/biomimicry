//! Ordered Phase 1 (regulatory) operation queue.
//!
//! Never mixed with Phase 2. Sorted by `(CausalStamp, SignalId, CellId)`.

use crate::causality::by_causal_order;
use crate::cell::{CellId, Operation};
use crate::medium::ScheduledOp;
use crate::signal::{CausalStamp, SignalId};

/// Phase 1 regulatory queue.
#[derive(Debug, Clone, Default)]
pub struct Phase1Queue {
    inner: Vec<ScheduledOp>,
}

impl Phase1Queue {
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

/// Total order for scheduled ops: signal causal order, then `CellId`.
pub(crate) fn cmp_scheduled(a: &ScheduledOp, b: &ScheduledOp) -> std::cmp::Ordering {
    match (signal_of(&a.op), signal_of(&b.op)) {
        (Some(sa), Some(sb)) => by_causal_order(sa, sb).then(a.cell.cmp(&b.cell)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a
            .cell
            .cmp(&b.cell)
            .then(op_rank(&a.op).cmp(&op_rank(&b.op))),
    }
}

fn signal_of(op: &Operation) -> Option<&crate::signal::Signal> {
    match op {
        Operation::Receive(s) | Operation::Emit(s) => Some(s),
        _ => None,
    }
}

fn op_rank(op: &Operation) -> u8 {
    match op {
        Operation::Express { .. } => 0,
        Operation::Differentiate => 1,
        Operation::Quiesce => 2,
        Operation::Die => 3,
        Operation::DivideFast => 4,
        Operation::DivideSlow => 5,
        Operation::Receive(_) => 6,
        Operation::Transduce(_) => 7,
        Operation::Emit(_) => 8,
    }
}

/// Order key helper for tests.
#[must_use]
pub fn scheduled_key(op: &ScheduledOp) -> (CausalStamp, SignalId, CellId) {
    match signal_of(&op.op) {
        Some(s) => (s.stamp, s.id, op.cell),
        None => (CausalStamp(0), SignalId(0), op.cell),
    }
}
