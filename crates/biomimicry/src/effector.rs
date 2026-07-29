//! Effectors — Phase 2 writes that leave the signal stream (the muscle).
//!
//! Declarations and re-exports only — no types or logic live here.
//!
//! An **effector** here always means a write outside the signal stream.
//! Homeostasis's corrective step is a different concept (see that module's docs).

pub mod id;
pub mod memory;
pub mod sink;
pub mod write;

pub use id::*;
pub use memory::*;
pub use sink::*;
pub use write::*;
