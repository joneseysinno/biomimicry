//! The Cell struct — wired-but-unpowered automaton (no scheduler drain).

use std::sync::Arc;

use super::{
    BehavioralMode, EnergyBudget, ExpressionState, LifecycleState, Operation, OperationQueue,
    operations_for_matched_gene,
};
use crate::causality::CausalClock;
use crate::error::{BiomimicryError, Result};
use crate::genesis::{GeneId, Genome};
use crate::signal::Signal;

/// Stable handle for a cell within an organism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct CellId(pub u64);

/// Summary of how a cell reacted to an inbound signal.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Reaction {
    /// Genes that matched via `Receptor+`.
    pub matched_genes: Vec<GeneId>,
    /// Operations enqueued as a result.
    pub enqueued: Vec<Operation>,
    /// Genes vetoed by an active `Receptor−`.
    pub vetoed: Vec<GeneId>,
    /// Why the signal was dropped, if it produced no enqueue.
    pub dropped_reason: Option<&'static str>,
}

/// The relational automaton — one living unit in the population.
///
/// Lifecycle is private; the only writer is [`Self::try_transition`].
#[derive(Debug, Clone)]
pub struct Cell {
    /// Stable identity.
    pub id: CellId,
    /// Shared read-only genome catalog.
    pub genome: Arc<Genome>,
    lifecycle: LifecycleState,
    /// Currently expressed genes + cached signaling profile.
    pub expression: ExpressionState,
    /// Behavioral posture (Layer 2 — distinct from lifecycle).
    pub mode: BehavioralMode,
    /// Pending phase-tagged operations (not drained in M2).
    pub pending: OperationQueue,
    /// Bifurcated energy pools.
    pub energy: EnergyBudget,
    /// Per-cell monotonic causal stamp clock.
    stamp_clock: CausalClock,
}

impl Cell {
    /// Create a new cell: `Undifferentiated`, empty expression, idle mode.
    #[must_use]
    pub fn new(id: CellId, genome: Arc<Genome>) -> Self {
        let expression = ExpressionState::new(Arc::clone(&genome));
        Self {
            id,
            genome,
            lifecycle: LifecycleState::Undifferentiated,
            expression,
            mode: BehavioralMode::Idle,
            pending: OperationQueue::new(),
            energy: EnergyBudget::default(),
            stamp_clock: CausalClock::new(),
        }
    }

    /// Current lifecycle state (read-only).
    #[must_use]
    pub fn lifecycle(&self) -> LifecycleState {
        self.lifecycle
    }

    /// Attempt a guarded lifecycle transition — the *only* lifecycle writer.
    ///
    /// # Errors
    ///
    /// Returns [`BiomimicryError::IllegalLifecycleTransition`] when the edge
    /// is not in the legal table; state is left unchanged.
    pub fn try_transition(&mut self, to: LifecycleState) -> Result<()> {
        if !super::lifecycle::is_legal(self.lifecycle, to) {
            return Err(BiomimicryError::IllegalLifecycleTransition {
                from: self.lifecycle,
                to,
            });
        }
        self.lifecycle = to;
        Ok(())
    }

    /// Issue the next causal stamp from this cell's clock.
    pub fn next_stamp(&mut self) -> crate::causality::CausalStamp {
        self.stamp_clock.tick()
    }

    /// Peek at the next stamp without advancing.
    #[must_use]
    pub fn peek_stamp(&self) -> crate::causality::CausalStamp {
        self.stamp_clock.peek()
    }

    /// Activate a gene in expression state (profile cache invalidated).
    pub fn activate(&mut self, gene: GeneId) {
        self.expression.activate(gene);
    }

    /// Suppress (remove) a gene from expression state.
    pub fn suppress(&mut self, gene: GeneId) {
        self.expression.suppress(gene);
    }

    /// Suppress by expressing the complement gene.
    pub fn suppress_by_complement(&mut self, gene: GeneId) {
        self.expression.suppress_by_complement(gene);
    }

    /// Enqueue an operation onto the pending queue.
    pub fn enqueue(&mut self, op: Operation) {
        self.pending.push(op);
    }

    /// Peek at the front of the pending queue.
    #[must_use]
    pub fn peek(&self) -> Option<&Operation> {
        self.pending.peek()
    }

    /// Receive a signal: match receptors → enqueue operations.
    ///
    /// Dead cells drop the dispatch. No queue drain happens here (that's M3).
    pub fn receive(&mut self, signal: &Signal) -> Reaction {
        if self.lifecycle == LifecycleState::Dead {
            return Reaction {
                dropped_reason: Some("dead-cell-dispatch"),
                ..Reaction::default()
            };
        }

        let m = self.expression.match_receptors(signal);
        if m.matched.is_empty() {
            return Reaction {
                matched_genes: Vec::new(),
                enqueued: Vec::new(),
                vetoed: m.vetoed,
                dropped_reason: Some("receptor-mismatch"),
            };
        }

        let mut enqueued = Vec::new();
        let receive_op = Operation::Receive(signal.clone());
        self.enqueue(receive_op.clone());
        enqueued.push(receive_op);

        for gene_id in &m.matched {
            for op in operations_for_matched_gene(*gene_id, &self.genome, signal) {
                self.enqueue(op.clone());
                enqueued.push(op);
            }
        }

        Reaction {
            matched_genes: m.matched,
            enqueued,
            vetoed: m.vetoed,
            dropped_reason: None,
        }
    }
}
