//! Behavioral modes of a cell (Part II.6 Layer 2).
//!
//! [`BehavioralMode::Differentiating`] overlaps in *name* with
//! [`crate::cell::LifecycleState::Differentiating`] but is a distinct axis:
//! lifecycle is identity/fate; mode is scheduler posture. They co-occur but
//! must not be treated as the same type.

/// Coarse behavioral posture of a cell during scheduling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BehavioralMode {
    /// Waiting for signals.
    #[default]
    Idle,
    /// Changing expression identity (Phase 1 work).
    ///
    /// Not the same type as [`crate::cell::LifecycleState::Differentiating`].
    Differentiating,
    /// Slow mitotic / population growth path.
    DividingSlow,
}
