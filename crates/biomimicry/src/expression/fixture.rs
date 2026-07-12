//! M4 integration fixture: rule network + cascade brains on a Population.

use std::sync::Arc;

use crate::cell::Operation;
use crate::cell::{Cell, CellId, LifecycleState};
use crate::expression::{NetworkRegulator, RegulatoryRule, RuleCondition, RuleNetwork};
use crate::genesis::{GeneId, cascade_dna, compile};
use crate::medium::ScheduledOp;
use crate::metabolism::{Cadence, Population, Scheduler};
use crate::signal::{CausalStamp, Payload, Scope, Signal, SignalKind, SignalType};
use crate::transduction::{Cascade, CascadeTransducer, TransductionFn};

/// Compiled M4 genome handles.
#[derive(Debug, Clone)]
pub struct M4Handles {
    /// Shared genome.
    pub genome: Arc<crate::genesis::Genome>,
    /// Gene with Receptor+/Expression+/Transduction+.
    pub cascade_path: GeneId,
    /// Second gene activated by the rule network.
    pub effector: GeneId,
}

/// Compile cascade DNA and resolve gene ids.
#[must_use]
pub fn m4_handles() -> M4Handles {
    let dna = cascade_dna();
    let genome = compile(&dna).expect("compile cascade_dna");
    let cascade_path = genome
        .iter()
        .find(|g| {
            g.cistron.kind.as_str() == "cascade_path"
                && g.cistron.endpoints.iter().any(|ep| {
                    ep.primitive == crate::genesis::Primitive::Receptor
                        && ep.polarity == crate::genesis::EndpointPolarity::Positive
                })
        })
        .map(|g| g.id)
        .expect("cascade_path");
    let effector = genome
        .iter()
        .find(|g| {
            g.cistron.kind.as_str() == "effector"
                && g.cistron.endpoints.iter().any(|ep| {
                    ep.primitive == crate::genesis::Primitive::Receptor
                        && ep.polarity == crate::genesis::EndpointPolarity::Positive
                })
        })
        .map(|g| g.id)
        .expect("effector");
    M4Handles {
        genome,
        cascade_path,
        effector,
    }
}

/// Rule network: on trigger kind → activate effector.
#[must_use]
pub fn m4_network(effector: GeneId) -> RuleNetwork {
    RuleNetwork::new().with_rule(
        RegulatoryRule::new("activate_effector")
            .with_condition(RuleCondition::SignalKind(SignalKind::new("trigger")))
            .with_express([effector]),
    )
}

/// Cascade transducer: identity-echo `cascade_out` to SelfCell for `cascade_path`.
#[must_use]
pub fn m4_transducer(cascade_path: GeneId) -> CascadeTransducer {
    let cascade =
        Cascade::new().with_step(TransductionFn::identity_echo("echo_out", "cascade_out"));
    CascadeTransducer::new().with_cascade(cascade_path, cascade)
}

/// Two Active cells with `cascade_path` expressed; Network+Cascade brains installed.
#[must_use]
pub fn m4_seeded_run(seed: u64) -> (Scheduler, Population, M4Handles) {
    let handles = m4_handles();
    let mut cells = Vec::new();
    for id in [1u64, 2] {
        let mut cell = Cell::new(CellId(id), Arc::clone(&handles.genome));
        cell.try_transition(LifecycleState::Differentiating)
            .unwrap();
        cell.try_transition(LifecycleState::Active).unwrap();
        cell.activate(handles.cascade_path);
        cells.push(cell);
    }
    let pop = Population::from_cells(cells);
    let mut sched = Scheduler::new(seed, Cadence::new(2));
    sched.with_regulator(NetworkRegulator::new(m4_network(handles.effector)));
    sched.with_transducer(m4_transducer(handles.cascade_path));
    let sig = Signal::new(
        SignalType::Operational,
        "trigger",
        Scope::Systemwide,
        Payload::empty(),
        CellId(1),
        CausalStamp(0),
    );
    sched.inject(ScheduledOp {
        cell: CellId(1),
        op: Operation::Receive(sig),
    });
    (sched, pop, handles)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_tag_index(tags: &[&str], want: &str) -> Option<usize> {
        tags.iter().position(|t| *t == want)
    }

    #[test]
    fn a1_causal_chain_order() {
        let (mut sched, mut pop, handles) = m4_seeded_run(42);
        sched.run(&mut pop, 4).unwrap();

        let tags: Vec<&str> = sched.log.events().iter().map(|e| e.tag.as_str()).collect();
        let receive = first_tag_index(&tags, "receive").expect("receive");
        let express = first_tag_index(&tags, "express").expect("express");
        let transduce = first_tag_index(&tags, "transduce").expect("transduce");
        let emit = first_tag_index(&tags, "emit").expect("emit");
        let deliver = first_tag_index(&tags, "deliver").expect("deliver");

        assert!(receive < express, "receive before express: {tags:?}");
        assert!(express < transduce, "express before transduce: {tags:?}");
        assert!(transduce < emit, "transduce before emit: {tags:?}");
        assert!(emit < deliver, "emit before deliver: {tags:?}");

        // Rule network activated effector on at least one cell.
        assert!(
            pop.cells()
                .iter()
                .any(|c| c.expression.is_active(handles.effector)),
            "effector should be active after Phase 1 rules"
        );
    }

    #[test]
    fn a2_phase_freeze_surfaces_stable_across_inners() {
        let (mut sched, mut pop, handles) = m4_seeded_run(3);
        // Advance far enough that expression has settled once.
        sched.run(&mut pop, 2).unwrap();
        let before: Vec<_> = pop
            .cells()
            .iter()
            .map(|c| c.expression.profile().receptor_surface.clone())
            .collect();
        // Mis-route a regulatory Express onto Phase 2 mid-inner — must not apply.
        sched.phase2.push(ScheduledOp {
            cell: CellId(1),
            op: Operation::Express {
                gene: handles.effector,
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
    }

    #[test]
    fn a3_replay_identical_logs_with_real_brains() {
        let (mut s1, mut p1, _) = m4_seeded_run(7);
        let (mut s2, mut p2, _) = m4_seeded_run(7);
        s1.run(&mut p1, 3).unwrap();
        s2.run(&mut p2, 3).unwrap();
        assert_eq!(s1.log.events(), s2.log.events());
    }
}
