//! Gene templates for protocol-matching surface cells.

use crate::cell::Cell;
use crate::genesis::GeneId;
use crate::membrane::escalation::MembranePolicy;

/// Template describing boundary-cell receptor / secretion genes.
#[derive(Debug, Clone, Default)]
pub struct BoundaryCellTemplate {
    /// Receptor genes that match an external protocol.
    pub receptors: Vec<GeneId>,
    /// Secretion / emission genes for replies.
    pub secretions: Vec<GeneId>,
    /// Escalate when inbound strength ≥ this milli (0 = strength never escalates).
    pub escalation_strength_milli: u32,
}

impl BoundaryCellTemplate {
    /// Create an empty boundary template.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: set receptor genes.
    #[must_use]
    pub fn with_receptors(mut self, receptors: impl IntoIterator<Item = GeneId>) -> Self {
        self.receptors = receptors.into_iter().collect();
        self
    }

    /// Builder: set secretion genes.
    #[must_use]
    pub fn with_secretions(mut self, secretions: impl IntoIterator<Item = GeneId>) -> Self {
        self.secretions = secretions.into_iter().collect();
        self
    }

    /// Builder: set escalation strength threshold (milli).
    #[must_use]
    pub fn with_escalation_strength_milli(mut self, milli: u32) -> Self {
        self.escalation_strength_milli = milli;
        self
    }

    /// Activate receptor and secretion genes on a cell.
    pub fn apply_to(&self, cell: &mut Cell) {
        for gene in self.receptors.iter().chain(self.secretions.iter()) {
            cell.activate(*gene);
        }
    }

    /// Build a [`MembranePolicy`] from template milli fields.
    #[must_use]
    pub fn to_policy(&self) -> MembranePolicy {
        MembranePolicy {
            escalation_strength_milli: self.escalation_strength_milli,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::{CellId, LifecycleState};
    use crate::genesis::{cascade_dna, compile};
    use std::sync::Arc;

    #[test]
    fn p3_apply_to_activates_listed_genes() {
        let genome = Arc::new(compile(&cascade_dna()).unwrap());
        let gene = genome.iter().next().unwrap().id;
        let mut cell = Cell::new(CellId(1), Arc::clone(&genome));
        cell.try_transition(LifecycleState::Differentiating)
            .unwrap();
        cell.try_transition(LifecycleState::Active).unwrap();
        let tmpl = BoundaryCellTemplate::new().with_receptors([gene]);
        tmpl.apply_to(&mut cell);
        assert!(cell.expression.is_active(gene));
    }
}
