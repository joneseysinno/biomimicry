//! Minimal population container for driving the M3 loop.
//!
//! Superseded by the full `organism` aggregate at M5. Cells are stored in a
//! `Vec` and always visited in stable [`CellId`] order — never hash iteration.

use crate::cell::{Cell, CellId};

/// Ordered population of cells.
#[derive(Debug, Clone, Default)]
pub struct Population {
    cells: Vec<Cell>,
}

impl Population {
    /// Create an empty population.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from cells (sorted by id).
    #[must_use]
    pub fn from_cells(mut cells: Vec<Cell>) -> Self {
        cells.sort_by_key(|c| c.id);
        Self { cells }
    }

    /// Add a cell and keep id order.
    pub fn push(&mut self, cell: Cell) {
        self.cells.push(cell);
        self.cells.sort_by_key(|c| c.id);
    }

    /// Number of cells.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Whether empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Borrow cells (already in `CellId` order).
    #[must_use]
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Mutable borrow of cells.
    pub fn cells_mut(&mut self) -> &mut [Cell] {
        &mut self.cells
    }

    /// Find a cell by id.
    #[must_use]
    pub fn get(&self, id: CellId) -> Option<&Cell> {
        self.cells.iter().find(|c| c.id == id)
    }

    /// Find a cell by id, mutably.
    pub fn get_mut(&mut self, id: CellId) -> Option<&mut Cell> {
        self.cells.iter_mut().find(|c| c.id == id)
    }

    /// Stable sorted list of cell ids.
    #[must_use]
    pub fn ids(&self) -> Vec<CellId> {
        self.cells.iter().map(|c| c.id).collect()
    }

    /// Shuffle storage order without changing ids (for P2). Re-sorts by id
    /// after — wait, P2 wants shuffled storage then harvest still stable.
    /// So we permute storage but harvest sorts by id.
    pub fn shuffle_storage(&mut self, order: &[usize]) {
        if order.len() != self.cells.len() {
            return;
        }
        let mut next = Vec::with_capacity(self.cells.len());
        for &i in order {
            if i < self.cells.len() {
                // placeholder — use swap-based permute
            }
        }
        let old = std::mem::take(&mut self.cells);
        for &i in order {
            if i < old.len() {
                next.push(old[i].clone());
            }
        }
        // If incomplete, fall back
        if next.len() == old.len() {
            self.cells = next;
        } else {
            self.cells = old;
        }
    }
}
