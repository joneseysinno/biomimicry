//! The organism aggregate — owns population, ganglia, medium, scheduler, clock.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::attractor::DEFAULT_CONVERGENCE_WINDOW;
use crate::causality::{CausalClock, CommitmentGate};
use crate::cell::CellId;
use crate::ganglion::{Ganglion, GanglionHandle, GanglionView, view as ganglion_view};
use crate::genesis::{GeneId, Genome};
use crate::homeostasis::PopulationSizeLoop;
use crate::medium::Delivery;
use crate::membrane::{
    BoundaryCellTemplate, EscalationOption, EscalationPacket, MembranePolicy, build_echo_options,
};
use crate::metabolism::{Population, Scheduler, SpaceConfig};
use crate::sensorium::{ImmuneFlag, ReadoutCollector, SignalSample, validate_integrity};
use crate::substrate::Store;

/// The thing you instantiate and **perturb**.
///
/// There is no `run()` and no orchestrator cell inside it.
#[derive(Debug)]
pub struct Organism<S: Store> {
    /// Compiled genome (shared read-only catalog).
    pub genome: Arc<Genome>,
    /// Living cell population (stable `CellId` order).
    pub population: Population,
    /// Named ganglia (empty until M6).
    pub ganglia: Vec<Ganglion>,
    /// Signaling medium (organism-level handle; scheduler owns the hot path).
    pub medium: Delivery,
    /// Two-phase scheduler.
    pub scheduler: Scheduler,
    /// Causal logical clock.
    pub clock: CausalClock,
    /// Persistence backend.
    pub store: S,
    /// Optional population-size homeostatic loop.
    pub pop_loop: Option<PopulationSizeLoop>,
    /// Gene activated on newly recruited cells (M5 recruitment).
    pub seed_gene: Option<GeneId>,
    /// Convergence window (identical trailing fingerprints).
    pub settle_window: usize,
    /// Fingerprint trajectory recorded during [`crate::organism::Organism::settle`].
    pub trajectory: Vec<u128>,
    /// Next cell id for recruitment / mitosis sync.
    pub(crate) next_cell_id: u64,
    /// Previous homeostatic error (milli) for derivative term.
    pub(crate) prev_homeo_error: i64,
    /// Passive observation readout collector (M6).
    pub collector: ReadoutCollector,
    /// Gate promoting working state to committed checkpoints (M7).
    pub commit_gate: CommitmentGate,
    /// When true, auto-flush causal DAG after successful settle (default false).
    pub persist_on_settle: bool,
    /// Per-boundary-cell membrane policies (M8).
    pub(crate) boundary_policies: BTreeMap<CellId, MembranePolicy>,
    /// Last attached template (used when Breadth-scaling new surfaces).
    pub(crate) default_boundary_template: Option<BoundaryCellTemplate>,
    /// Escalation packets awaiting an external decision (M8).
    pub escalation_inbox: Vec<EscalationPacket>,
    /// Builds costed options when [`crate::organism::Organism::ingress`] escalates (M9).
    ///
    /// Defaults to [`build_echo_options`]; AEC replaces with domain builders.
    pub escalation_builder: fn(&crate::signal::Signal) -> Vec<EscalationOption>,
}

impl<S: Store> Organism<S> {
    /// Borrow cells in `CellId` order.
    #[must_use]
    pub fn cells(&self) -> &[crate::cell::Cell] {
        self.population.cells()
    }

    /// Count non-dead cells.
    #[must_use]
    pub fn living_count(&self) -> usize {
        self.population
            .cells()
            .iter()
            .filter(|c| c.lifecycle() != crate::cell::LifecycleState::Dead)
            .count()
    }

    /// Allocate the next cell id for recruitment.
    pub(crate) fn alloc_cell_id(&mut self) -> CellId {
        let id = CellId(self.next_cell_id);
        self.next_cell_id = self.next_cell_id.saturating_add(1);
        id
    }

    /// Effective K: sole non-empty ganglion's `space.k`, else scheduler cadence.
    #[must_use]
    pub fn effective_k(&self) -> u32 {
        let nonempty: Vec<_> = self
            .ganglia
            .iter()
            .filter(|g| !g.members.is_empty())
            .collect();
        if nonempty.len() == 1 {
            nonempty[0].space.k.max(1)
        } else {
            self.scheduler.cadence.k
        }
    }

    /// Attach a named ganglion with members and capacity.
    pub fn attach_ganglion(
        &mut self,
        handle: GanglionHandle,
        name: impl Into<String>,
        capacity: usize,
        space: SpaceConfig,
        members: impl IntoIterator<Item = CellId>,
    ) {
        let mut g = Ganglion::new(handle, name, capacity).with_space(space);
        for id in members {
            let _ = g.try_add(id);
        }
        g.refresh_health(self.population.cells());
        self.ganglia.push(g);
    }

    /// Inspect a ganglion as a unit.
    #[must_use]
    pub fn inspect_ganglion(&self, handle: GanglionHandle) -> Option<GanglionView> {
        self.ganglia
            .iter()
            .find(|g| g.handle == handle)
            .map(|g| ganglion_view(g, self.population.cells()))
    }

    /// Drain passive sensory readout samples.
    pub fn readout(&mut self) -> Vec<SignalSample> {
        self.collector.drain()
    }

    /// Run immune integrity scan.
    #[must_use]
    pub fn immune_flags(&self) -> Vec<ImmuneFlag> {
        validate_integrity(self.population.cells(), &self.ganglia)
    }

    /// Refresh health for all ganglia.
    pub fn refresh_ganglia_health(&mut self) {
        let cells = self.population.cells().to_vec();
        for g in &mut self.ganglia {
            g.refresh_health(&cells);
        }
    }

    /// Replace the escalation option builder (AEC / custom domains).
    pub fn set_escalation_builder(
        &mut self,
        builder: fn(&crate::signal::Signal) -> Vec<EscalationOption>,
    ) {
        self.escalation_builder = builder;
    }
}

/// Internal constructor used by the builder.
pub(crate) fn new_organism<S: Store>(
    genome: Arc<Genome>,
    population: Population,
    scheduler: Scheduler,
    store: S,
    pop_loop: Option<PopulationSizeLoop>,
    seed_gene: Option<GeneId>,
    next_cell_id: u64,
) -> Organism<S> {
    Organism {
        genome,
        population,
        ganglia: Vec::new(),
        medium: Delivery::new(),
        scheduler,
        clock: CausalClock::new(),
        store,
        pop_loop,
        seed_gene,
        settle_window: DEFAULT_CONVERGENCE_WINDOW,
        trajectory: Vec::new(),
        next_cell_id,
        prev_homeo_error: 0,
        collector: ReadoutCollector::new(),
        commit_gate: CommitmentGate::default(),
        persist_on_settle: false,
        boundary_policies: BTreeMap::new(),
        default_boundary_template: None,
        escalation_inbox: Vec::new(),
        escalation_builder: build_echo_options,
    }
}
