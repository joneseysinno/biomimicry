//! Property tests P1–P7 for metabolism + medium.

#![cfg(test)]

use proptest::prelude::*;

use crate::cell::{CellId, LifecycleState, Operation};
use crate::error::BiomimicryError;
use crate::medium::ScheduledOp;
use crate::medium::scoping::{resolve_targets, resolve_targets_bruteforce};
use crate::metabolism::fixture::{seeded_run_ready, sensory_population, systemwide_trigger};
use crate::metabolism::{Cadence, OrganismEnergy, Population};
use crate::signal::{CausalStamp, Payload, Scope, Signal, SignalType};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// P1 · determinism: same seed → identical logs.
    #[test]
    fn p1_determinism(seed in 0u64..64, cycles in 1u32..4) {
        let (mut a, mut pa, _) = seeded_run_ready(seed);
        let (mut b, mut pb, _) = seeded_run_ready(seed);
        a.run(&mut pa, cycles).unwrap();
        b.run(&mut pb, cycles).unwrap();
        prop_assert_eq!(a.log.events(), b.log.events());
    }

    /// P2 · harvest order-independence under storage shuffle.
    #[test]
    fn p2_harvest_order_independent(seed in 0u64..32) {
        let (mut s1, mut p1, _) = seeded_run_ready(seed);
        let (mut s2, mut p2, _) = seeded_run_ready(seed);
        // Shuffle p2 storage order.
        let n = p2.len();
        if n >= 2 {
            let perm: Vec<usize> = (0..n).rev().collect();
            p2.shuffle_storage(&perm);
        }
        s1.run(&mut p1, 2).unwrap();
        s2.run(&mut p2, 2).unwrap();
        prop_assert_eq!(s1.log.events(), s2.log.events());
    }

    /// P5 · scope soundness.
    #[test]
    fn p5_scope_soundness(_u in Just(())) {
        let (pop, _, _) = sensory_population();
        let source = pop.get(CellId(1)).unwrap();
        let self_sig = Signal::new(
            SignalType::Operational,
            "trigger",
            Scope::SelfCell,
            Payload::empty(),
            source.id,
            CausalStamp(0),
        );
        let t = resolve_targets(source, pop.cells(), &self_sig).unwrap();
        prop_assert_eq!(t, vec![source.id]);

        let neigh = Signal::new(
            SignalType::Operational,
            "trigger",
            Scope::Neighbors,
            Payload::empty(),
            source.id,
            CausalStamp(0),
        );
        let sys = systemwide_trigger(source.id, CausalStamp(0));
        let tn = resolve_targets(source, pop.cells(), &neigh).unwrap();
        let ts = resolve_targets(source, pop.cells(), &sys).unwrap();
        prop_assert!(tn.iter().all(|id| ts.contains(id)));
        prop_assert!(!tn.contains(&source.id));

        let cluster = Signal::new(
            SignalType::Operational,
            "trigger",
            Scope::Cluster,
            Payload::empty(),
            source.id,
            CausalStamp(0),
        );
        let err = resolve_targets(source, pop.cells(), &cluster).unwrap_err();
        let is_unavail = matches!(
            err,
            BiomimicryError::ScopeUnavailable {
                scope: Scope::Cluster,
                ..
            }
        );
        prop_assert!(is_unavail);
    }

    /// P6 · surface-intersection = brute-force scan.
    #[test]
    fn p6_surface_eq_scan(scope_i in 0usize..3) {
        let (pop, _, _) = sensory_population();
        let source = pop.get(CellId(1)).unwrap();
        let scope = [Scope::SelfCell, Scope::Neighbors, Scope::Systemwide][scope_i];
        let sig = Signal::new(
            SignalType::Operational,
            "trigger",
            scope,
            Payload::empty(),
            source.id,
            CausalStamp(0),
        );
        let a = resolve_targets(source, pop.cells(), &sig).unwrap();
        let b = resolve_targets_bruteforce(source, pop.cells(), &sig).unwrap();
        prop_assert_eq!(a, b);
    }
}

