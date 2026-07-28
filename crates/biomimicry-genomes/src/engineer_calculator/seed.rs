//! Seed-on-first-run helper for the engineering calculator genome.

use biomimicry::error::Result;
use biomimicry::substrate::Store;

use crate::engineer_calculator::dna::calculator_dna;
use crate::engineer_calculator::kinds::SCHEMA_STAMP;

/// Outcome of a seed attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedOutcome {
    /// Genome was written from `calculator_dna()`.
    Seeded,
    /// Schema stamp already present — no write.
    AlreadyPresent,
}

/// Whether the engineer_calculator schema stamp is already in `store`.
///
/// # Errors
///
/// Propagates Store I/O errors.
pub fn is_engineer_calculator_seeded(store: &dyn Store) -> Result<bool> {
    Ok(store.iter_cistrons()?.iter().any(|c| {
        c.kind.as_str() == "genome_stamp"
            && c.endpoints
                .iter()
                .any(|ep| ep.role.as_str() == SCHEMA_STAMP)
    }))
}

/// Seed the engineering calculator genome if absent (schema-version stamp).
///
/// Regenerates from code so GeneIds are always computed under the current
/// engine version.
///
/// # Errors
///
/// Propagates Store I/O errors.
pub fn seed_engineer_calculator_dyn(store: &mut dyn Store) -> Result<SeedOutcome> {
    if is_engineer_calculator_seeded(store)? {
        return Ok(SeedOutcome::AlreadyPresent);
    }
    calculator_dna().persist(store)?;
    Ok(SeedOutcome::Seeded)
}

/// Seed helper for concrete [`Store`] backends.
///
/// # Errors
///
/// Propagates Store I/O errors.
pub fn seed_engineer_calculator<S: Store>(store: &mut S) -> Result<SeedOutcome> {
    seed_engineer_calculator_dyn(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use biomimicry::genesis::compile;
    use biomimicry::substrate::MemoryStore;

    #[test]
    fn g3_seed_round_trips_memory_store() {
        let mut store = MemoryStore::new();
        assert!(!is_engineer_calculator_seeded(&store).unwrap());
        assert_eq!(
            seed_engineer_calculator(&mut store).unwrap(),
            SeedOutcome::Seeded
        );
        assert_eq!(
            seed_engineer_calculator(&mut store).unwrap(),
            SeedOutcome::AlreadyPresent
        );

        let loaded = store.load_grn().unwrap();
        let expected = calculator_dna();
        assert_eq!(loaded.cistron_count(), expected.cistron_count());
        let g1 = compile(&expected).unwrap();
        let g2 = compile(&loaded).unwrap();
        assert_eq!(g1.len(), g2.len());
        assert_eq!(g1.traversed_ids(), g2.traversed_ids());
    }
}
