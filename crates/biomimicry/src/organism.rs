//! Aggregate root — the handle you instantiate and perturb (no main()).
//!
//! Declarations and re-exports only — no types or logic live here.

pub mod builder;
pub mod fixture;
pub mod membrane;
pub mod persist;
pub mod perturb;
pub mod root;
pub mod settle;

pub use builder::*;
pub use fixture::*;
pub use root::*;
