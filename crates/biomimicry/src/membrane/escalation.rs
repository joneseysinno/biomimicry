//! Reflex (automatic) vs escalation (compute options, route out).

use crate::signal::Signal;

/// How a boundary cell handles an inbound stimulus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResponseMode {
    /// Automatic cascade — no external decision.
    Reflex,
    /// Compute fully-informed options and route them out.
    Escalation,
}

/// Classify whether a stimulus should reflex or escalate.
#[must_use]
pub fn classify(_stimulus: &Signal) -> ResponseMode {
    todo!("classify reflex vs escalation")
}
