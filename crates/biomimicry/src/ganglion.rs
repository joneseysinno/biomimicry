//! Ganglion — named, bounded cell population tracked as a unit (Part II.4).
//!
//! Declarations and re-exports only — no types or logic live here.

pub mod debug;
pub mod handle;
pub mod health;
pub mod population;
pub mod port;
pub mod stimulate;

pub use debug::*;
pub use handle::*;
pub use health::*;
pub use population::*;
pub use port::*;
pub use stimulate::*;
