//! DNA — spatial hypergraph + genome (design Part II.0, II.2).
//!
//! Declarations and re-exports only — no types or logic live here.

pub mod compile;
pub mod debug;
pub mod distance;
pub mod endpoint;
pub mod fixture;
pub mod gene;
pub mod genome;
pub(crate) mod hash;
pub mod hyperedge;
pub mod hypergraph;
pub mod polarity;
pub mod primitive;

#[cfg(test)]
mod props;

pub use compile::*;
pub use debug::*;
pub use distance::*;
pub use endpoint::*;
pub use fixture::{cascade_dna, toy_dna, with_dangling};
pub use gene::*;
pub use genome::*;
pub use hyperedge::*;
pub use hypergraph::*;
pub use polarity::*;
pub use primitive::*;
