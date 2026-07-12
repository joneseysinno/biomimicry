//! Signaling Medium — typed pub/sub with relational scoping / diffusion (Part III.2).
//!
//! Declarations and re-exports only — no types or logic live here.

pub mod delivery;
pub mod diffusion;
pub mod scoping;

pub use delivery::*;
pub use diffusion::*;
pub use scoping::*;
