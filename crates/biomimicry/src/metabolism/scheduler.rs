//! Two nested loops — outer Phase 1, inner Phase 2 × K — with seeded ordering.
//!
//! **Sole time-driver.** Cells never self-advance. Under the `determinism`
//! feature (default), the loop is purely queue-driven — no wall-clock trigger
//! on the replay path. "Drained" means both scheduler queues are empty (not
//! attractor convergence — that is M5).
//!
//! `Transduce` ops harvested mid-cycle are **held** until after the next Phase 1
//! drain so expression changes land before cascades (M4 causal chain).

use crate::causality::{CausalEvent, CausalEventLog, Prng};
use crate::cell::{BehavioralMode, Cell, CellId, LifecycleState, Operation};
use crate::error::{BiomimicryError, Result};
use crate::expression::NetworkRegulator;
use crate::ganglion::Ganglion;
use crate::genesis::GeneId;
use crate::medium::{Medium, ScheduledOp};
use crate::metabolism::budget::OrganismEnergy;
use crate::metabolism::cadence::Cadence;
use crate::metabolism::phase1_queue::Phase1Queue;
use crate::metabolism::phase2_queue::Phase2Queue;
use crate::metabolism::population::Population;
use crate::metabolism::reactor::{Phase1Brain, Phase2Brain, Regulator, Transducer};
use crate::sensorium::{ReadoutCollector, SignalSample};
use crate::signal::{Payload, Phase, Scope, Signal, SignalType};
use crate::transduction::CascadeTransducer;
use std::sync::Arc;

/// Two-phase nested-loop scheduler.
#[derive(Debug)]
pub struct Scheduler {
    /// Global K-ratio.
    pub cadence: Cadence,
    /// Regulatory queue.
    pub phase1: Phase1Queue,
    /// Operational queue.
    pub phase2: Phase2Queue,
    /// Aggregate energy snapshot.
    pub energy: OrganismEnergy,
    /// Determinism seed.
    pub seed: u64,
    /// Integer PRNG (choice points only).
    prng: Prng,
    /// Delivery fabric.
    pub medium: Medium,
    /// Ordered causal event log (replay artifact).
    pub log: CausalEventLog,
    /// Phase 1 brain (default: Explicit).
    regulator: Phase1Brain,
    /// Phase 2 brain (default: Echo).
    transducer: Phase2Brain,
    /// `Transduce` ops waiting for a Phase 1 boundary before becoming runnable.
    held_transduce: Vec<ScheduledOp>,
    /// Outer cycles completed.
    pub outer_cycles: u32,
    /// Inner cycles completed.
    pub inner_cycles: u32,
    /// When true, `Divide*` spawns daughters (M6); otherwise typed-unavailable.
    pub divide_enabled: bool,
    /// Next cell id for mitotic daughters.
    pub next_daughter_id: u64,
    /// Gene activated on daughters (optional).
    pub seed_gene: Option<GeneId>,
    /// Parent→daughter pairs from the last divide batch (organism applies to ganglia).
    pending_lineage: Vec<(CellId, CellId)>,
    /// Ganglia snapshot for Cluster delivery (set by organism each tick).
    pub delivery_ganglia: Vec<Ganglion>,
    /// Passive observation buffer filled during deliver.
    pending_observations: Vec<SignalSample>,
}

impl Scheduler {
    /// Create a scheduler with the given seed and cadence.
    ///
    /// # Errors
    ///
    /// Returns `CadenceMisconfigured` when `cadence.k == 0`.
    pub fn try_new(seed: u64, cadence: Cadence) -> Result<Self> {
        cadence.validate()?;
        Ok(Self {
            cadence,
            phase1: Phase1Queue::new(),
            phase2: Phase2Queue::new(),
            energy: OrganismEnergy::default(),
            seed,
            prng: Prng::new(seed),
            medium: Medium::new(),
            log: CausalEventLog::new(),
            regulator: Phase1Brain::default(),
            transducer: Phase2Brain::default(),
            held_transduce: Vec::new(),
            outer_cycles: 0,
            inner_cycles: 0,
            divide_enabled: false,
            next_daughter_id: 1,
            seed_gene: None,
            pending_lineage: Vec::new(),
            delivery_ganglia: Vec::new(),
            pending_observations: Vec::new(),
        })
    }

    /// Create a scheduler (default K=10).
    #[must_use]
    pub fn new(seed: u64, cadence: Cadence) -> Self {
        Self::try_new(seed, cadence).expect("cadence.k must be ≥ 1")
    }

    /// Install a Phase 1 rule-network brain.
    pub fn with_regulator(&mut self, regulator: NetworkRegulator) -> &mut Self {
        self.regulator = Phase1Brain::Network(regulator);
        self
    }

