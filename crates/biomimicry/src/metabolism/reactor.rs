//! Reactor seams — Phase 1 `Regulator` and Phase 2 `Transducer`.
//!
//! M3 ships trivial deterministic impls; M4 swaps in the real rule network and
//! cascade bodies behind the same signatures.

use crate::cell::{Cell, Operation};
use crate::expression::NetworkRegulator;
use crate::genesis::GeneId;
use crate::signal::{Payload, Scope, Signal, SignalKind, SignalType};
use crate::transduction::CascadeTransducer;

/// Expression changes produced by a Phase 1 regulatory step.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExpressionDelta {
    /// Genes to activate.
    pub activate: Vec<GeneId>,
    /// Genes to suppress (remove).
    pub suppress: Vec<GeneId>,
}

impl ExpressionDelta {
    /// Apply this delta to a cell via M2 primitives.
    pub fn apply(&self, cell: &mut Cell) {
        for g in &self.activate {
            cell.activate(*g);
        }
        for g in &self.suppress {
            cell.suppress(*g);
        }
    }
}

/// Phase 1 brain seam: decide expression changes from queued regulatory ops.
pub trait Regulator {
    /// Produce an expression delta for `cell` from the queued Phase 1 ops that
    /// target it.
    fn regulate(&self, cell: &Cell, queued: &[Operation]) -> ExpressionDelta;
}

/// Phase 2 brain seam: decide operations/emissions from a transduction.
pub trait Transducer {
    /// Produce follow-on operations for `cell` given an inbound operational
    /// context signal and the gene that requested transduction.
    fn transduce(&self, cell: &Cell, sig: &Signal, gene: GeneId) -> Vec<Operation>;
}

/// M3 stand-in: apply exactly the explicit `Express { gene, on }` ops.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExplicitRegulator;

impl Regulator for ExplicitRegulator {
    fn regulate(&self, _cell: &Cell, queued: &[Operation]) -> ExpressionDelta {
        let mut delta = ExpressionDelta::default();
        for op in queued {
            if let Operation::Express { gene, on } = op {
                if *on {
                    delta.activate.push(*gene);
                } else {
                    delta.suppress.push(*gene);
                }
            }
        }
        delta
    }
}

/// M3 stand-in: echo a fixed operational follow-on to `SelfCell`.
#[derive(Debug, Clone)]
pub struct EchoTransducer {
    /// Kind label of the follow-on signal.
    pub follow_kind: SignalKind,
}

impl Default for EchoTransducer {
    fn default() -> Self {
        Self {
            follow_kind: SignalKind::new("echo"),
        }
    }
}

impl Transducer for EchoTransducer {
    fn transduce(&self, cell: &Cell, sig: &Signal, gene: GeneId) -> Vec<Operation> {
        let _ = (sig, gene);
        let stamp = cell.peek_stamp();
        let follow = Signal::new(
            SignalType::Operational,
            self.follow_kind.clone(),
            Scope::SelfCell,
            Payload::empty(),
            cell.id,
            stamp,
        );
        vec![Operation::Emit(follow)]
    }
}

/// Pluggable Phase 1 brain (enum — no `Box<dyn>` alloc surprises).
#[derive(Debug, Clone)]
pub enum Phase1Brain {
    /// M3 explicit Express stand-in.
    Explicit(ExplicitRegulator),
    /// M4 rule network.
    Network(NetworkRegulator),
}

impl Default for Phase1Brain {
    fn default() -> Self {
        Self::Explicit(ExplicitRegulator)
    }
}

impl Regulator for Phase1Brain {
    fn regulate(&self, cell: &Cell, queued: &[Operation]) -> ExpressionDelta {
        match self {
            Self::Explicit(r) => r.regulate(cell, queued),
            Self::Network(r) => r.regulate(cell, queued),
        }
    }
}

/// Pluggable Phase 2 brain (enum — no `Box<dyn>` alloc surprises).
#[derive(Debug, Clone)]
pub enum Phase2Brain {
    /// M3 echo stand-in.
    Echo(EchoTransducer),
    /// M4 cascade map.
    Cascade(CascadeTransducer),
}

impl Default for Phase2Brain {
    fn default() -> Self {
        Self::Echo(EchoTransducer::default())
    }
}

impl Transducer for Phase2Brain {
    fn transduce(&self, cell: &Cell, sig: &Signal, gene: GeneId) -> Vec<Operation> {
        match self {
            Self::Echo(t) => t.transduce(cell, sig, gene),
            Self::Cascade(t) => t.transduce(cell, sig, gene),
        }
    }
}
