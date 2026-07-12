//! Causal event log — the replay artifact and M7 DAG seed.

use crate::cell::CellId;
use crate::signal::{CausalStamp, SignalId};

/// One stamped parent→child causal edge produced by the scheduler/medium.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CausalEvent {
    /// Parent signal, if any (e.g. the emit that caused a receive).
    pub parent: Option<SignalId>,
    /// Child / resulting signal id.
    pub child: SignalId,
    /// Cell that owns the child event.
    pub cell: CellId,
    /// Causal stamp of the child.
    pub stamp: CausalStamp,
    /// Short tag (`deliver`, `emit`, `transduce`, …).
    pub tag: String,
}

/// Ordered in-memory causal event stream (byte-identical under fixed seed).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CausalEventLog {
    events: Vec<CausalEvent>,
}

impl CausalEventLog {
    /// Create an empty log.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an event (append-only; order is the replay sequence).
    pub fn push(&mut self, event: CausalEvent) {
        self.events.push(event);
    }

    /// Borrow the ordered events.
    #[must_use]
    pub fn events(&self) -> &[CausalEvent] {
        &self.events
    }

    /// Number of events.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the log is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear the log.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}