    /// Install a Phase 2 cascade brain.
    pub fn with_transducer(&mut self, transducer: CascadeTransducer) -> &mut Self {
        self.transducer = Phase2Brain::Cascade(transducer);
        self
    }

    /// Whether both scheduler queues (and held transduction) are empty.
    #[must_use]
    pub fn is_drained(&self) -> bool {
        self.phase1.is_empty() && self.phase2.is_empty() && self.held_transduce.is_empty()
    }

    /// Enable mitotic divide execution (M6).
    pub fn enable_divide(&mut self, next_id: u64, seed_gene: Option<GeneId>) {
        self.divide_enabled = true;
        self.next_daughter_id = next_id;
        self.seed_gene = seed_gene;
    }

    /// Take parent→daughter pairs produced by divide ops.
    pub fn take_lineage(&mut self) -> Vec<(CellId, CellId)> {
        std::mem::take(&mut self.pending_lineage)
    }

    /// Take observation samples buffered during delivery.
    pub fn take_observations(&mut self) -> Vec<SignalSample> {
        std::mem::take(&mut self.pending_observations)
    }

    /// Replace the echo follow-on kind (no-op if cascade brain is installed).
    pub fn set_echo_kind(&mut self, kind: impl Into<crate::signal::SignalKind>) {
        if let Phase2Brain::Echo(echo) = &mut self.transducer {
            echo.follow_kind = kind.into();
        }
    }

    /// Inject a scheduled op directly (perturbation helper).
    pub fn inject(&mut self, op: ScheduledOp) {
        match op.op.phase() {
            Phase::Phase1 => self.phase1.push(op),
            Phase::Phase2 => {
                if matches!(op.op, Operation::Transduce { .. }) {
                    self.held_transduce.push(op);
                } else {
                    self.phase2.push(op);
                }
            }
        }
    }

    /// Run exactly `cycles` outer cycles (each with `cadence.k` inners).
    ///
    /// # Errors
    ///
    /// Propagates scheduling / scope errors.
    pub fn run(&mut self, population: &mut Population, cycles: u32) -> Result<()> {
        for _ in 0..cycles {
            self.outer_cycle(population)?;
        }
        Ok(())
    }

    /// Run until drained or `cap` outer cycles.
    ///
    /// # Errors
    ///
    /// Propagates scheduling / scope errors.
    pub fn run_until_drained(&mut self, population: &mut Population, cap: u32) -> Result<bool> {
        for _ in 0..cap {
            if self.is_drained() && population_pending_empty(population) {
                return Ok(true);
            }
            self.outer_cycle(population)?;
        }
        Ok(self.is_drained() && population_pending_empty(population))
    }

    /// One outer Phase 1 cycle including `K` inner Phase 2 cycles.
    ///
    /// # Errors
    ///
    /// Propagates scheduling / scope errors.
    pub fn outer_cycle(&mut self, population: &mut Population) -> Result<()> {
        OrganismEnergy::reset_p1_all(population.cells_mut());
        self.harvest(population);

        // Drain Phase 1 (expression apply via Regulator).
        self.drain_phase1(population)?;

        // Release held transduction only after expression has been applied.
        self.release_held_transduce();

        // Freeze expression for the whole inner loop — only P2 mutates queues.
        for _ in 0..self.cadence.k {
            self.inner_cycle(population)?;
        }

        self.outer_cycles = self.outer_cycles.saturating_add(1);
        self.energy.refresh_aggregate(population.cells());
        Ok(())
    }

    pub(crate) fn inner_cycle(&mut self, population: &mut Population) -> Result<()> {
        OrganismEnergy::reset_p2_all(population.cells_mut());
        // Absorb any Phase-2 ops cells enqueued during prior receives.
        self.harvest_phase2(population);
        self.phase2.sort();
        self.drain_phase2(population)?;
        self.inner_cycles = self.inner_cycles.saturating_add(1);
        Ok(())
    }

    /// Visit cells in stable `CellId` order; route pending by `op.phase()`.
    fn harvest(&mut self, population: &mut Population) {
        let mut ids = population.ids();
        ids.sort();
        for id in ids {
            self.absorb_cell(population, id);
        }
        self.phase1.sort();
        self.phase2.sort();
    }

    /// Test hook: harvest without draining.
    #[cfg(test)]
    pub(crate) fn harvest_for_test(&mut self, population: &mut Population) {
        self.harvest(population);
    }

    fn harvest_phase2(&mut self, population: &mut Population) {
        let mut ids = population.ids();
        ids.sort();
        for id in ids {
            self.absorb_cell(population, id);
        }
        self.phase2.sort();
    }

