//! Interface Model — boundary cells, breadth/depth scaling, reflex vs escalation (Part V).
//!
//! Declarations and re-exports only — no types or logic live here.

pub mod boundary_cell;
pub mod escalation;
pub mod fixture;
pub mod scaling;

pub use boundary_cell::*;
pub use escalation::*;
pub use fixture::*;
pub use scaling::*;
