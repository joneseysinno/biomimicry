//! Content-addressed effector identity.

use blake3::Hasher;

use crate::genesis::hash::{finalize_u128, update_str};

/// Content-addressed effector id (`BLAKE3₁₂₈` of a qualified name).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectorId(pub u128);

impl EffectorId {
    /// Derive from a qualified name (stable across Rust binding renames).
    #[must_use]
    pub fn named(qualified: impl AsRef<str>) -> Self {
        let mut hasher = Hasher::new();
        update_str(&mut hasher, "effector");
        update_str(&mut hasher, qualified.as_ref());
        Self(finalize_u128(&hasher))
    }
}
