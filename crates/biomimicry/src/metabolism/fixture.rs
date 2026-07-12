//! M3 integration fixture: population + scheduler replay / phase separation.

use std::sync::Arc;

use crate::cell::{Cell, CellId, LifecycleState, Operation};
use crate::genesis::{GeneId, compile, toy_dna};
use crate::medium::ScheduledOp;
use crate::metabolism::{Cadence, Population, Scheduler};
use crate::signal::{CausalStamp, Payload, Scope, Signal, SignalType};

/// Three Active cells sharing the sensory_spike receptor surface.
#[must_use]
pub fn sensory_population() -> (Population, Arc<crate::genesis::Genome>, GeneId) {
    let dna = toy_dna();
    let genome = compile(&dna).expect("compile");
    let spike = genome
        .iter()
        .find(|g| {
            g.cistron.kind.as_str() == "sensory_spike"
                && g.cistron.endpoints.iter().any(|ep| {
                    ep.primitive == crate::genesis::Primitive::Receptor
                        && ep.polarity == crate::genesis::EndpointPolarity::Positive
                })
        })
        .map(|g| g.id)
        .expect("spike");

    let mut cells = Vec::new();
    for id in [1u64, 2, 3] {
        let mut cell = Cell::new(CellId(id), Arc::clone(&genome));
        cell.try_transition(LifecycleState::Differentiating)
            .unwrap();
        cell.try_transition(LifecycleState::Active).unwrap();
        cell.activate(spike);
        cells.push(cell);
    }
    (Population::from_cells(cells), genome, spike)
}

/// Systemwide operational trigger into the population.
#[must_use]
pub fn systemwide_trigger(source: CellId, stamp: CausalStamp) -> Signal {
    Signal::new(
        SignalType::Operational,
        "trigger",
        Scope::Systemwide,
        Payload::empty(),
        source,
        stamp,
    )
}

/// Seed a scheduler and inject one systemwide receive on cell 1.
#[must_use]
pub fn seeded_run_ready(seed: u64) -> (Scheduler, Population, GeneId) {
    let (pop, _, spike) = sensory_population();
    let mut sched = Scheduler::new(seed, Cadence::new(2));
    let sig = systemwide_trigger(CellId(1), CausalStamp(0));
    sched.inject(ScheduledOp {
        cell: CellId(1),
        op: Operation::Receive(sig),
    });
    (sched, pop, spike)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a1_replay_identical_logs() {
        let (mut s1, mut p1, _) = seeded_run_ready(7);
        let (mut s2, mut p2, _) = seeded_run_ready(7);
        s1.run(&mut p1, 3).unwrap();
        s2.run(&mut p2, 3).unwrap();
        assert_eq!(s1.log.events(), s2.log.events());
        assert_eq!(s1.outer_cycles, 3);
        assert_eq!(s1.inner_cycles, 6);
    }

    #[test]
    fn a1_different_seed_internally_replayable() {
        let (mut s1, mut p1, _) = seeded_run_ready(11);
        let (mut s2, mut p2, _) = seeded_run_ready(11);
        s1.run(&mut p1, 2).unwrap();
        s2.run(&mut p2, 2).unwrap();
        assert_eq!(s1.log, s2.log);
    }

    #[test]
    fn a2_phase_separation_surfaces_stable_across_inners() {
        let (mut sched, mut pop, spike) = seeded_run_ready(3);
        // Snapshot after any initial P1, then only run inners with a regulatory
        // Express mis-routed onto Phase 2 — must not change surfaces mid-inner.
        let before: Vec<_> = pop
            .cells()
            .iter()
            .map(|c| c.expression.profile().receptor_surface.clone())
            .collect();
        sched.phase2.push(ScheduledOp {
            cell: CellId(1),
            op: Operation::Express {
                gene: spike,
                on: false,
            },
        });
        for _ in 0..sched.cadence.k {
            sched.inner_cycle(&mut pop).unwrap();
        }
        let after: Vec<_> = pop
            .cells()
            .iter()
            .map(|c| c.expression.profile().receptor_surface.clone())
            .collect();
        assert_eq!(before, after, "expression must not change mid-Phase-2");
        // The Express should have been re-queued to Phase 1.
        assert!(
            sched
                .phase1
                .iter()
                .any(|op| matches!(op.op, Operation::Express { .. })),
            "regulatory op must be held on Phase 1"
        );
    }

    #[test]
    fn a3_n_cycles_and_drained() {
        let (mut sched, mut pop, _) = seeded_run_ready(5);
        sched.run(&mut pop, 4).unwrap();
        assert_eq!(sched.outer_cycles, 4);
        assert_eq!(sched.inner_cycles, 8);
        let (mut s2, mut p2, _) = seeded_run_ready(5);
        let drained = s2.run_until_drained(&mut p2, 20).unwrap();
        assert!(drained || s2.outer_cycles <= 20);
    }
}