    fn absorb_cell(&mut self, population: &mut Population, id: CellId) {
        let Some(cell) = population.get_mut(id) else {
            return;
        };
        while let Some(op) = cell.pending.pop() {
            let scheduled = ScheduledOp { cell: id, op };
            match &scheduled.op {
                Operation::Transduce { .. } => self.held_transduce.push(scheduled),
                _ => match scheduled.op.phase() {
                    Phase::Phase1 => self.phase1.push(scheduled),
                    Phase::Phase2 => self.phase2.push(scheduled),
                },
            }
        }
    }

    fn release_held_transduce(&mut self) {
        let held = std::mem::take(&mut self.held_transduce);
        self.phase2.extend(held);
    }

    fn drain_phase1(&mut self, population: &mut Population) -> Result<()> {
        let mut deferred = Vec::new();
        while let Some(scheduled) = self.phase1.pop() {
            let Some(cell) = population.get_mut(scheduled.cell) else {
                continue;
            };
            if cell.lifecycle() == LifecycleState::Dead {
                continue;
            }
            match &scheduled.op {
                Operation::Express { .. } => {
                    if !OrganismEnergy::gate_express(cell) {
                        deferred.push(scheduled);
                        continue;
                    }
                    let delta = self
                        .regulator
                        .regulate(cell, std::slice::from_ref(&scheduled.op));
                    if OrganismEnergy::try_spend(cell, Phase::Phase1, 1) {
                        delta.apply(cell);
                        self.log_tag(scheduled.cell, "express");
                    } else {
                        deferred.push(scheduled);
                    }
                }
                Operation::Quiesce => {
                    let _ = cell.try_transition(LifecycleState::Quiescent);
                    // Suspend budgets while quiescent.
                    cell.energy.phase1.remaining_milli = 0;
                    cell.energy.phase2.remaining_milli = 0;
                    self.log_tag(scheduled.cell, "quiesce");
                }
                Operation::Die => {
                    self.execute_die(population, scheduled.cell)?;
                }
                Operation::Differentiate => {
                    let _ = cell.try_transition(LifecycleState::Differentiating);
                    self.log_tag(scheduled.cell, "differentiate");
                }
                Operation::DivideFast => {
                    self.execute_divide(population, scheduled.cell, false)?;
                }
                Operation::DivideSlow => {
                    self.execute_divide(population, scheduled.cell, true)?;
                }
                // Receive is Phase2; ignore if mis-routed.
                other => {
                    self.phase2.push(ScheduledOp {
                        cell: scheduled.cell,
                        op: other.clone(),
                    });
                }
            }
        }
        self.phase1.extend(deferred);
        Ok(())
    }

