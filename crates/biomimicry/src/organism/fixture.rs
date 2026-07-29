//! M5/M6 integration fixtures: settle + ganglion/sensorium.

use std::sync::Arc;

use crate::cell::CellId;
use crate::expression::{NetworkRegulator, m4_handles, m4_network, m4_transducer};
use crate::ganglion::GanglionHandle;
use crate::metabolism::{Cadence, SpaceConfig};
use crate::organism::OrganismBuilder;
use crate::signal::{CausalStamp, Payload, Scope, Signal, SignalType};
use crate::substrate::MemoryStore;

/// Build a small cascade organism ready to perturb (damped pop loop at size).
#[must_use]
pub fn settle_ready(seed: u64) -> crate::organism::Organism<MemoryStore> {
    let handles = m4_handles();
    OrganismBuilder::new()
        .genome(Arc::clone(&handles.genome))
        .seed(seed)
        .cadence(Cadence::new(2))
        .population_size(2)
        .target_population(2)
        .seed_gene(handles.cascade_path)
        .regulator(NetworkRegulator::new(m4_network(handles.downstream)))
        .transducer(m4_transducer(handles.cascade_path))
        .build()
        .expect("build organism")
}

/// Undamped population loop (oscillation / limit-cycle demo).
#[must_use]
pub fn undamped_ready(seed: u64) -> crate::organism::Organism<MemoryStore> {
    let handles = m4_handles();
    OrganismBuilder::new()
        .genome(Arc::clone(&handles.genome))
        .seed(seed)
        .cadence(Cadence::new(2))
        .population_size(2)
        .target_population(2)
        .undamped_population()
        .seed_gene(handles.cascade_path)
        .build()
        .expect("build undamped organism")
}

/// Systemwide trigger for cascade_path receptors.
#[must_use]
pub fn trigger_signal() -> Signal {
    Signal::new(
        SignalType::Operational,
        "trigger",
        Scope::Systemwide,
        Payload::empty(),
        CellId(1),
        CausalStamp(0),
    )
}

