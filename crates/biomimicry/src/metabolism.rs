//! The Scheduler — two-phase nested loops, K-ratio, energy budgets (Part III.1).
//!
//! Declarations and re-exports only — no types or logic live here.
//!
//! Under the `determinism` feature (default), the loop is purely queue-driven.
//! "Drained" = both queues empty — not attractor convergence (M5).

pub mod budget;
pub mod cadence;
pub mod debug;
pub mod fixture;
pub mod phase1_queue;
pub mod phase2_queue;
pub mod population;
pub mod reactor;
pub mod scheduler;

#[cfg(test)]
mod props;

pub use budget::*;
pub use cadence::*;
pub use debug::*;
pub use fixture::{seeded_run_ready, sensory_population, systemwide_trigger};
pub use phase1_queue::*;
pub use phase2_queue::*;
pub use population::*;
pub use reactor::*;
pub use scheduler::*;
