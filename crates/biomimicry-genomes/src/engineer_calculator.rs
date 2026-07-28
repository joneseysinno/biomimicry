//! Engineering calculator default genome.
//!
//! Sensor → Gate → Reducer triad per operator, plus Bound / Boundary families.

mod dna;
mod kernels;
mod kinds;
mod seed;
mod wiring;

pub use dna::calculator_dna;
pub use kernels::{
    kernel_add, kernel_compare, kernel_mul, kernel_negate, kernel_reciprocal,
};
pub use kinds::*;
pub use seed::{
    SeedOutcome, is_engineer_calculator_seeded, seed_engineer_calculator,
    seed_engineer_calculator_dyn,
};
pub use wiring::{
    CalculatorHandles, binary_op_signal, calculator_handles, calculator_network,
    calculator_ready, calculator_transducer, readout_value,
};