/// Small cascade organism with a 2-cell ganglion (M6).
#[must_use]
pub fn ganglion_ready(seed: u64) -> crate::organism::Organism<MemoryStore> {
    let handles = m4_handles();
    let mut org = OrganismBuilder::new()
        .genome(Arc::clone(&handles.genome))
        .seed(seed)
        .cadence(Cadence::new(2))
        .population_size(2)
        .target_population(2)
        .seed_gene(handles.cascade_path)
        .without_pop_loop()
        .regulator(NetworkRegulator::new(m4_network(handles.downstream)))
        .transducer(m4_transducer(handles.cascade_path))
        .build()
        .expect("build");
    org.attach_ganglion(
        GanglionHandle(1),
        "circuit",
        4,
        SpaceConfig { k: 2 },
        [CellId(1), CellId(2)],
    );
    org
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attractor::{DivergenceKind, SettleStatus, detect_divergence};
    use crate::cell::Operation;
    use crate::ganglion::{Ganglion, GanglionHealth};
    use crate::medium::ScheduledOp;
    use crate::organism::Organism;
    use crate::sensorium::validate_integrity;
    use crate::substrate::Store;

    #[test]
    fn a1_first_settle_converged() {
        let mut org = settle_ready(42);
        org.perturb(trigger_signal()).unwrap();
        let status = org.settle(32).unwrap();
        assert_eq!(status, SettleStatus::Converged);
        assert!(org.trajectory().len() >= org.settle_window);
    }

    #[test]
    fn a2_undamped_limit_cycle() {
        let mut org = undamped_ready(7);
        let status = org.settle(24).unwrap();
        assert_ne!(status, SettleStatus::Converged);
        assert_eq!(
            detect_divergence(org.trajectory()),
            Some(DivergenceKind::LimitCycle),
            "traj={:?}",
            org.trajectory()
        );
    }

    #[test]
    fn a3_replay_identical_trajectory() {
        let mut a = settle_ready(11);
        let mut b = settle_ready(11);
        a.perturb(trigger_signal()).unwrap();
        b.perturb(trigger_signal()).unwrap();
        let sa = a.settle(32).unwrap();
        let sb = b.settle(32).unwrap();
        assert_eq!(sa, sb);
        assert_eq!(a.trajectory(), b.trajectory());
        assert_eq!(a.scheduler.log.events(), b.scheduler.log.events());
    }

    #[test]
    fn m6_a1_inspect_ganglion() {
        let mut org = ganglion_ready(1);
        let view = org.inspect_ganglion(GanglionHandle(1)).expect("ganglion");
        assert_eq!(view.name, "circuit");
        assert_eq!(view.members.len(), 2);
        assert_eq!(view.health, GanglionHealth::Healthy);

        org.scheduler.inject(ScheduledOp {
            cell: CellId(2),
            op: Operation::Die,
        });
        org.scheduler.delivery_ganglia = org.ganglia.clone();
        org.scheduler.outer_cycle(&mut org.population).unwrap();
        org.refresh_ganglia_health();
        let view = org.inspect_ganglion(GanglionHandle(1)).unwrap();
        assert_eq!(view.health, GanglionHealth::Degraded);
    }

    #[test]
    fn m6_a2_passive_readout() {
        let mut org = ganglion_ready(2);
        let sig = Signal::new(
            SignalType::Operational,
            "report",
            Scope::SelfCell,
            Payload::empty().with_observation("spike"),
            CellId(1),
            CausalStamp(0),
        );
        org.scheduler.inject(ScheduledOp {
            cell: CellId(1),
            op: Operation::Emit(sig),
        });
        org.scheduler.delivery_ganglia = org.ganglia.clone();
        org.scheduler.outer_cycle(&mut org.population).unwrap();
        for s in org.scheduler.take_observations() {
            org.collector.observe(s);
        }
        let samples = org.readout();
        assert_eq!(samples.len(), 1);
        assert!(samples[0].payload.is_observation());
    }

    #[test]
    fn m6_a3_immune_flags() {
        let org = ganglion_ready(3);
        assert!(org.immune_flags().is_empty());

        let mut g = Ganglion::new(GanglionHandle(99), "ghost", 2);
        g.try_add(CellId(999));
        let flags = validate_integrity(org.population.cells(), std::slice::from_ref(&g));
        assert!(flags.iter().any(|f| f.code == "dangling_member"));
    }

    #[test]
    fn m6_a4_cluster_delivery() {
        let mut org = ganglion_ready(4);
        let sig = Signal::new(
            SignalType::Operational,
            "ping",
            Scope::Cluster,
            Payload::empty(),
            CellId(1),
            CausalStamp(0),
        );
        org.scheduler.inject(ScheduledOp {
            cell: CellId(1),
            op: Operation::Emit(sig),
        });
        org.scheduler.delivery_ganglia = org.ganglia.clone();
        org.scheduler.outer_cycle(&mut org.population).unwrap();
        let log_has_deliver = org
            .scheduler
            .log
            .events()
            .iter()
            .any(|e| e.tag == "deliver" && e.cell == CellId(2));
        assert!(log_has_deliver, "cluster should deliver to co-member");
    }

    #[test]
    fn m6_a5_divide_fast() {
        let mut org = ganglion_ready(5);
        let before = org.population.len();
        org.scheduler.inject(ScheduledOp {
            cell: CellId(1),
            op: Operation::DivideFast,
        });
        org.scheduler.delivery_ganglia = org.ganglia.clone();
        org.scheduler.outer_cycle(&mut org.population).unwrap();
        for (parent, daughter) in org.scheduler.take_lineage() {
            for g in &mut org.ganglia {
                if g.contains(parent) {
                    assert!(g.try_add(daughter));
                }
            }
        }
        assert_eq!(org.population.len(), before + 1);
        let g = org.inspect_ganglion(GanglionHandle(1)).unwrap();
        assert!(g.members.len() >= 3);
    }

    #[test]
    fn m7_a1_persist_and_replay() {
        let mut org = settle_ready(7);
        org.perturb(trigger_signal()).unwrap();
        assert_eq!(org.settle(32).unwrap(), SettleStatus::Converged);
        org.open_commit_gate();
        let meta = org.checkpoint("a1", false).unwrap();
        let before = org.load_causal_dag().unwrap();
        assert!(!before.is_empty());

        // Mutate live store causal, then restore.
        org.store
            .replace_causal_dag(crate::causality::CausalDag::new())
            .unwrap();
        assert!(org.load_causal_dag().unwrap().is_empty());
        org.restore_checkpoint(meta.id).unwrap();
        let after = org.load_causal_dag().unwrap();
        assert_eq!(before, after);
        assert!(!org.scheduler.log.is_empty());
    }

    #[test]
    fn m7_a2_branch_counterfactual() {
        let mut org = settle_ready(11);
        org.perturb(trigger_signal()).unwrap();
        assert_eq!(org.settle(32).unwrap(), SettleStatus::Converged);
        org.open_commit_gate();
        let meta = org.checkpoint("base", false).unwrap();
        let base_dag = org.load_causal_dag().unwrap();

        org.fork_branch(meta.id, "counterfactual").unwrap();
        // Different seed path: clear log, re-perturb with a second settle after more ticks.
        org.scheduler.log.clear();
        org.store
            .replace_causal_dag(crate::causality::CausalDag::new())
            .unwrap();
        org.perturb(trigger_signal()).unwrap();
        let _ = org.settle(16).unwrap();
        org.flush_causal().unwrap();
        let branched = org.load_causal_dag().unwrap();

        let diff = Organism::<MemoryStore>::diff_causal(&base_dag, &branched);
        assert!(
            diff.only_in_a > 0 || diff.only_in_b > 0 || base_dag != branched,
            "expected DAG divergence after counterfactual path: {diff:?}"
        );
    }

    #[test]
    fn m7_p3_gate_closed_blocks_checkpoint() {
        let mut org = settle_ready(3);
        org.close_commit_gate();
        let err = org.checkpoint("blocked", false).expect_err("gate");
        assert!(matches!(
            err,
            crate::error::BiomimicryError::CommitmentGateClosed
        ));
        let _ = org.checkpoint("forced", true).unwrap();
    }
}
