//! The `Signal` event type and content-addressed [`SignalId`].

use blake3::Hasher;

use super::{CausalStamp, Payload, Scope, SignalKind, SignalType};
use crate::cell::CellId;
use crate::genesis::hash::{finalize_u128, update_str};

/// Content-addressed signal identity.
///
/// `id = BLAKE3₁₂₈(kind ‖ scope ‖ payload_digest ‖ source ‖ stamp)`.
/// Including the stamp makes identical emissions at different logical times
/// distinct events (causal-DAG hooks for M7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignalId(pub u128);

impl SignalId {
    /// Compute the id from signal fields.
    #[must_use]
    pub fn of(
        kind: &SignalKind,
        scope: Scope,
        payload: &Payload,
        source: CellId,
        stamp: CausalStamp,
    ) -> Self {
        let mut hasher = Hasher::new();
        update_str(&mut hasher, kind.as_str());
        hasher.update(&[scope.wire_tag()]);
        hasher.update(&payload.digest().to_le_bytes());
        hasher.update(&source.0.to_le_bytes());
        hasher.update(&stamp.0.to_le_bytes());
        Self(finalize_u128(&hasher))
    }
}

impl std::fmt::Display for SignalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

/// A signal event: typed, scoped, stamped, content-addressed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signal {
    /// Content-addressed event id.
    pub id: SignalId,
    /// Regulatory vs operational (queue routing).
    pub ty: SignalType,
    /// Receptor match key.
    pub kind: SignalKind,
    /// Delivery scope.
    pub scope: Scope,
    /// Body + metadata.
    pub payload: Payload,
    /// Emitting cell.
    pub source: CellId,
    /// Causal logical stamp.
    pub stamp: CausalStamp,
}

impl Signal {
    /// Construct a signal; id is derived from the fields.
    #[must_use]
    pub fn new(
        ty: SignalType,
        kind: impl Into<SignalKind>,
        scope: Scope,
        payload: Payload,
        source: CellId,
        stamp: CausalStamp,
    ) -> Self {
        let kind = kind.into();
        let id = SignalId::of(&kind, scope, &payload, source, stamp);
        Self {
            id,
            ty,
            kind,
            scope,
            payload,
            source,
            stamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::Payload;

    #[test]
    fn signal_id_stable_and_stamp_sensitive() {
        let payload = Payload::new(b"hi".as_slice());
        let a = Signal::new(
            SignalType::Regulatory,
            "trigger",
            Scope::Systemwide,
            payload.clone(),
            CellId(1),
            CausalStamp(0),
        );
        let b = Signal::new(
            SignalType::Regulatory,
            "trigger",
            Scope::Systemwide,
            payload.clone(),
            CellId(1),
            CausalStamp(0),
        );
        let c = Signal::new(
            SignalType::Regulatory,
            "trigger",
            Scope::Systemwide,
            payload,
            CellId(1),
            CausalStamp(1),
        );
        assert_eq!(a.id, b.id);
        assert_ne!(a.id, c.id);
    }
}
