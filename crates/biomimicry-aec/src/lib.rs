//! AEC reference application (Part VIII) — wall-move scenario.
//!
//! Domain SignalKinds, DNA, and scenario driver live here. The core engine
//! stays free of `wall` / `beam` types.

pub mod dna;
pub mod fixture;
pub mod kinds;
pub mod options;
pub mod scenario;

pub use dna::aec_dna;
pub use fixture::{AecHandles, aec_handles, overspan_signal, wall_move_ready, wall_move_signal};
pub use kinds::*;
pub use options::build_aec_beam_options;
pub use scenario::{
    WallMoveReport, cascade_evidence, run_counterfactual_fork, run_reflex, run_wall_move,
};

/// Placeholder / public scenario name.
#[must_use]
pub fn scenario_name() -> &'static str {
    "wall-move"
}

#[cfg(test)]
mod tests {
    use super::*;
    use biomimicry::attractor::SettleStatus;
    use biomimicry::causality::Equilibrium;
    use biomimicry::error::BiomimicryError;
    use biomimicry::membrane::ResponseMode;

    #[test]
    fn scaffold_crate_loads() {
        assert_eq!(scenario_name(), "wall-move");
    }

    #[test]
    fn m9_a1_reflex_wall_move() {
        let (_org, report) = run_reflex(42, 32).expect("reflex");
        assert_eq!(report.settle, SettleStatus::Converged);
        assert!(report.cascade_fired, "tags={:?}", report.reflex_tags);
        assert!(cascade_evidence(&report.reflex_tags));
    }

    #[test]
    fn m9_a2_overspan_escalate() {
        let mut org = wall_move_ready(7);
        let mode = org.ingress(overspan_signal()).unwrap();
        assert_eq!(mode, ResponseMode::Escalation);
        let inbox = org.drain_escalations();
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].options.len(), 2);
        assert_eq!(inbox[0].options[0].label, "upsize_W14x53");
        assert_eq!(inbox[0].options[0].cost_milli, 4_200_000);
        assert!(!org.commit_gate().open);
    }

    #[test]
    fn m9_a3_commit_gate() {
        let report = run_wall_move(11, 32, 0).expect("wall-move");
        assert_eq!(report.equilibrium, Equilibrium::Committed);
        assert!(report.snapshot.is_some());
        assert_eq!(report.chosen_option.as_deref(), Some("upsize_W14x53"));

        let mut org = wall_move_ready(12);
        let err = org.checkpoint("blocked", false).unwrap_err();
        assert!(matches!(err, BiomimicryError::CommitmentGateClosed));
    }

    #[test]
    fn m9_a4_domain_isolation() {
        // Kinds and builders live in this crate; core has no wall/beam types.
        assert_eq!(WALL_MOVE, "aec.wall.move");
        assert_eq!(BEAM_OVERSPAN, "aec.beam.overspan");
        let _ = aec_dna();
        let _ = build_aec_beam_options;
    }

    #[test]
    fn m9_a5_counterfactual_fork() {
        let diverged = run_counterfactual_fork(13, 32).expect("fork");
        assert!(diverged);
    }

    #[test]
    fn p1_same_seed_identical_reflex_tags() {
        let (_, a) = run_reflex(99, 32).unwrap();
        let (_, b) = run_reflex(99, 32).unwrap();
        assert_eq!(a.reflex_tags, b.reflex_tags);
        assert_eq!(a.settle, b.settle);
    }
}
