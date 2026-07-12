//! Wall-move scenario driver (Part VIII walkthrough).

use biomimicry::attractor::SettleStatus;
use biomimicry::causality::Equilibrium;
use biomimicry::error::{BiomimicryError, Result};
use biomimicry::membrane::ResponseMode;
use biomimicry::organism::Organism;
use biomimicry::substrate::{MemoryStore, SnapshotMeta, Store};

use crate::fixture::{overspan_signal, wall_move_ready, wall_move_signal};
use crate::kinds::RECOMPUTE_KINDS;

/// Report from a full wall-move run.
#[derive(Debug, Clone)]
pub struct WallMoveReport {
    /// Scenario name.
    pub scenario: &'static str,
    /// Reflex settle status.
    pub settle: SettleStatus,
    /// Causal tags observed after reflex.
    pub reflex_tags: Vec<String>,
    /// Whether recompute emissions appeared (emit/transduce present).
    pub cascade_fired: bool,
    /// Chosen escalation option label (if any).
    pub chosen_option: Option<String>,
    /// Equilibrium after commit attempt.
    pub equilibrium: Equilibrium,
    /// Checkpoint metadata when committed.
    pub snapshot: Option<SnapshotMeta>,
}

/// Run reflex wall-move only (A1).
///
/// # Errors
///
/// Returns an error if ingress or settle fails.
pub fn run_reflex(seed: u64, max_ticks: u64) -> Result<(Organism<MemoryStore>, WallMoveReport)> {
    let mut org = wall_move_ready(seed);
    let mode = org.ingress(wall_move_signal())?;
    assert_eq!(mode, ResponseMode::Reflex);
    let settle = org.settle(max_ticks)?;
    let reflex_tags: Vec<String> = org
        .scheduler
        .log
        .events()
        .iter()
        .map(|e| e.tag.clone())
        .collect();
    let cascade_fired = reflex_tags.iter().any(|t| t == "emit" || t == "transduce");
    let report = WallMoveReport {
        scenario: crate::scenario_name(),
        settle,
        reflex_tags,
        cascade_fired,
        chosen_option: None,
        equilibrium: Equilibrium::Working,
        snapshot: None,
    };
    Ok((org, report))
}

/// Full Part VIII path: reflex → overspan escalate → choose → checkpoint.
///
/// # Errors
///
/// Returns an error if any scenario step fails.
pub fn run_wall_move(seed: u64, max_ticks: u64, choose_index: usize) -> Result<WallMoveReport> {
    let (mut org, mut report) = run_reflex(seed, max_ticks)?;

    // Closed gate must reject.
    let closed = org.checkpoint("premature", false);
    assert!(matches!(closed, Err(BiomimicryError::CommitmentGateClosed)));

    let mode = org.ingress(overspan_signal())?;
    assert_eq!(mode, ResponseMode::Escalation);
    assert!(!org.commit_gate().open);

    let inbox = org.drain_escalations();
    assert_eq!(inbox.len(), 1);
    assert_eq!(inbox[0].options.len(), 2);
    let chosen = inbox[0]
        .options
        .get(choose_index)
        .ok_or_else(|| BiomimicryError::Organism("bad choose_index".into()))?
        .label
        .clone();
    report.chosen_option = Some(chosen);

    org.open_commit_gate();
    let meta = org.checkpoint("wall-move-committed", false)?;
    report.snapshot = Some(meta);
    report.equilibrium = Equilibrium::Committed;
    Ok(report)
}

/// Whether the reflex log proves the cascade recomputes story (emit/transduce).
#[must_use]
pub fn cascade_evidence(tags: &[String]) -> bool {
    let _ = RECOMPUTE_KINDS;
    tags.iter().any(|t| t == "emit" || t == "transduce")
}

/// Counterfactual stretch: fork after reflex, second overspan path, diff DAGs.
///
/// # Errors
///
/// Returns an error on store/organism failure.
pub fn run_counterfactual_fork(seed: u64, max_ticks: u64) -> Result<bool> {
    let (mut org, _) = run_reflex(seed, max_ticks)?;
    org.open_commit_gate();
    let meta = org.checkpoint("base", false)?;
    let base = org.load_causal_dag()?;

    org.fork_branch(meta.id, "what-if")?;
    org.scheduler.log.clear();
    org.store
        .replace_causal_dag(biomimicry::causality::CausalDag::new())?;
    let _ = org.ingress(overspan_signal())?;
    // Overspan is escalation-only; also inject a second wall move for log divergence.
    let _ = org.ingress(wall_move_signal())?;
    let _ = org.settle(max_ticks)?;
    org.flush_causal()?;
    let branched = org.load_causal_dag()?;
    let diff = Organism::<MemoryStore>::diff_causal(&base, &branched);
    Ok(diff.only_in_a > 0 || diff.only_in_b > 0 || base != branched)
}
