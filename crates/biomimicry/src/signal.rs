//! Signals — regulatory/operational kinds, scopes, payloads, causal stamp (Part II.5).
//!
//! Declarations and re-exports only — no types or logic live here.

pub mod event;
pub mod kind;
pub mod payload;
pub mod phase;
pub mod scope;
pub mod stamp;

pub use event::*;
pub use kind::*;
pub use payload::*;
pub use phase::*;
pub use scope::{Scope, SignalScope, scope_compatible};
pub use stamp::*;
