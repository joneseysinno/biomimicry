//! Bounded set of cell handles tracked as a unit.

use super::{GanglionHandle, GanglionHealth};
use crate::cell::CellId;

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
}

impl Ganglion {
    /// Create an empty named ganglion with a capacity bound.
    #[must_use]
    pub fn new(handle: GanglionHandle, name: impl Into<String>, capacity: usize) -> Self {
        Self {
            handle,
            name: name.into(),
            members: Vec::new(),
            capacity,
            health: GanglionHealth::Healthy,
        }
    }

    /// Add a cell if under capacity.
    ///
    /// Returns `false` if the ganglion is at capacity.
    pub fn try_add(&mut self, _cell: CellId) -> bool {
        todo!("add cell under capacity bound")
    }

    /// Recompute collective health from member lifecycles.
    pub fn refresh_health(&mut self) {
        todo!("derive GanglionHealth from members")
    }
}
