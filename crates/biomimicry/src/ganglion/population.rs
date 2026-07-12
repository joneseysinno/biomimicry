//! Bounded set of cell handles tracked as a unit.

use super::{GanglionHandle, GanglionHealth};
use crate::cell::{Cell, CellId, LifecycleState};
use crate::metabolism::SpaceConfig;

/// Named, bounded cell population — the engine's "function/module."
#[derive(Debug, Clone)]
pub struct Ganglion {
    /// Stable lineage handle.
    pub handle: GanglionHandle,
    /// Human-readable name.
    pub name: String,
    /// Member cells.
    pub members: Vec<CellId>,
    /// Maximum population size.
    pub capacity: usize,
    /// Collective health.
    pub health: GanglionHealth,
    /// Per-ganglion Phase-2 K seam (M6).
    pub space: SpaceConfig,
}

impl Ganglion {
    /// Create an empty named ganglion with a capacity bound (default K=10).
    #[must_use]
    pub fn new(handle: GanglionHandle, name: impl Into<String>, capacity: usize) -> Self {
        Self {
            handle,
            name: name.into(),
            members: Vec::new(),
            capacity,
            health: GanglionHealth::Healthy,
            space: SpaceConfig { k: 10 },
        }
    }

    /// Builder: set per-ganglion K.
    #[must_use]
    pub fn with_space(mut self, space: SpaceConfig) -> Self {
        self.space = space;
        self
    }

    /// Whether `cell` is a member.
    #[must_use]
    pub fn contains(&self, cell: CellId) -> bool {
        self.members.contains(&cell)
    }

    /// Add a cell if under capacity and not already a member.
    ///
    /// Returns `false` if at capacity (duplicates are ignored as success).
    pub fn try_add(&mut self, cell: CellId) -> bool {
        if self.members.contains(&cell) {
            return true;
        }
        if self.members.len() >= self.capacity {
            return false;
        }
        self.members.push(cell);
        true
    }

    /// Recompute collective health from member lifecycles in `population`.
    pub fn refresh_health(&mut self, population: &[Cell]) {
        if self.members.is_empty() {
            self.health = GanglionHealth::Dead;
            return;
        }
        let mut active = 0usize;
        let mut any_bad = false;
        let mut all_dead = true;
        for id in &self.members {
            let life = population
                .iter()
                .find(|c| c.id == *id)
                .map_or(LifecycleState::Dead, Cell::lifecycle);
            match life {
                LifecycleState::Dead => any_bad = true,
                LifecycleState::Quiescent => {
                    any_bad = true;
                    all_dead = false;
                }
                LifecycleState::Active => {
                    active += 1;
                    all_dead = false;
                }
                _ => {
                    all_dead = false;
                }
            }
        }
        if all_dead {
            self.health = GanglionHealth::Dead;
            return;
        }
        let frac_milli = (active.saturating_mul(1000)) / self.members.len();
        if any_bad || frac_milli < 500 {
            self.health = GanglionHealth::Degraded;
        } else {
            self.health = GanglionHealth::Healthy;
        }
    }
}

/// Snapshot for inspect-as-unit API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GanglionView {
    /// Handle.
    pub handle: GanglionHandle,
    /// Name.
    pub name: String,
    /// Health.
    pub health: GanglionHealth,
    /// Members.
    pub members: Vec<CellId>,
    /// Non-dead member count.
    pub living: usize,
    /// Capacity.
    pub capacity: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;
    use crate::genesis::{compile, toy_dna};
    use std::sync::Arc;

    #[test]
    fn try_add_respects_capacity() {
        let mut g = Ganglion::new(GanglionHandle(1), "g", 1);
        assert!(g.try_add(CellId(1)));
        assert!(!g.try_add(CellId(2)));
        assert!(g.try_add(CellId(1))); // duplicate ok
    }

    #[test]
    fn health_dead_when_all_dead() {
        let genome = compile(&toy_dna()).unwrap();
        let mut a = Cell::new(CellId(1), Arc::clone(&genome));
        a.try_transition(LifecycleState::Differentiating).unwrap();
        a.try_transition(LifecycleState::Active).unwrap();
        a.try_transition(LifecycleState::Dead).unwrap();
        let mut g = Ganglion::new(GanglionHandle(1), "g", 2);
        g.try_add(CellId(1));
        g.refresh_health(std::slice::from_ref(&a));
        assert_eq!(g.health, GanglionHealth::Dead);
    }
}
