//! Expression-state fingerprints for settle detection.

use blake3::Hasher;

use crate::cell::{Cell, LifecycleState};
use crate::genesis::hash::finalize_u128;
use crate::metabolism::Population;

/// BLAKE3₁₂₈ fingerprint of living cells' expression state.
///
/// Absorbs stable `CellId` order, skipping `Dead` cells. For each living cell:
/// `cell_id` then sorted active `GeneId`s.
#[must_use]
pub fn expression_fingerprint(population: &Population) -> u128 {
    fingerprint_cells(population.cells())
}

/// Fingerprint a cell slice (already `CellId`-ordered preferred).
#[must_use]
pub fn fingerprint_cells(cells: &[Cell]) -> u128 {
    let mut hasher = Hasher::new();
    let mut ordered: Vec<&Cell> = cells
        .iter()
        .filter(|c| c.lifecycle() != LifecycleState::Dead)
        .collect();
    ordered.sort_by_key(|c| c.id);
    for cell in ordered {
        hasher.update(&cell.id.0.to_le_bytes());
        let mut genes: Vec<_> = cell.expression.active_genes().collect();
        genes.sort_unstable();
        hasher.update(&(genes.len() as u64).to_le_bytes());
        for g in genes {
            hasher.update(&g.0.to_le_bytes());
        }
    }
    finalize_u128(&hasher)
}
