//! Computation as Relaxation — landscape/basin, convergence & divergence (Part IV).
//!
//! Declarations and re-exports only — no types or logic live here.

pub mod basin;
pub mod convergence;
pub mod debug;
pub mod divergence;
pub mod fingerprint;
pub mod landscape;

#[cfg(test)]
mod props;

pub use basin::*;
pub use convergence::*;
pub use debug::*;
pub use divergence::*;
pub use fingerprint::*;
pub use landscape::*;
