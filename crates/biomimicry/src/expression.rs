//! Expression Engine — Phase 1 reactive rule network (Part III.3).
//!
//! Declarations and re-exports only — no types or logic live here.

pub mod apply;
pub mod debug;
pub mod fixture;
pub mod network;
pub mod regulator;
pub mod rule;

#[cfg(test)]
mod props;

pub use apply::*;
pub use debug::*;
pub use fixture::*;
pub use network::*;
pub use regulator::*;
pub use rule::*;
