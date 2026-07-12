//! Property / unit tests for the expression engine (M4).

use crate::cell::{Cell, CellId, Operation};
use crate::expression::apply::resolve_conflicts;
use crate::expression::{
    NetworkRegulator, RegulatoryRule, RuleCondition, RuleNetwork, apply_delta,
};
use crate::genesis::{GeneId, compile, toy_dna};
use crate::metabolism::{ExplicitRegulator, ExpressionDelta, Regulator};
use crate::signal::{CausalStamp, Payload, Scope, Signal, SignalKind, SignalType};
use std::sync::Arc;

#[test]
fn p1_rule_order_determinism() {
    let genome = compile(&toy_dna()).unwrap();
    let g_a = GeneId(1);
    let g_b = GeneId(2);
    let network = RuleNetwork::new()
        .with_rule(RegulatoryRule::new("first").with_express([g_a]))
        .with_rule(RegulatoryRule::new("second").with_express([g_b]));
    let cell = Cell::new(CellId(1), Arc::clone(&genome));
    let d1 = network.evaluate(&cell, &[]);
    let d2 = network.evaluate(&cell, &[]);
    assert_eq!(d1, d2);
    assert_eq!(d1.activate, vec![g_a, g_b]);
}

#[test]
fn p2_suppress_wins_conflict() {
    let g = GeneId(9);
    let delta = ExpressionDelta {
        activate: vec![g],
        suppress: vec![g],
    };
    let r = resolve_conflicts(delta);
    assert!(r.activate.is_empty());
    assert_eq!(r.suppress, vec![g]);
}

#[test]
fn p5_empty_network_compat_with_explicit() {
    let genome = compile(&toy_dna()).unwrap();
    let cell = Cell::new(CellId(1), Arc::clone(&genome));
    let gene = GeneId(3);
    let queued = [Operation::Express { gene, on: true }];
    let network = NetworkRegulator::new(RuleNetwork::new());
    let explicit = ExplicitRegulator;
    assert_eq!(
        network.regulate(&cell, &queued),
        explicit.regulate(&cell, &queued)
    );
}

#[test]
fn apply_delta_mutates_expression() {
    let genome = compile(&toy_dna()).unwrap();
    let mut cell = Cell::new(CellId(1), Arc::clone(&genome));
    let spike = genome
        .iter()
        .find(|g| g.cistron.kind.as_str() == "sensory_spike")
        .map(|g| g.id)
        .unwrap();
    apply_delta(
        &mut cell.expression,
        &ExpressionDelta {
            activate: vec![spike],
            suppress: vec![],
        },
    );
    assert!(cell.expression.is_active(spike));
}

#[test]
fn signal_kind_uses_last_inbound() {
    let genome = compile(&toy_dna()).unwrap();
    let mut cell = Cell::new(CellId(1), Arc::clone(&genome));
    cell.last_inbound_kind = Some(SignalKind::new("trigger"));
    let rule = RegulatoryRule::new("r")
        .with_condition(RuleCondition::SignalKind(SignalKind::new("trigger")))
        .with_express([GeneId(1)]);
    assert!(rule.matches(&cell, &[]));
}

#[test]
fn no_todo_smoke_evaluate() {
    let genome = compile(&toy_dna()).unwrap();
    let cell = Cell::new(CellId(1), Arc::clone(&genome));
    let sig = Signal::new(
        SignalType::Regulatory,
        "trigger",
        Scope::SelfCell,
        Payload::empty(),
        CellId(1),
        CausalStamp(0),
    );
    let net = RuleNetwork::new().with_rule(
        RegulatoryRule::new("r")
            .with_condition(RuleCondition::SignalKind(SignalKind::new("trigger"))),
    );
    let _ = net.evaluate(&cell, &[Operation::Receive(sig)]);
}
