//! Bifurcated Phase 1 / Phase 2 energy budgets (integer millis).
//!
//! Exhaustion rule (Part III.1): exhausted P2 + live P1 =
//! differentiating-but-not-acting — encoded as [`EnergyBudget::is_differentiating_but_not_acting`].

use crate::signal::Phase;

/// Single energy pool in integer millis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Budget {
    /// Remaining units this cycle.
    pub remaining_milli: i64,
    /// Capacity restored on reset.
    pub capacity_milli: i64,
}

impl Budget {
    /// Full budget at `capacity_milli`.
    #[must_use]
    pub const fn full(capacity_milli: i64) -> Self {
        Self {
            remaining_milli: capacity_milli,
            capacity_milli,
        }
    }

    /// Whether any spend of `cost` would succeed.
    #[must_use]
    pub const fn can_spend(self, cost: i64) -> bool {
        cost >= 0 && self.remaining_milli >= cost
    }

    /// Debit `cost`; returns `false` without mutating when insufficient.
    pub fn try_spend(&mut self, cost: i64) -> bool {
        if !self.can_spend(cost) {
            return false;
        }
        self.remaining_milli -= cost;
        true
    }

    /// Restore remaining to capacity.
    pub fn reset(&mut self) {
        self.remaining_milli = self.capacity_milli;
    }
}

impl Default for Budget {
    fn default() -> Self {
        Self::full(1_000)
    }
}

/// Separate Phase 1 / Phase 2 pools so they cannot silently starve each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnergyBudget {
    /// Phase 1 (regulatory / expression) pool.
    pub phase1: Budget,
    /// Phase 2 (operational / transduction) pool.
    pub phase2: Budget,
}

impl Default for EnergyBudget {
    fn default() -> Self {
        Self {
            phase1: Budget::full(1_000),
            phase2: Budget::full(1_000),
        }
    }
}

impl EnergyBudget {
    /// Create budgets with the given capacities (starting full).
    #[must_use]
    pub const fn new(phase1_capacity: i64, phase2_capacity: i64) -> Self {
        Self {
            phase1: Budget::full(phase1_capacity),
            phase2: Budget::full(phase2_capacity),
        }
    }

    /// Whether expression writes are still affordable.
    #[must_use]
    pub const fn can_express(&self) -> bool {
        self.phase1.remaining_milli > 0
    }

    /// Whether transduction events are still affordable.
    #[must_use]
    pub const fn can_transduce(&self) -> bool {
        self.phase2.remaining_milli > 0
    }

    /// Exhausted P2 + live P1 ⇒ differentiating-but-not-acting (Part III.1).
    #[must_use]
    pub const fn is_differentiating_but_not_acting(&self) -> bool {
        self.can_express() && !self.can_transduce()
    }

    /// Try to spend Phase 1 energy.
    pub fn try_spend_p1(&mut self, cost: i64) -> bool {
        self.phase1.try_spend(cost)
    }

    /// Try to spend Phase 2 energy.
    pub fn try_spend_p2(&mut self, cost: i64) -> bool {
        self.phase2.try_spend(cost)
    }

    /// Reset Phase 1 remaining to capacity.
    pub fn reset_p1(&mut self) {
        self.phase1.reset();
    }

    /// Reset Phase 2 remaining to capacity.
    pub fn reset_p2(&mut self) {
        self.phase2.reset();
    }

    /// Spend against the pool for `phase`.
    pub fn try_spend(&mut self, phase: Phase, cost: i64) -> bool {
        match phase {
            Phase::Phase1 => self.try_spend_p1(cost),
            Phase::Phase2 => self.try_spend_p2(cost),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spend_exhaust_reset() {
        let mut e = EnergyBudget::new(100, 50);
        assert!(e.try_spend_p1(60));
        assert_eq!(e.phase1.remaining_milli, 40);
        assert!(!e.try_spend_p1(50));
        e.reset_p1();
        assert_eq!(e.phase1.remaining_milli, 100);
    }

    #[test]
    fn differentiating_but_not_acting_truth_table() {
        let mut e = EnergyBudget::new(100, 100);
        assert!(!e.is_differentiating_but_not_acting());
        assert!(e.try_spend_p2(100));
        assert!(e.is_differentiating_but_not_acting());
        assert!(e.try_spend_p1(100));
        assert!(!e.is_differentiating_but_not_acting());
    }
}
