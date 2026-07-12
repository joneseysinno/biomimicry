//! Scheduler phase tag — data-only until metabolism (M3) drains queues.
//!
//! Homed here so the cell operation queue can reference it before `metabolism`
//! exists (M1 `Scope` pattern).

/// Which nested scheduler loop owns an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Phase {
    /// Outer / regulatory loop (expression, lifecycle control).
    Phase1 = 1,
    /// Inner / operational loop (transduction, emission).
    Phase2 = 2,
}
