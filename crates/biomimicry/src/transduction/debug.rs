//! Inspector / decision-trace helpers for transduction.

use crate::cell::Cell;
use crate::genesis::GeneId;
use crate::transduction::CascadeTransducer;

/// Human-readable trace of whether a gene's cascade would run.
#[must_use]
pub fn decision_trace(transducer: &CascadeTransducer, cell: &Cell, gene: GeneId) -> String {
    let active = cell.expression.is_active(gene);
    let has = transducer.cascades.contains_key(&gene);
    format!("transduction decision\n  gene {gene:?}: active={active} cascade_registered={has}\n")
}
