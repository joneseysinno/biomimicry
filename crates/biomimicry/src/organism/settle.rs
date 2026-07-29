//! Drive the scheduler until convergence or timeout.

use std::sync::Arc;

use crate::attractor::{
    SettleStatus, detect_convergence, detect_divergence, expression_fingerprint,
};
use crate::cell::{LifecycleState, Operation};
use crate::error::Result;
use crate::homeostasis::HomeostaticLoop;
use crate::medium::ScheduledOp;
use crate::organism::Organism;
use crate::substrate::Store;

impl<S: Store> Organism<S> {
    /// Drive metabolism until the organism settles or the timeout elapses.
    ///
    /// Each tick: optional population-size homeostasis → one scheduler outer
    /// cycle → record expression fingerprint → convergence check.
    ///
    /// # Errors
    ///
    /// Returns an error if scheduling or homeostasis fails.
    pub fn settle(&mut self, max_ticks: u64) -> Result<SettleStatus> {
        self.trajectory.clear();
        let window = self.settle_window.max(1);

        for _ in 0..max_ticks {
            self.homeostasis_tick()?;
            self.scheduler.delivery_ganglia = self.ganglia.clone();
            // Align K with effective_k when a single ganglion owns the circuit.
            self.scheduler.cadence.k = self.effective_k();
            self.scheduler.outer_cycle(&mut self.population)?;
            self.drain_effects_into_sink();
            for sample in self.scheduler.take_observations() {
                self.collector.observe(sample);
            }
            for (parent, daughter) in self.scheduler.take_lineage() {
                for g in &mut self.ganglia {
                    if g.contains(parent) {
                        let _ = g.try_add(daughter);
                    }
                }
            }
            self.next_cell_id = self.scheduler.next_daughter_id;
            self.refresh_ganglia_health();

            let fp = expression_fingerprint(&self.population);
            self.trajectory.push(fp);

            if detect_convergence(&self.trajectory, window) == SettleStatus::Converged {
                if self.persist_on_settle {
                    self.flush_causal()?;
                }
                return Ok(SettleStatus::Converged);
            }
        }

        // Final classification: limit-cycle vs timed-out transient.
        let _ = detect_divergence(&self.trajectory);
        Ok(SettleStatus::TimedOut)
    }

    /// Fingerprint trajectory recorded by the last [`Self::settle`] call.
    #[must_use]
    pub fn trajectory(&self) -> &[u128] {
        &self.trajectory
    }

    fn homeostasis_tick(&mut self) -> Result<()> {
        let Some(loop_) = self.pop_loop.as_mut() else {
            return Ok(());
        };
        loop_.current = self
            .population
            .cells()
            .iter()
            .filter(|c| c.lifecycle() != LifecycleState::Dead)
            .count();
        let prev = self.prev_homeo_error;
        let err = loop_.step(prev)?;
        self.prev_homeo_error = err;

        let recruit = loop_.take_recruit();
        let cull = loop_.take_cull();
        self.apply_cull(cull);
        self.apply_recruit(recruit)?;
        Ok(())
    }

    fn apply_cull(&mut self, n: usize) {
        if n == 0 {
            return;
        }
        // Cull highest CellId Active cells first (stable order).
        let mut victims: Vec<_> = self
            .population
            .cells()
            .iter()
            .filter(|c| c.lifecycle() == LifecycleState::Active)
            .map(|c| c.id)
            .collect();
        victims.sort_by(|a, b| b.cmp(a));
        for id in victims.into_iter().take(n) {
            self.scheduler.inject(ScheduledOp {
                cell: id,
                op: Operation::Die,
            });
        }
    }

    pub(crate) fn apply_recruit(&mut self, n: usize) -> Result<()> {
        if n == 0 {
            return Ok(());
        }
        let seed = self.seed_gene;
        let genome = Arc::clone(&self.genome);
        let mut remaining = n;

        // Reuse Dead cell slots (same CellId) so undamped pop loops can cycle.
        let dead_ids: Vec<_> = self
            .population
            .cells()
            .iter()
            .filter(|c| c.lifecycle() == LifecycleState::Dead)
            .map(|c| c.id)
            .collect();
        for id in dead_ids {
            if remaining == 0 {
                break;
            }
            if let Some(cell) = self.population.get_mut(id) {
                // Rebuild a fresh Active cell in-place with the same id.
                let mut fresh = crate::cell::Cell::new(id, Arc::clone(&genome));
                fresh.try_transition(LifecycleState::Differentiating)?;
                fresh.try_transition(LifecycleState::Active)?;
                if let Some(g) = seed {
                    fresh.activate(g);
                }
                *cell = fresh;
                remaining -= 1;
            }
        }

        for _ in 0..remaining {
            let id = self.alloc_cell_id();
            let mut cell = crate::cell::Cell::new(id, Arc::clone(&genome));
            cell.try_transition(LifecycleState::Differentiating)?;
            cell.try_transition(LifecycleState::Active)?;
            if let Some(g) = seed {
                cell.activate(g);
            }
            self.population.push(cell);
        }
        Ok(())
    }
}
