//! Assemble an organism from a genome + config.

use std::sync::Arc;

use crate::error::Result;
use crate::genesis::Genome;
use crate::metabolism::Cadence;
use crate::organism::Organism;
use crate::substrate::MemoryStore;

/// Builder for [`Organism`].
#[derive(Debug)]
pub struct OrganismBuilder {
    genome: Option<Arc<Genome>>,
    seed: u64,
    cadence: Cadence,
}

impl Default for OrganismBuilder {
    fn default() -> Self {
        Self {
            genome: None,
            seed: 0,
            cadence: Cadence::default(),
        }
    }
}

impl OrganismBuilder {
    /// Create a builder with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the compiled genome.
    #[must_use]
    pub fn genome(mut self, genome: impl Into<Arc<Genome>>) -> Self {
        self.genome = Some(genome.into());
        self
    }

    /// Set the determinism seed.
    #[must_use]
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Set scheduler cadence.
    #[must_use]
    pub fn cadence(mut self, cadence: Cadence) -> Self {
        self.cadence = cadence;
        self
    }

    /// Build the organism.
    ///
    /// # Errors
    ///
    /// Returns an error if required configuration is missing.
    pub fn build(self) -> Result<Organism<MemoryStore>> {
        let _ = (self.genome, self.seed, self.cadence);
        todo!("assemble organism from genome + config")
    }
}
