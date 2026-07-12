//! Relational delivery scopes.
//!
//! Biological ↔ engine mapping (locked in M2):
//! - `autocrine` → [`Scope::SelfCell`]
//! - `paracrine` / `juxtacrine` → [`Scope::Neighbors`]
//! - **Coupled cluster** (design Part II.5) → [`Scope::Cluster`]
//! - `endocrine` → [`Scope::Systemwide`]
//!
//! `SelfCell` is named that way because `Self` is a Rust keyword — do not
//! rename the variant to `Self`.

/// How far a signal may diffuse through the relational medium.
///
/// Also available as [`SignalScope`] for call sites that prefer the signal-
/// domain name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
#[repr(u8)]
pub enum Scope {
    /// Only the emitting cell (`autocrine`).
    ///
    /// Named `SelfCell` because `Self` is a Rust keyword.
    #[default]
    SelfCell = 1,
    /// Immediate relational neighbors (`paracrine` / `juxtacrine`).
    Neighbors = 2,
    /// Coupled cluster / ganglion neighborhood (design: "Coupled cluster").
    Cluster = 3,
    /// Entire organism (`endocrine`).
    Systemwide = 4,
}

/// Alias matching the signal-module vocabulary.
pub type SignalScope = Scope;

impl Scope {
    /// Discriminant used in content-hash serialization (`1..=4`).
    #[must_use]
    pub const fn wire_tag(self) -> u8 {
        self as u8
    }

    /// Reconstruct from a wire tag.
    #[must_use]
    pub const fn from_wire_tag(tag: u8) -> Option<Self> {
        match tag {
            1 => Some(Self::SelfCell),
            2 => Some(Self::Neighbors),
            3 => Some(Self::Cluster),
            4 => Some(Self::Systemwide),
            _ => None,
        }
    }

    /// Breadth order: `SelfCell` < `Neighbors` < `Cluster` < `Systemwide`.
    #[must_use]
    pub const fn breadth(self) -> u8 {
        self as u8
    }
}

/// Whether a receptor's optional scope is compatible with an incoming signal scope.
///
/// - `None` on the receptor = matches any signal scope.
/// - `Some(r)` matches when the signal is at least as broad as `r` (exact or
///   broader / "washes over"), i.e. `signal.breadth() >= r.breadth()`.
#[must_use]
pub const fn scope_compatible(receptor: Option<Scope>, signal: Scope) -> bool {
    match receptor {
        None => true,
        Some(r) => signal.breadth() >= r.breadth(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_receptor_matches_all() {
        for s in [
            Scope::SelfCell,
            Scope::Neighbors,
            Scope::Cluster,
            Scope::Systemwide,
        ] {
            assert!(scope_compatible(None, s));
        }
    }

    #[test]
    fn broader_signal_matches_narrower_receptor() {
        assert!(scope_compatible(Some(Scope::Neighbors), Scope::Neighbors));
        assert!(scope_compatible(Some(Scope::Neighbors), Scope::Systemwide));
        assert!(!scope_compatible(Some(Scope::Neighbors), Scope::SelfCell));
        assert!(scope_compatible(Some(Scope::Systemwide), Scope::Systemwide));
        assert!(!scope_compatible(Some(Scope::Systemwide), Scope::Cluster));
    }
}
