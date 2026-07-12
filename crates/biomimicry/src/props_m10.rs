//! M10 gardenability props — organism-level determinism, replay, convergence.

#[cfg(test)]
mod tests {
    use crate::attractor::SettleStatus;
    use crate::membrane::{echo_ready, echo_request};
    use crate::organism::{settle_ready, trigger_signal};
    use crate::substrate::Store;

    #[test]
    fn p_det_same_seed_identical_trajectory_and_tags() {
        let run = |seed: u64| {
            let mut org = settle_ready(seed);
            org.perturb(trigger_signal()).unwrap();
            let status = org.settle(32).unwrap();
            let traj = org.trajectory().to_vec();
            let tags: Vec<String> = org
                .scheduler
                .log
                .events()
                .iter()
                .map(|e| e.tag.clone())
                .collect();
            (status, traj, tags)
        };
        let a = run(42);
        let b = run(42);
        assert_eq!(a, b);
    }

    #[test]
    fn p_replay_checkpoint_restore_dag() {
        let mut org = settle_ready(7);
        org.perturb(trigger_signal()).unwrap();
        assert_eq!(org.settle(32).unwrap(), SettleStatus::Converged);
        org.open_commit_gate();
        let meta = org.checkpoint("garden", false).unwrap();
        let before = org.load_causal_dag().unwrap();
        org.store
            .replace_causal_dag(crate::causality::CausalDag::new())
            .unwrap();
        assert!(org.load_causal_dag().unwrap().is_empty());
        org.restore_checkpoint(meta.id).unwrap();
        assert_eq!(org.load_causal_dag().unwrap(), before);
    }

    #[test]
    fn p_converge_settle_ready_seeds() {
        for seed in [1u64, 2, 3, 5, 8, 13] {
            let mut org = settle_ready(seed);
            org.perturb(trigger_signal()).unwrap();
            let status = org.settle(48).unwrap();
            assert_eq!(
                status,
                SettleStatus::Converged,
                "seed {seed} did not converge"
            );
        }
    }

    #[test]
    fn p_det_echo_ingress_deterministic() {
        let run = |seed: u64| {
            let mut org = echo_ready(seed);
            let _ = org.ingress(echo_request(100)).unwrap();
            let status = org.settle(32).unwrap();
            let tags: Vec<String> = org
                .scheduler
                .log
                .events()
                .iter()
                .map(|e| e.tag.clone())
                .collect();
            (status, tags)
        };
        assert_eq!(run(11), run(11));
    }
}
