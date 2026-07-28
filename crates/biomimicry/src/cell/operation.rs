//! Operations enqueued for a cell and the phase-tagged operation queue.
//!
//! The cell holds **one** queue; each op knows its [`Phase`] so M3 can split
//! into Phase 1 / Phase 2 drains without retrofitting.

use std::collections::VecDeque;

use crate::error::{BiomimicryError, Result};
use crate::genesis::GeneId;
use crate::signal::{Phase, Signal};

/// A unit of work the cell may perform (Part II.6 mechanism verbs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    /// Record an inbound signal (Phase 1).
    Receive(Signal),
    /// Activate (`on = true`) or deactivate (`on = false`) a gene (Phase 1).
    Express {
        /// Target gene.
        gene: GeneId,
        /// Whether to turn the gene on.
        on: bool,
    },
    /// Run transduction for a gene (Phase 2) — execution deferred to M3/M4.
    ///
    /// `input` is the inbound signal that triggered the match (operands / meta).
    Transduce {
        /// Gene whose cascade to run.
        gene: GeneId,
        /// Trigger signal (payload available to [`crate::transduction::TransductionFn`]).
        input: Signal,
    },
    /// Emit an outbound signal (Phase 2).
    Emit(Signal),
    /// Enter differentiating lifecycle path (Phase 1 control).
    Differentiate,
    /// Fast division — population op, execution deferred (M3/M6).
    DivideFast,
    /// Slow division — population op, execution deferred (M3/M6).
    DivideSlow,
    /// Enter quiescent lifecycle (Phase 1 control).
    Quiesce,
    /// Die — terminal; execution deferred for medium cleanup (M3).
    Die,
}

impl Operation {
    /// Scheduler phase that owns this operation.
    #[must_use]
    pub const fn phase(&self) -> Phase {
        match self {
            Self::Receive(_) | Self::Transduce { .. } | Self::Emit(_) => Phase::Phase2,
            Self::Express { .. }
            | Self::Differentiate
            | Self::DivideFast
            | Self::DivideSlow
            | Self::Quiesce
            | Self::Die => Phase::Phase1,
        }
    }

    /// Whether M2 can *inspect/enqueue* this op as a fully local side effect.
    ///
    /// Divide*/Transduce *execution* is typed-deferred; they may still sit on
    /// the queue for the scheduler to pick up later.
    #[must_use]
    pub const fn is_inspectable_in_m2(&self) -> bool {
        matches!(
            self,
            Self::Receive(_) | Self::Express { .. } | Self::Emit(_) | Self::Quiesce
        )
    }

    /// Attempt to *execute* this operation in M2 (pre-scheduler).
    ///
    /// Divide* stays deferred to M6. Transduce/Die execution is the scheduler's
    /// job (M3+); M2 still reports them unavailable for hand-execution.
    ///
    /// # Errors
    ///
    /// Returns `OperationUnavailable` for deferred ops.
    pub fn execute_in_m2(&self) -> Result<()> {
        match self {
            Self::DivideFast | Self::DivideSlow => Err(BiomimicryError::OperationUnavailable {
                op: self.op_name(),
                since_milestone: 6,
            }),
            Self::Transduce { .. } => Err(BiomimicryError::OperationUnavailable {
                op: self.op_name(),
                since_milestone: 3,
            }),
            Self::Receive(_)
            | Self::Express { .. }
            | Self::Emit(_)
            | Self::Quiesce
            | Self::Differentiate
            | Self::Die => Ok(()),
        }
    }

    /// Stable name for error reporting.
    #[must_use]
    pub const fn op_name(&self) -> &'static str {
        match self {
            Self::Receive(_) => "receive",
            Self::Express { .. } => "express",
            Self::Transduce { .. } => "transduce",
            Self::Emit(_) => "emit",
            Self::Differentiate => "differentiate",
            Self::DivideFast => "divide-fast",
            Self::DivideSlow => "divide-slow",
            Self::Quiesce => "quiesce",
            Self::Die => "die",
        }
    }
}

/// FIFO queue of pending operations for a cell.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OperationQueue {
    inner: VecDeque<Operation>,
}

impl OperationQueue {
    /// Create an empty queue.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueue an operation.
    pub fn push(&mut self, op: Operation) {
        self.inner.push_back(op);
    }

    /// Dequeue the next operation, if any.
    ///
    /// M2 tests may pop for assertions; the live motor that drains is M3.
    pub fn pop(&mut self) -> Option<Operation> {
        self.inner.pop_front()
    }

    /// Peek at the front without removing.
    #[must_use]
    pub fn peek(&self) -> Option<&Operation> {
        self.inner.front()
    }

    /// Iterate pending ops in order.
    pub fn iter(&self) -> impl Iterator<Item = &Operation> {
        self.inner.iter()
    }

    /// Number of pending operations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether the queue is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Drain into a vector (test helper).
    #[must_use]
    pub fn to_vec(&self) -> Vec<Operation> {
        self.inner.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deferred_ops_return_unavailable_not_panic() {
        let err = Operation::DivideFast.execute_in_m2().unwrap_err();
        assert!(matches!(
            err,
            BiomimicryError::OperationUnavailable {
                op: "divide-fast",
                since_milestone: 6
            }
        ));
        assert!(
            Operation::Express {
                gene: GeneId(0),
                on: true
            }
            .execute_in_m2()
            .is_ok()
        );
    }

    #[test]
    fn phase_tags() {
        assert_eq!(
            Operation::Express {
                gene: GeneId(1),
                on: true
            }
            .phase(),
            Phase::Phase1
        );
        let stub = Signal::new(
            crate::signal::SignalType::Operational,
            "stub",
            crate::signal::Scope::SelfCell,
            crate::signal::Payload::empty(),
            crate::cell::CellId(0),
            crate::signal::CausalStamp(0),
        );
        assert_eq!(
            Operation::Transduce {
                gene: GeneId(1),
                input: stub,
            }
            .phase(),
            Phase::Phase2
        );
    }
}
