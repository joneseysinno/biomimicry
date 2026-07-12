//! Four-gene sensory template (low threshold, refractory, …).

use crate::genesis::GeneId;

/// Template for sensory expression patterns (not a cell type enum).
#[derive(Debug, Clone, Default)]
pub struct SensoryGeneTemplate {
    /// Low-threshold receptor gene.
    pub low_threshold: Option<GeneId>,
    /// Refractory / recovery gene.
    pub refractory: Option<GeneId>,
    /// Amplification gene.
    pub amplify: Option<GeneId>,
    /// Report / sample emission gene.
    pub report: Option<GeneId>,
}

impl SensoryGeneTemplate {
    /// Create an empty sensory template.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
