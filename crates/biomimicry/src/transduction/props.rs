//! Property / unit tests for transduction cascades (M4 / M11).

use crate::cell::{Cell, CellId, LifecycleState};
use crate::genesis::{GeneId, arith_dna, compile, toy_dna};
use crate::metabolism::Transducer;
use crate::signal::{CausalStamp, Payload, Scope, Signal, SignalType, Value};
use crate::transduction::{
    ArithOp, Cascade, CascadeTransducer, FoldSpec, TransductionFn, TransductionFnSpec,
    TransductionKind, TransductionSpec, cascade_from_spec, emit_from_cascade, eval_binary,
};
use proptest::prelude::*;
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
        .find(|g| g.cistron.kind.as_str() == "sensory_spike")
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

fn arb_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Unit),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::Int),
        "[a-z]{0,8}".prop_map(Value::text),
    ];
    leaf.prop_recursive(4, 32, 4, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::List),
            prop::collection::btree_map("[a-z]{1,4}", inner, 0..4).prop_map(|m| {
                Value::Record(
                    m.into_iter()
                        .map(|(k, v)| (smol_str::SmolStr::new(k), v))
                        .collect(),
                )
            }),
        ]
    })
}

proptest! {
    #[test]
    fn p1_codec_roundtrip(v in arb_value()) {
        prop_assume!(v.depth() <= crate::signal::MAX_VALUE_DEPTH);
        let bytes = v.encode().unwrap();
        let decoded = Value::decode(&bytes).unwrap();
        prop_assert_eq!(v, decoded);
    }

    #[test]
    fn p3_arith_mul_determinism(a in -10_000i64..10_000, b in -10_000i64..10_000) {
        let once = eval_binary(ArithOp::Mul, a, b).unwrap();
        let twice = eval_binary(ArithOp::Mul, a, b).unwrap();
        prop_assert_eq!(once, twice);
        let expected = {
            let numer = i128::from(a) * i128::from(b);
            crate::transduction::round_half_away(numer, 1000)
        };
        prop_assert_eq!(once, expected);
    }

    #[test]
    fn p3_div_by_zero_always_errors(a in any::<i64>()) {
        prop_assert!(eval_binary(ArithOp::Div, a, 0).is_err());
    }
}

#[test]
fn p4_cascade_composition_maps() {
    let spec = TransductionSpec::new()
        .with_step(TransductionFnSpec {
            name: "get_a".into(),
            kind: TransductionKind::Map(crate::transduction::MapSpec::Get {
                key: "a".into(),
            }),
            output_kind: "x".into(),
            output_scope: Scope::SelfCell,
            enabled: true,
        })
        .with_step(TransductionFnSpec::arith("neg", ArithOp::Neg, "y"));
    let cascade = cascade_from_spec(&spec);
    let input = Signal::new(
        SignalType::Operational,
        "in",
        Scope::SelfCell,
        Payload::of(Value::record_from([("a", Value::Int(5))]).unwrap()),
        CellId(1),
        CausalStamp(0),
    );
    let genome = compile(&toy_dna()).unwrap();
    let cell = Cell::new(CellId(1), Arc::clone(&genome));
    let outs = cascade.run(&cell.expression, &input).unwrap();
    assert_eq!(outs.len(), 1);
    assert_eq!(outs[0].payload.value().unwrap(), Value::Int(-5));
}

#[test]
fn from_genome_matches_arith_dna_specs() {
    let genome = compile(&arith_dna()).unwrap();
    assert!(!genome.cascades().is_empty());
    let t = CascadeTransducer::from_genome(&genome);
    assert_eq!(t.cascades.len(), genome.cascades().len());
}

#[test]
fn fold_add_arity_two_in_one_call() {
    let step = TransductionFn::identity_echo("f", "total").with_kind(TransductionKind::Fold(
        FoldSpec::arity(ArithOp::Add, 2),
    ));
    let a = Signal::new(
        SignalType::Operational,
        "a",
        Scope::SelfCell,
        Payload::of(Value::Int(3000)),
        CellId(1),
        CausalStamp(1),
    );
    let b = Signal::new(
        SignalType::Operational,
        "b",
        Scope::SelfCell,
        Payload::of(Value::Int(4000)),
        CellId(1),
        CausalStamp(2),
    );
    let out = step.call_many(&[a, b]).unwrap();
    assert_eq!(out[0].payload.value().unwrap(), Value::Int(7000));
}

/// P5: fold application order is `(stamp, SignalId)`, so delivery shuffle is irrelevant.
#[test]
fn p5_fold_delivery_order_independent() {
    let step = TransductionFn::identity_echo("f", "total").with_kind(TransductionKind::Fold(
        FoldSpec::arity(ArithOp::Sub, 3),
    ));
    // Non-commutative Sub: result must be identical regardless of call order
    // because fold_signals sorts by (stamp, id).
    let mk = |stamp: i64, id_salt: u8, v: i64| {
        let mut s = Signal::new(
            SignalType::Operational,
            "x",
            Scope::SelfCell,
            Payload::of(Value::Int(v)),
            CellId(1),
            CausalStamp(stamp),
        );
        // Force distinct ids while keeping stamps as the primary order key.
        s.id = crate::signal::SignalId(u128::from(id_salt));
        s
    };
    let ordered = [mk(1, 1, 10_000), mk(2, 2, 3_000), mk(3, 3, 1_000)];
    let shuffled = [mk(3, 3, 1_000), mk(1, 1, 10_000), mk(2, 2, 3_000)];
    let a = step.call_many(&ordered).unwrap();
    let b = step.call_many(&shuffled).unwrap();
    assert_eq!(a[0].payload.value().unwrap(), b[0].payload.value().unwrap());
    // 10000 - 3000 - 1000 = 6000
    assert_eq!(a[0].payload.value().unwrap(), Value::Int(6_000));
}

/// P2: PartialEq and digest agree (one hash authority).
#[test]
fn p2_one_hash_authority() {
    let a = Payload::of(Value::Int(42)).with_meta("k", "v");
    let b = Payload::of(Value::Int(42)).with_meta("k", "v");
    let c = Payload::of(Value::Int(43));
    assert_eq!(a, b);
    assert_eq!(a.digest(), b.digest());
    assert_ne!(a, c);
    assert_ne!(a.digest(), c.digest());
}

/// P9: typed mismatch is an error, never Ok(empty) for failure.
#[test]
fn p9_no_silent_type_mismatch() {
    let step = TransductionFn::identity_echo("neg", "out").with_kind(TransductionKind::Arith(
        ArithOp::Neg,
    ));
    let bad = Signal::new(
        SignalType::Operational,
        "in",
        Scope::SelfCell,
        Payload::of(Value::Bool(true)),
        CellId(1),
        CausalStamp(0),
    );
    let err = step.call(&bad).unwrap_err();
    assert!(matches!(
        err,
        crate::error::BiomimicryError::ValueTypeMismatch { .. }
    ));
}

