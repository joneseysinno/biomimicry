//! `CascadeTransducer` — real Phase 2 brain behind the M3 `Transducer` seam.

use std::collections::BTreeMap;

use crate::cell::{Cell, Operation};
use crate::error::{BiomimicryError, Result};
use crate::genesis::GeneId;
use crate::metabolism::Transducer;
use crate::signal::Signal;
use crate::transduction::{Cascade, emit_from_cascade};

/// Phase 2 transducer driven by per-gene cascades.
#[derive(Debug, Clone, Default)]
pub struct CascadeTransducer {
    /// Cascades keyed by gene id.
    pub cascades: BTreeMap<GeneId, Cascade>,
}

impl CascadeTransducer {
    /// Create an empty transducer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: register a cascade for a gene.
    #[must_use]
    pub fn with_cascade(mut self, gene: GeneId, cascade: Cascade) -> Self {
        self.cascades.insert(gene, cascade);
        self
    }

    /// Run transduction for `gene`, or return a typed error if missing.
    ///
    /// # Errors
    ///
    /// Returns [`BiomimicryError::CascadeUnavailable`] when no cascade is
    /// registered for an active gene.
    pub fn transduce_checked(
        &self,
        cell: &Cell,
        sig: &Signal,
        gene: GeneId,
    ) -> Result<Vec<Operation>> {
        if !cell.expression.is_active(gene) {
            return Ok(Vec::new());
        }
        let Some(cascade) = self.cascades.get(&gene) else {
            return Err(BiomimicryError::CascadeUnavailable { gene });
        };
        let outputs = cascade.run(&cell.expression, sig)?;
        let stamp = cell.peek_stamp();
        let signals = emit_from_cascade(outputs, cell.id, stamp)?;
        Ok(signals.into_iter().map(Operation::Emit).collect())
    }
}

impl Transducer for CascadeTransducer {
    fn transduce(&self, cell: &Cell, sig: &Signal, gene: GeneId) -> Vec<Operation> {
        self.transduce_checked(cell, sig, gene).unwrap_or_default()
    }
}
