//! M2 integration fixture: sensory_spike cell receive + lifecycle guards.

use std::sync::Arc;

use crate::cell::{Cell, CellId, LifecycleState};
use crate::genesis::{GeneId, compile, toy_dna};
use crate::signal::{CausalStamp, Payload, Scope, Signal, SignalType};

/// Compile the M1 toy DNA and return genome + sensory_spike gene id.
/// Sensory genome spike id — prefer the traversed gene with `Receptor+`.
#[must_use]
pub fn sensory_genome() -> (Arc<crate::genesis::Genome>, GeneId) {
    let dna = toy_dna();
    let genome = compile(&dna).expect("compile toy dna");
    let spike = genome
        .iter()
        .find(|gene| {
            gene.hyperedge.kind.as_str() == "sensory_spike"
                && gene.hyperedge.endpoints.iter().any(|ep| {
                    ep.primitive == crate::genesis::Primitive::Receptor
                        && ep.polarity == crate::genesis::EndpointPolarity::Positive
                })
        })
        .map(|g| g.id)
        .expect("sensory_spike Receptor+");
    (genome, spike)
}

/// Build an Active cell with sensory_spike expressed.
#[must_use]
pub fn active_sensory_cell() -> (Cell, GeneId) {
    let (genome, spike) = sensory_genome();
    let mut cell = Cell::new(CellId(1), genome);
    cell.try_transition(LifecycleState::Differentiating)
        .expect("undiff→diff");
    cell.try_transition(LifecycleState::Active)
        .expect("diff→active");
    cell.activate(spike);
    (cell, spike)
}

/// Signal whose kind matches the sensory_spike Receptor+ role `"trigger"`.
#[must_use]
pub fn trigger_signal(source: CellId, stamp: CausalStamp) -> Signal {
    Signal::new(
        SignalType::Regulatory,
        "trigger",
        Scope::Systemwide,
        Payload::empty(),
        source,
        stamp,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::{BehavioralMode, Operation, lifecycle_dot, profile};
    use crate::error::BiomimicryError;
    use crate::signal::Phase;

    #[test]
    fn a1_match_and_enqueue() {
        let (mut cell, spike) = active_sensory_cell();
        let sig = trigger_signal(CellId(99), CausalStamp(0));
        let reaction = cell.receive(&sig);

        assert!(reaction.matched_genes.contains(&spike));
        assert!(reaction.dropped_reason.is_none());

        assert!(
            reaction
                .enqueued
                .iter()
                .any(|op| matches!(op, Operation::Receive(_)))
        );
        assert!(reaction.enqueued.iter().any(|op| {
            matches!(
                op,
                Operation::Express {
                    gene,
                    on: true
                } if *gene == spike
            )
        }));
        assert!(
            reaction
                .enqueued
                .iter()
                .any(|op| matches!(op, Operation::Emit(_)))
        );

        let express = reaction
            .enqueued
            .iter()
            .find(|op| matches!(op, Operation::Express { .. }))
            .unwrap();
        assert_eq!(express.phase(), Phase::Phase1);
        let emit = reaction
            .enqueued
            .iter()
            .find(|op| matches!(op, Operation::Emit(_)))
            .unwrap();
        assert_eq!(emit.phase(), Phase::Phase2);

        assert_eq!(cell.pending.len(), reaction.enqueued.len());
    }

    #[test]
    fn a2_illegal_transition_rejected() {
        let (genome, _) = sensory_genome();
        let mut cell = Cell::new(CellId(1), genome);
        assert_eq!(cell.lifecycle(), LifecycleState::Undifferentiated);
        let err = cell
            .try_transition(LifecycleState::Active)
            .expect_err("skip differentiating");
        assert!(matches!(
            err,
            BiomimicryError::IllegalLifecycleTransition {
                from: LifecycleState::Undifferentiated,
                to: LifecycleState::Active,
            }
        ));
        assert_eq!(cell.lifecycle(), LifecycleState::Undifferentiated);
    }

    #[test]
    fn dead_cell_drops_dispatch() {
        let (mut cell, _) = active_sensory_cell();
        cell.try_transition(LifecycleState::Dead).unwrap();
        let reaction = cell.receive(&trigger_signal(CellId(2), CausalStamp(1)));
        assert_eq!(reaction.dropped_reason, Some("dead-cell-dispatch"));
        assert!(cell.pending.is_empty());
    }

    #[test]
    fn inspector_dumps_nonempty() {
        let (cell, _) = active_sensory_cell();
        let p = profile(&cell);
        assert!(p.contains("lifecycle"));
        assert!(p.contains("Active"));
        let _ = BehavioralMode::Idle;
        let dot = lifecycle_dot();
        assert!(dot.contains("digraph"));
        assert!(dot.contains("Dead"));
    }
}
