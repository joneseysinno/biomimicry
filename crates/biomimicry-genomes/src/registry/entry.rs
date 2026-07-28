//! Registry entry for a default genome.

use biomimicry::error::Result;
use biomimicry::genesis::Grn;
use biomimicry::substrate::Store;

use crate::engineer_calculator::{
    SeedOutcome, calculator_dna, seed_engineer_calculator_dyn,
};

/// Stable name → DNA builder + seed helper.
#[derive(Debug, Clone, Copy)]
pub struct GenomeEntry {
    /// Stable registry name (e.g. `"engineer_calculator"`).
    pub name: &'static str,
    /// Build the genome DNA from code.
    pub build_dna: fn() -> Grn,
    /// Seed-on-first-run into a [`Store`].
    pub seed: fn(&mut dyn Store) -> Result<SeedOutcome>,
}

/// All default genomes shipped by this crate.
#[must_use]
pub fn default_genomes() -> &'static [GenomeEntry] {
    &DEFAULTS
}

const DEFAULTS: [GenomeEntry; 1] = [GenomeEntry {
    name: "engineer_calculator",
    build_dna: calculator_dna,
    seed: seed_engineer_calculator_dyn,
}];

/// Look up a default genome by stable name.
#[must_use]
pub fn find_genome(name: &str) -> Option<&'static GenomeEntry> {
    default_genomes().iter().find(|e| e.name == name)
}
