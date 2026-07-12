//! Property tests P1–P6 for cell receive / lifecycle / expression.

#![cfg(test)]

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use proptest::prelude::*;

use crate::cell::fixture::{active_sensory_cell, sensory_genome, trigger_signal};
use crate::cell::lifecycle::{LifecycleState, all_states, is_legal};
use crate::cell::{Cell, CellId};
use crate::genesis::{
    DimensionVector, EndpointPolarity, Hyperedge, Primitive, PrimitiveNode, SpatialHypergraph,
    compile, endpoint,
};
use crate::signal::{CausalStamp, Payload, Scope, Signal, SignalType};

fn activate_path(cell: &mut Cell) {
    cell.try_transition(LifecycleState::Differentiating)
        .unwrap();
    cell.try_transition(LifecycleState::Active).unwrap();
}

fn force_lifecycle(cell: &mut Cell, target: LifecycleState) -> bool {
    if cell.lifecycle() == target {
        return true;
    }
    let mut prev: HashMap<LifecycleState, Option<(LifecycleState, LifecycleState)>> =
        HashMap::new();
    let start = cell.lifecycle();
    prev.insert(start, None);
    let mut q = VecDeque::from([start]);
    while let Some(s) = q.pop_front() {
        if s == target {
            break;
        }
        for to in all_states() {
            if is_legal(s, to) && !prev.contains_key(&to) {
                prev.insert(to, Some((s, to)));
                q.push_back(to);
            }
        }
    }
    if !prev.contains_key(&target) {
        return false;
    }
    let mut path = Vec::new();
    let mut cur = target;
    while let Some(Some((from, to))) = prev.get(&cur).copied() {
        path.push(to);
        cur = from;
        if from == start {
            break;
        }
    }
    path.reverse();
    for to in path {
        if cell.try_transition(to).is_err() {
            return false;
        }
    }
    cell.lifecycle() == target
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// P4 · lifecycle table is exactly the legal set; Dead is a sink.
    #[test]
    fn p4_lifecycle_table(
        from_i in 0usize..5,
        to_i in 0usize..5,
    ) {
        let states = all_states();
        let from = states[from_i];
        let to = states[to_i];
        let (genome, _) = sensory_genome();
        let mut cell = Cell::new(CellId(1), genome);
        if from != LifecycleState::Undifferentiated {
            let path_ok = force_lifecycle(&mut cell, from);
            prop_assume!(path_ok);
        }
        let before = cell.lifecycle();
        prop_assert_eq!(before, from);
        let result = cell.try_transition(to);
        if is_legal(from, to) {
            prop_assert!(result.is_ok());
            prop_assert_eq!(cell.lifecycle(), to);
        } else {
            prop_assert!(result.is_err());
            prop_assert_eq!(cell.lifecycle(), from);
        }
    }

    /// P5 · profile ↔ expression coherence after activate/suppress sequences.
    #[test]
    fn p5_profile_coherent(
        ops in proptest::collection::vec(any::<bool>(), 1..8)
    ) {
        let (genome, spike) = sensory_genome();
        let gate = genome.genes_of_kind("local_gate").next().unwrap();
        let mut cell = Cell::new(CellId(1), Arc::clone(&genome));
        activate_path(&mut cell);
        let genes = [spike, gate];
        for (i, on) in ops.into_iter().enumerate() {
            let g = genes[i % genes.len()];
            if on {
                cell.activate(g);
            } else {
                cell.suppress(g);
            }
        }
        let fresh = cell.expression.recompute_profile_fresh();
        prop_assert_eq!(cell.expression.profile(), &fresh);
    }

    /// P1 · match soundness (enqueue iff Receptor+ match and no veto).
    #[test]
    fn p1_match_soundness(stamp in 0i64..32) {
        let (mut cell, spike) = active_sensory_cell();
        let matching = trigger_signal(CellId(7), CausalStamp(stamp));
        let r = cell.receive(&matching);
        prop_assert!(r.matched_genes.contains(&spike));
        prop_assert!(!r.enqueued.is_empty());

        let (mut cell2, _) = active_sensory_cell();
        let mismatch = Signal::new(
            SignalType::Regulatory,
            "nope",
            Scope::Systemwide,
            Payload::empty(),
            CellId(7),
            CausalStamp(stamp),
        );
        let r2 = cell2.receive(&mismatch);
        prop_assert!(r2.matched_genes.is_empty());
        prop_assert!(r2.enqueued.is_empty());
    }

    /// P2 · receptor-relativity: same signal, different expression → different enqueue.
    #[test]
    fn p2_receptor_relativity(_u in Just(())) {
        let (genome, spike) = sensory_genome();
        let mut a = Cell::new(CellId(1), Arc::clone(&genome));
        let mut b = Cell::new(CellId(2), Arc::clone(&genome));
        activate_path(&mut a);
        activate_path(&mut b);
        a.activate(spike);
        let sig = trigger_signal(CellId(9), CausalStamp(0));
        let ra = a.receive(&sig);
        let rb = b.receive(&sig);
        prop_assert!(!ra.enqueued.is_empty());
        prop_assert!(rb.enqueued.is_empty());
    }

    /// P6 · complement inhibition removes prior match.
    #[test]
    fn p6_complement_inhibition(_u in Just(())) {
        let (mut cell, spike) = active_sensory_cell();
        let sig = trigger_signal(CellId(3), CausalStamp(0));
        let before = cell.receive(&sig);
        prop_assert!(!before.enqueued.is_empty());
        while cell.pending.pop().is_some() {}
        cell.suppress_by_complement(spike);
        let after = cell.receive(&sig);
        prop_assert!(!cell.expression.is_active(spike));
        prop_assert!(after.enqueued.is_empty());
    }
}

