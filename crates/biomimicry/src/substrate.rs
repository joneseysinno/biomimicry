//! Persistence Boundary — Store trait side only; infinite-db lives in biomimicry-substrate (Part VII).
//!
//! Declarations and re-exports only — no types or logic live here.

pub mod memory;
pub mod snapshot;
pub mod store;

pub use memory::*;
pub use snapshot::*;
pub use store::*;
