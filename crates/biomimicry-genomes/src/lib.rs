//! Default genomes for biomimicry engineering apps.
//!
//! Pure DNA-as-code plus seed helpers. Depends only on `biomimicry` (via the
//! `Store` trait); durable backends are wired by the app.

pub mod engineer_calculator;
pub mod registry;

pub use engineer_calculator::{
    CalculatorHandles, SeedOutcome, calculator_dna, calculator_handles, calculator_ready,
    seed_engineer_calculator,
};
pub use registry::{GenomeEntry, default_genomes, find_genome};

#[cfg(test)]
mod tests {
    use super::*;
    use biomimicry::attractor::SettleStatus;
    use biomimicry::genesis::compile;
    use biomimicry::substrate::MemoryStore;

    use crate::engineer_calculator::{
        META_VALUE, OP_ADD, OP_MUL, RESULT, binary_op_signal, readout_value,
    };

    #[test]
    fn domain_isolation_kinds_live_here() {
        // Calculator vocabulary and builders live in this crate; core has no calc types.
        assert_eq!(OP_ADD, "calc.op.add");
        assert_eq!(OP_MUL, "calc.op.mul");
        assert_eq!(RESULT, "calc.result");
        let _ = calculator_dna();
        let _ = calculator_handles;
        let _ = find_genome("engineer_calculator");
    }

    #[test]
    fn registry_lists_engineer_calculator() {
        let entry = find_genome("engineer_calculator").expect("registered");
        assert_eq!(entry.name, "engineer_calculator");
        let dna = (entry.build_dna)();
        assert!(compile(&dna).is_ok());
        let mut store = MemoryStore::new();
        assert!(matches!(
            (entry.seed)(&mut store).unwrap(),
            SeedOutcome::Seeded
        ));
    }

    #[test]
    fn g4_smoke_two_plus_three_reads_five() {
        let mut org = calculator_ready(42);
        org.perturb(binary_op_signal(OP_ADD, 2, 3))
            .expect("perturb");
        let status = org.settle(32).expect("settle");
        assert_eq!(status, SettleStatus::Converged);
        assert_eq!(readout_value(&org), Some(5), "meta={META_VALUE}");
    }

    #[test]
    fn g5_mul_reuses_triad_without_new_families() {
        let handles = calculator_handles();
        // Same family kinds as add — only role / gene id differ.
        assert!(handles
            .genome
            .get(handles.sensor_mul)
            .unwrap()
            .cistron
            .kind
            .as_str()
            .starts_with("sensor_"));
        assert!(handles
            .genome
            .get(handles.reducer_mul)
            .unwrap()
            .cistron
            .kind
            .as_str()
            .starts_with("reducer_"));

        // Seed gene is reducer_add by default; build a mul-seeded organism instead.
        let mut org = biomimicry::organism::OrganismBuilder::new()
            .genome(std::sync::Arc::clone(&handles.genome))
            .seed(7)
            .cadence(biomimicry::metabolism::Cadence::new(2))
            .population_size(2)
            .target_population(2)
            .seed_gene(handles.reducer_mul)
            .without_pop_loop()
            .regulator(biomimicry::expression::NetworkRegulator::new(
                crate::engineer_calculator::calculator_network(&handles),
            ))
            .transducer(crate::engineer_calculator::calculator_transducer(&handles))
            .build()
            .expect("build");
        org.perturb(binary_op_signal(OP_MUL, 2, 3)).expect("perturb");
        assert_eq!(org.settle(32).expect("settle"), SettleStatus::Converged);
        assert_eq!(readout_value(&org), Some(6));
    }
}