/// P3 · veto: active Receptor− clears a previously matching enqueue.
#[test]
fn p3_veto_removes_match() {
    let mut g = SpatialHypergraph::new();
    let r_pos = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0]));
    let r_neg = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([1]));
    let expr = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2]));
    g.add_node(r_pos.clone()).unwrap();
    g.add_node(r_neg.clone()).unwrap();
    g.add_node(expr.clone()).unwrap();
    g.add_hyperedge(Hyperedge::new(
        "listen",
        vec![
            endpoint(&r_pos, EndpointPolarity::Positive, "ping", None),
            endpoint(&expr, EndpointPolarity::Positive, "go", None),
        ],
    ));
    g.add_hyperedge(Hyperedge::new(
        "block",
        vec![endpoint(&r_neg, EndpointPolarity::Negative, "ping", None)],
    ));
    let genome = compile(&g).unwrap();
    let listen = genome
        .iter()
        .find(|gene| {
            gene.hyperedge.kind.as_str() == "listen"
                && gene.hyperedge.endpoints.iter().any(|ep| {
                    ep.primitive == Primitive::Receptor && ep.polarity == EndpointPolarity::Positive
                })
        })
        .map(|g| g.id)
        .expect("listen Receptor+");
    let block = genome
        .iter()
        .find(|gene| {
            gene.hyperedge.kind.as_str() == "block"
                && gene.hyperedge.endpoints.iter().any(|ep| {
                    ep.primitive == Primitive::Receptor && ep.polarity == EndpointPolarity::Negative
                })
        })
        .map(|g| g.id)
        .expect("block Receptor−");

    let mut cell = Cell::new(CellId(1), genome);
    activate_path(&mut cell);
    cell.activate(listen);
    let sig = Signal::new(
        SignalType::Regulatory,
        "ping",
        Scope::SelfCell,
        Payload::empty(),
        CellId(0),
        CausalStamp(0),
    );
    let before = cell.receive(&sig);
    assert!(!before.enqueued.is_empty());
    while cell.pending.pop().is_some() {}

    cell.activate(block);
    let after = cell.receive(&sig);
    assert!(
        after.enqueued.is_empty(),
        "expected global veto from active Receptor−, got {after:?}"
    );
    assert!(!after.vetoed.is_empty());
}
