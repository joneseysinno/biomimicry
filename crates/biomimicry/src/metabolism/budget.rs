//! Budget accounting policy over M2's per-cell [`EnergyBudget`].
//!
//! Sole authority for reset cadence: P1 once per outer cycle, P2 once per
//! inner cycle. Exhaustion effects follow Part III.1:
//! - exhausted P1 → hold expression, keep transducing
//! - exhausted P2 → operationally quiescent, keep differentiating

use crate::cell::{Cell, EnergyBudget};
use crate::signal::Phase;

/// Organism-level energy accounting helpers.
#[derive(Debug, Clone, Default)]
pub struct OrganismEnergy {
    /// Aggregate snapshot (informative; per-cell budgets are authoritative).
    pub aggregate: EnergyBudget,
}

impl OrganismEnergy {
    /// Reset every cell's Phase 1 budget (call at the top of each outer cycle).
    pub fn reset_p1_all(population: &mut [Cell]) {
        for cell in population {
            cell.energy.reset_p1();
        }
    }

    /// Reset every cell's Phase 2 budget (call at the top of each inner cycle).
    pub fn reset_p2_all(population: &mut [Cell]) {
        for cell in population {
            cell.energy.reset_p2();
        }
    }

    /// Whether the cell may apply an expression change (P1 gate).
    #[must_use]
    pub fn gate_express(cell: &Cell) -> bool {
        cell.energy.can_express()
    }

    /// Whether the cell may run a transduction (P2 gate).
    #[must_use]
    pub fn gate_transduce(cell: &Cell) -> bool {
        cell.energy.can_transduce()
    }

    /// Spend one unit on the given phase if possible.
    pub fn try_spend(cell: &mut Cell, phase: Phase, cost: i64) -> bool {
        cell.energy.try_spend(phase, cost)
    }

    /// Refresh the aggregate snapshot from the population.
    pub fn refresh_aggregate(&mut self, population: &[Cell]) {
        let mut p1 = 0i64;
        let mut p2 = 0i64;
        let mut c1 = 0i64;
        let mut c2 = 0i64;
        for cell in population {
            p1 = p1.saturating_add(cell.energy.phase1.remaining_milli);
            p2 = p2.saturating_add(cell.energy.phase2.remaining_milli);
            c1 = c1.saturating_add(cell.energy.phase1.capacity_milli);
            c2 = c2.saturating_add(cell.energy.phase2.capacity_milli);
        }
        self.aggregate.phase1.remaining_milli = p1;
        self.aggregate.phase1.capacity_milli = c1;
        self.aggregate.phase2.remaining_milli = p2;
        self.aggregate.phase2.capacity_milli = c2;
    }
}
