//! Causal timestamp handle on a signal (defined in `causality`, used here).

pub use crate::causality::CausalStamp;

/// Stamp attached to a signal for causal ordering.
pub type SignalStamp = CausalStamp;
