//! Transduction Cascades — Phase 2 functions over active genes (Part III.4).
//!
//! Declarations and re-exports only — no types or logic live here.

pub mod cascade;
pub mod debug;
pub mod emit;
pub mod function;
pub mod transducer;

#[cfg(test)]
mod props;

pub use cascade::*;
pub use debug::*;
pub use emit::*;
pub use function::*;
pub use transducer::*;
