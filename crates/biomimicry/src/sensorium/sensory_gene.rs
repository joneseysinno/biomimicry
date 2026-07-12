//! Four-gene sensory template (low threshold, refractory, …).
//!
//! Design IX.3 maps to fields: `amplify` ≈ temporal integration gene,
//! `report` ≈ narrow emission / report gene. Milli policy gates live on
//! [`crate::sensorium::SensoryPolicy`], configured via
//! [`SensoryGeneTemplate::to_policy`].

use crate::genesis::GeneId;
use crate::sensorium::SensoryPolicy;

/// Template for sensory expression patterns (not a cell type enum).
#[derive(Debug, Clone, Default)]
pub struct SensoryGeneTemplate {
    /// Low-threshold receptor gene.
    pub low_threshold: Option<GeneId>,
    /// Refractory / recovery gene.
    pub refractory: Option<GeneId>,
    /// Amplification / temporal-integration gene.
    pub amplify: Option<GeneId>,
    /// Report / sample emission gene.
    pub report: Option<GeneId>,
    /// Mechanical threshold (milli); 0 = off.
    pub threshold_milli: u32,
    /// Mechanical refractory window (logical millis); 0 = off.
    pub refractory_milli: u32,
}

impl SensoryGeneTemplate {
    /// Create an empty sensory template.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a [`SensoryPolicy`] from milli fields.
    #[must_use]
    pub fn to_policy(&self) -> SensoryPolicy {
        SensoryPolicy::new(self.threshold_milli, self.refractory_milli)
    }
}