    fn drain_phase2(&mut self, population: &mut Population) -> Result<()> {
        // Snapshot length so newly appended ops run in later inners / sorts.
        let mut batch = Vec::new();
        while let Some(op) = self.phase2.pop() {
            batch.push(op);
        }
        for scheduled in batch {
            let Some(cell_life) = population.get(scheduled.cell).map(Cell::lifecycle) else {
                continue;
            };
            if cell_life == LifecycleState::Dead {
                continue;
            }
            match scheduled.op {
                Operation::Emit(signal) => {
                    let source_id = scheduled.cell;
                    self.log.push(CausalEvent {
                        parent: None,
                        child: signal.id,
                        cell: source_id,
                        stamp: signal.stamp,
                        tag: "emit".into(),
                    });
                    let delivers = {
                        let source = population.get(source_id).expect("source");
                        let mut collector = ReadoutCollector::new();
                        let ops = self.medium.deliver(
                            source,
                            population.cells(),
                            &signal,
                            &mut self.log,
                            &self.delivery_ganglia,
                            Some(&mut collector),
                        )?;
                        self.pending_observations.extend(collector.drain());
                        ops
                    };
                    self.phase2.extend(delivers);
                }
                Operation::Receive(signal) => {
                    if let Some(cell) = population.get_mut(scheduled.cell) {
                        let _reaction = cell.receive(&signal);
                        self.log.push(CausalEvent {
                            parent: Some(signal.id),
                            child: signal.id,
                            cell: scheduled.cell,
                            stamp: signal.stamp,
                            tag: "receive".into(),
                        });
                        self.absorb_cell(population, scheduled.cell);
                    }
                }
                Operation::Transduce { gene, input } => {
                    let cell_id = scheduled.cell;
                    let (ops, stamp) = {
                        let cell = population.get_mut(cell_id).expect("cell");
                        if !OrganismEnergy::gate_transduce(cell) {
                            continue;
                        }
                        if !OrganismEnergy::try_spend(cell, Phase::Phase2, 1) {
                            continue;
                        }
                        let stamp = cell.next_stamp();
                        // Prefer the match-trigger payload (operands); fall back
                        // to a stamped empty context when none was provided.
                        let mut ctx = input;
                        ctx.source = cell_id;
                        ctx.stamp = stamp;
                        let ops = self.transducer.transduce(cell, &ctx, gene);
                        (ops, stamp)
                    };
                    self.log.push(CausalEvent {
                        parent: None,
                        child: crate::signal::SignalId(stamp.0.unsigned_abs().into()),
                        cell: cell_id,
                        stamp,
                        tag: "transduce".into(),
                    });
                    for op in ops {
                        self.phase2.push(ScheduledOp { cell: cell_id, op });
                    }
                }
                // Regulatory ops found in P2 → hold on P1.
                op if op.phase() == Phase::Phase1 => {
                    self.phase1.push(ScheduledOp {
                        cell: scheduled.cell,
                        op,
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn execute_die(&mut self, population: &mut Population, id: CellId) -> Result<()> {
        // Notify 1-hop neighbors before transition.
        let death = {
            let cell = population.get(id).expect("dying cell");
            let stamp = cell.peek_stamp();
            Signal::new(
                SignalType::Regulatory,
                "cell_died",
                Scope::Neighbors,
                Payload::empty(),
                id,
                stamp,
            )
        };
        let notifies = {
            let source = population.get(id).expect("dying");
            match self.medium.deliver(
                source,
                population.cells(),
                &death,
                &mut self.log,
                &self.delivery_ganglia,
                None,
            ) {
                Ok(ops) => ops,
                Err(BiomimicryError::ScopeUnavailable { .. }) => Vec::new(),
                Err(e) => return Err(e),
            }
        };
        self.phase2.extend(notifies);

        if let Some(cell) = population.get_mut(id) {
            let from = cell.lifecycle();
            if from != LifecycleState::Dead {
                // Active|Quiescent → Dead
                if from == LifecycleState::Active || from == LifecycleState::Quiescent {
                    let _ = cell.try_transition(LifecycleState::Dead);
                }
            }
        }
        self.phase1.purge_cell(id);
        self.phase2.purge_cell(id);
        self.held_transduce.retain(|op| op.cell != id);
        self.medium.drop_in_flight(id);
        self.log_tag(id, "die");
        Ok(())
    }

    fn execute_divide(
        &mut self,
        population: &mut Population,
        parent_id: CellId,
        slow: bool,
    ) -> Result<()> {
        if !self.divide_enabled {
            return Err(BiomimicryError::OperationUnavailable {
                op: if slow { "divide-slow" } else { "divide-fast" },
                since_milestone: 6,
            });
        }
        let (genome, genes) = {
            let parent = population
                .get(parent_id)
                .ok_or_else(|| BiomimicryError::Metabolism("divide: missing parent".into()))?;
            if parent.lifecycle() == LifecycleState::Dead {
                return Ok(());
            }
            let genes: Vec<GeneId> = parent.expression.active_genes().collect();
            (Arc::clone(&parent.genome), genes)
        };
        if slow {
            if let Some(parent) = population.get_mut(parent_id) {
                parent.mode = BehavioralMode::DividingSlow;
            }
        }
        let daughter_id = CellId(self.next_daughter_id);
        self.next_daughter_id = self.next_daughter_id.saturating_add(1);
        let mut daughter = Cell::new(daughter_id, genome);
        daughter.try_transition(LifecycleState::Differentiating)?;
        daughter.try_transition(LifecycleState::Active)?;
        if let Some(g) = self.seed_gene {
            daughter.activate(g);
        } else {
            for g in genes {
                daughter.activate(g);
            }
        }
        population.push(daughter);
        self.pending_lineage.push((parent_id, daughter_id));
        self.log_tag(parent_id, if slow { "divide-slow" } else { "divide-fast" });
        Ok(())
    }

    fn log_tag(&mut self, cell: CellId, tag: &'static str) {
        self.log.push(CausalEvent {
            parent: None,
            child: crate::signal::SignalId(u128::from(self.prng.next_u64())),
            cell,
            stamp: crate::causality::CausalStamp(i64::from(self.outer_cycles)),
            tag: tag.into(),
        });
    }

    /// Wall-clock trigger — only compiled when determinism is off.
    #[cfg(not(feature = "determinism"))]
    pub fn wall_clock_tick(&mut self, population: &mut Population) -> Result<()> {
        self.outer_cycle(population)
    }
}

fn population_pending_empty(population: &Population) -> bool {
    population.cells().iter().all(|c| c.pending.is_empty())
}
