//! Continuous validation / malformed-state flagging.

/// Immune-layer finding about organism integrity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImmuneFlag {
    /// Short machine-readable code.
    pub code: String,
    /// Human-readable detail.
    pub detail: String,
}

/// Scan for malformed state and return flags.
#[must_use]
pub fn validate_integrity() -> Vec<ImmuneFlag> {
    todo!("continuous validation / malformed-state flagging")
}
