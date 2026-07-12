//! Gene templates for protocol-matching surface cells.

use crate::genesis::GeneId;

/// Template describing boundary-cell receptor / secretion genes.
#[derive(Debug, Clone, Default)]
pub struct BoundaryCellTemplate {
    /// Receptor genes that match an external protocol.
    pub receptors: Vec<GeneId>,
    /// Secretion / emission genes for replies.
    pub secretions: Vec<GeneId>,
}

impl BoundaryCellTemplate {
    /// Create an empty boundary template.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
