//! The Cell — relational automaton: lifecycle, expression, mode, operations (Part II.1, II.3).
//!
//! Declarations and re-exports only — no types or logic live here.

pub mod automaton;
pub mod behavioral_mode;
pub mod debug;
pub mod energy;
pub mod expression_state;
pub mod fixture;
pub mod lifecycle;
pub mod operation;

#[cfg(test)]
mod props;

pub use automaton::*;
pub use behavioral_mode::*;
pub use debug::{lifecycle_dot, profile};
pub use energy::*;
pub use expression_state::*;
pub use fixture::{active_sensory_cell, sensory_genome, trigger_signal};
pub use lifecycle::*;
pub use operation::*;
