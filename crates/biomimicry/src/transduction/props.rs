//! Property / unit tests for transduction cascades (M4).

use crate::cell::{Cell, CellId, LifecycleState};
use crate::genesis::{GeneId, compile, toy_dna};
use crate::metabolism::Transducer;
use crate::signal::{CausalStamp, Payload, Scope, Signal, SignalType};
use crate::transduction::{Cascade, CascadeTransducer, TransductionFn, emit_from_cascade};
use std::sync::Arc;

#[test]
fn p3_inactive_gene_yields_no_ops() {
    let genome = compile(&toy_dna()).unwrap();
    let cell = Cell::new(CellId(1), Arc::clone(&genome));
    let gene = GeneId(1);
    let t = CascadeTransducer::new().with_cascade(
        gene,
        Cascade::new().with_step(TransductionFn::identity_echo("e", "out")),
    );
    let sig = Signal::new(
        SignalType::Operational,
        "transduce",
        Scope::SelfCell,
        Payload::empty(),
        CellId(1),
        CausalStamp(0),
    );
    assert!(t.transduce(&cell, &sig, gene).is_empty());
}

#[test]
fn p4_cascade_only_when_gene_active() {
    let genome = compile(&toy_dna()).unwrap();
    let mut cell = Cell::new(CellId(1), Arc::clone(&genome));
    cell.try_transition(LifecycleState::Differentiating)
        .unwrap();
    cell.try_transition(LifecycleState::Active).unwrap();
    let spike = genome
        .iter()
        .find(|g| g.hyperedge.kind.as_str() == "sensory_spike")
        .map(|g| g.id)
        .unwrap();
    cell.activate(spike);
    let t = CascadeTransducer::new().with_cascade(
        spike,
        Cascade::new().with_step(TransductionFn::identity_echo("e", "out")),
    );
    let sig = Signal::new(
        SignalType::Operational,
        "transduce",
        Scope::SelfCell,
        Payload::empty(),
        CellId(1),
        CausalStamp(0),
    );
    let ops = t.transduce(&cell, &sig, spike);
    assert_eq!(ops.len(), 1);
}

#[test]
fn emit_from_cascade_restamps() {
    let sig = Signal::new(
        SignalType::Operational,
        "out",
        Scope::Neighbors,
        Payload::empty(),
        CellId(9),
        CausalStamp(0),
    );
    let out = emit_from_cascade(vec![sig], CellId(1), CausalStamp(7)).unwrap();
    assert_eq!(out[0].source, CellId(1));
    assert_eq!(out[0].stamp, CausalStamp(7));
}

#[test]
fn p6_no_todo_cascade_run() {
    let genome = compile(&toy_dna()).unwrap();
    let cell = Cell::new(CellId(1), Arc::clone(&genome));
    let c = Cascade::new().with_step(TransductionFn::identity_echo("e", "out"));
    let sig = Signal::new(
        SignalType::Operational,
        "x",
        Scope::SelfCell,
        Payload::empty(),
        CellId(1),
        CausalStamp(0),
    );
    let outs = c.run(&cell.expression, &sig).unwrap();
    assert_eq!(outs.len(), 1);
}