/// P3 · phase invariant across inners of one outer.
#[test]
fn p3_phase_invariant() {
    let (mut sched, mut pop, _) = seeded_run_ready(9);
    // Finish P1 of one outer manually: harvest + drain p1 via outer_cycle start
    // Then snapshot and run remaining inners — easier: run(1) and check that
    // Express on P2 doesn't change mid-inner (covered by A2). Here: snapshot
    // surfaces between consecutive inner_cycle calls after a clean harvest.
    sched.harvest_for_test(&mut pop);
    let snap = |p: &Population| {
        p.cells()
            .iter()
            .map(|c| c.expression.profile().receptor_surface.clone())
            .collect::<Vec<_>>()
    };
    let mut prev = snap(&pop);
    for _ in 0..sched.cadence.k {
        sched.inner_cycle(&mut pop).unwrap();
        let now = snap(&pop);
        assert_eq!(prev, now);
        prev = now;
    }
}

/// P4 · budget exhaustion caps transduction spends.
#[test]
fn p4_budget_exhaustion() {
    let (mut pop, _, _) = sensory_population();
    // Zero P2 capacity on cell 1.
    {
        let c = pop.get_mut(CellId(1)).unwrap();
        c.energy.phase2.capacity_milli = 2;
        c.energy.phase2.remaining_milli = 2;
    }
    OrganismEnergy::reset_p2_all(pop.cells_mut());
    assert_eq!(pop.get(CellId(1)).unwrap().energy.phase2.remaining_milli, 2);
    let mut spends = 0;
    {
        let c = pop.get_mut(CellId(1)).unwrap();
        while OrganismEnergy::try_spend(c, crate::signal::Phase::Phase2, 1) {
            spends += 1;
        }
    }
    assert_eq!(spends, 2);
    assert!(!OrganismEnergy::gate_transduce(pop.get(CellId(1)).unwrap()));
    assert!(OrganismEnergy::gate_express(pop.get(CellId(1)).unwrap()));
}

/// P7 · die purges in-flight ops; dead dispatch is a no-op.
#[test]
fn p7_die_drop_and_dead_dispatch() {
    let (mut sched, mut pop, _) = seeded_run_ready(1);
    sched.inject(ScheduledOp {
        cell: CellId(2),
        op: Operation::Emit(systemwide_trigger(CellId(2), CausalStamp(1))),
    });
    sched.inject(ScheduledOp {
        cell: CellId(2),
        op: Operation::Die,
    });
    // Die is Phase1 — run outer to execute it.
    sched.run(&mut pop, 1).unwrap();
    assert_eq!(
        pop.get(CellId(2)).unwrap().lifecycle(),
        LifecycleState::Dead
    );
    // Further receives on dead cell produce no pending.
    let before = pop.get(CellId(2)).unwrap().pending.len();
    let _ = pop
        .get_mut(CellId(2))
        .unwrap()
        .receive(&systemwide_trigger(CellId(2), CausalStamp(9)));
    assert_eq!(pop.get(CellId(2)).unwrap().pending.len(), before);
}

#[test]
fn cadence_rejects_zero() {
    assert!(matches!(
        Cadence::try_new(0),
        Err(BiomimicryError::CadenceMisconfigured { k: 0 })
    ));
}

#[test]
fn seed_sweep_all_replayable() {
    for seed in 0u64..8 {
        let (mut a, mut pa, _) = seeded_run_ready(seed);
        let (mut b, mut pb, _) = seeded_run_ready(seed);
        let _ = a.run_until_drained(&mut pa, 10).unwrap();
        let _ = b.run_until_drained(&mut pb, 10).unwrap();
        assert_eq!(a.log.events(), b.log.events(), "seed {seed}");
    }
}
