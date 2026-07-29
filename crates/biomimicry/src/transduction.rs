//! Transduction Cascades — Phase 2 functions over active genes (Part III.4).
//!
//! Declarations and re-exports only — no types or logic live here.

pub mod arith;
pub mod cascade;
pub mod debug;
pub mod emit;
pub mod fold;
pub mod function;
pub mod map;
pub mod resolve;
pub mod spec;
pub mod transducer;

#[cfg(test)]
mod props;

pub use arith::*;
pub use cascade::*;
pub use debug::*;
pub use emit::*;
pub use fold::*;
pub use function::*;
pub use map::*;
pub use resolve::*;
pub use spec::*;
pub use transducer::*;
