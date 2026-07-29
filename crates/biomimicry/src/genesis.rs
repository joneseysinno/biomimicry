//! DNA — gene regulatory network (GRN) + genome (design Part II.0, II.2).
//!
//! Engine vocabulary is biological ([`Cistron`], [`Grn`]); infinite-db's
//! graph-theoretic `Hyperedge` / `Space` names stay behind `biomimicry-substrate`.
//! Declarations and re-exports only — no types or logic live here.

pub mod cistron;
pub mod compile;
pub mod debug;
pub mod distance;
pub mod endpoint;
pub mod fixture;
pub mod gene;
pub mod genome;
pub mod grn;
pub(crate) mod hash;
pub mod polarity;
pub mod primitive;

#[cfg(test)]
mod props;

pub use cistron::*;
pub use compile::*;
pub use debug::*;
pub use distance::*;
pub use endpoint::*;
pub use fixture::{arith_dna, cascade_dna, toy_dna, with_dangling};
pub use gene::*;
pub use genome::*;
pub use grn::*;
pub use polarity::*;
pub use primitive::*;
