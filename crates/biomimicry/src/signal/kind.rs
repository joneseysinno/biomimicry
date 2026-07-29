//! Signal type (queue routing) vs signal kind (receptor match key).
//!
//! [`SignalType`] chooses Phase 1 vs Phase 2 routing.
//! [`SignalKind`] is the opaque label compared to a receptor's [`crate::genesis::Role`].

use smol_str::SmolStr;

use crate::genesis::Role;

/// Routes a signal onto the Phase 1 (regulatory) or Phase 2 (operational) queue.
///
/// Distinct from [`SignalKind`], which is the receptor-matching label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum SignalType {
    /// Slow path — expression / identity changes (Phase 1).
    #[default]
    Regulatory = 0,
    /// Fast path — transduction / action (Phase 2).
    Operational = 1,
}

/// Receptor-matching label (Role-like interned string).
///
/// A [`crate::genesis::EndpointRef`] with `Receptor+` matches when
/// `receptor.role == signal.kind` (and scopes are compatible).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct SignalKind(pub SmolStr);

impl SignalKind {
    /// Construct from any stringy value.
    #[must_use]
    pub fn new(label: impl AsRef<str>) -> Self {
        Self(SmolStr::new(label.as_ref()))
    }

    /// Qualified kind formatter: `block::local` (M12 namespacing).
    ///
    /// Authors write local names inside a block; the linker rewrites match keys
    /// via this formatter so independently authored fragments cannot collide.
    #[must_use]
    pub fn qualified(block: &str, local: impl AsRef<str>) -> Self {
        Self::new(format!("{}::{}", block, local.as_ref()))
    }

    /// Local suffix after the last `::`, or the whole label if unqualified.
    #[must_use]
    pub fn local_name(&self) -> &str {
        self.as_str()
            .rsplit_once("::")
            .map_or(self.as_str(), |(_, local)| local)
    }

    /// Whether this kind contains a `::` qualification separator.
    #[must_use]
    pub fn is_qualified(&self) -> bool {
        self.as_str().contains("::")
    }

    /// Borrow the kind label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Compare to a genesis [`Role`] (byte-equal labels match).
    #[must_use]
    pub fn matches_role(&self, role: &Role) -> bool {
        self.as_str() == role.as_str()
    }
}

impl From<&str> for SignalKind {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SignalKind {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<Role> for SignalKind {
    fn from(value: Role) -> Self {
        Self::new(value.as_str())
    }
}

impl AsRef<str> for SignalKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
