//! Typed pub/sub delivery through the signaling medium.
//!
//! Given an `Emit`, resolve targets by surface intersection + scope, enqueue
//! `Receive` ops (returned to the scheduler), skip `Dead` targets, and append
//! causal `(parent → child)` events to the log.

use crate::causality::{CausalEvent, CausalEventLog};
use crate::cell::{Cell, CellId, LifecycleState, Operation};
use crate::error::Result;
use crate::ganglion::Ganglion;
use crate::medium::scoping::resolve_targets;
use crate::sensorium::{ReadoutCollector, SignalSample};
use crate::signal::Signal;

/// An operation scheduled against a specific cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledOp {
    /// Target / owning cell.
    pub cell: CellId,
    /// Operation to run.
    pub op: Operation,
}

/// Delivery fabric for signals between cells.
#[derive(Debug, Default)]
pub struct Medium {
    /// In-flight scheduled receives not yet absorbed into scheduler queues.
    pending_receives: Vec<ScheduledOp>,
}

/// Alias preserved for the organism aggregate.
pub type Delivery = Medium;

impl Medium {
    /// Create a new delivery fabric.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Deliver an emitted signal: resolve targets → schedule `Receive` ops.
    ///
    /// When `signal.payload` is observation-tagged, also records a passive
    /// sample on `collector` (if provided).
    ///
    /// # Errors
    ///
    /// Propagates scope resolution errors (none for Cluster in M6).
    pub fn deliver(
        &mut self,
        source: &Cell,
        population: &[Cell],
        signal: &Signal,
        log: &mut CausalEventLog,
        ganglia: &[Ganglion],
        collector: Option<&mut ReadoutCollector>,
    ) -> Result<Vec<ScheduledOp>> {
        if let Some(col) = collector {
            if signal.payload.is_observation() {
                col.observe(SignalSample {
                    source: source.id.0,
                    payload: signal.payload.clone(),
                });
            }
        }
        let targets = resolve_targets(source, population, signal, ganglia)?;
        let mut out = Vec::with_capacity(targets.len());
        for tid in targets {
            let Some(target) = population.iter().find(|c| c.id == tid) else {
                continue;
            };
            if target.lifecycle() == LifecycleState::Dead {
                continue;
            }
            log.push(CausalEvent {
                parent: Some(signal.id),
                child: signal.id,
                cell: tid,
                stamp: signal.stamp,
                tag: "deliver".into(),
            });
            out.push(ScheduledOp {
                cell: tid,
                op: Operation::Receive(signal.clone()),
            });
        }
        Ok(out)
    }

    /// Drop in-flight scheduled ops belonging to a dying cell.
    pub fn drop_in_flight(&mut self, cell: CellId) {
        self.pending_receives.retain(|op| op.cell != cell);
    }

    /// Drain any buffered receives (test helper).
    pub fn take_pending(&mut self) -> Vec<ScheduledOp> {
        std::mem::take(&mut self.pending_receives)
    }
}
