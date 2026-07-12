//! Endpoint polarity (+/−) and pole semantics per primitive.
//!
//! Substrate Tail/Head/Neutral mapping is deferred to M7 — polarity here is the
//! engine-facing +/− property on hyperedge endpoints.

use super::Primitive;

/// Polarity of a hyperedge endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum EndpointPolarity {
    /// Positive pole (`+`).
    Positive = 0,
    /// Negative pole (`−`).
    Negative = 1,
}

impl EndpointPolarity {
    /// Flip `+ ↔ −`.
    #[must_use]
    pub const fn flip(self) -> Self {
        match self {
            Self::Positive => Self::Negative,
            Self::Negative => Self::Positive,
        }
    }

    /// Alias for [`Self::flip`] — polarity inversion is an involution.
    #[must_use]
    pub const fn complement(self) -> Self {
        self.flip()
    }
}

/// Semantic meaning of a `(Primitive, EndpointPolarity)` pole.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoleSemantic {
    /// Signal+: emit outward, with scope.
    Emit,
    /// Signal−: absorb / cancel before propagation.
    Absorb,
    /// Receptor+: open — match an incoming signal.
    Open,
    /// Receptor−: blocked — mismatch a specific signal.
    Blocked,
    /// Expression+: activate gene(s).
    Activate,
    /// Expression−: suppress gene(s).
    Suppress,
    /// Transduction+: produce output.
    Produce,
    /// Transduction−: inhibit / reverse output.
    Inhibit,
}

/// Look up the pole-semantics table for a primitive × polarity pair.
#[must_use]
pub const fn pole_semantics(primitive: Primitive, polarity: EndpointPolarity) -> PoleSemantic {
    match (primitive, polarity) {
        (Primitive::Signal, EndpointPolarity::Positive) => PoleSemantic::Emit,
        (Primitive::Signal, EndpointPolarity::Negative) => PoleSemantic::Absorb,
        (Primitive::Receptor, EndpointPolarity::Positive) => PoleSemantic::Open,
        (Primitive::Receptor, EndpointPolarity::Negative) => PoleSemantic::Blocked,
        (Primitive::Expression, EndpointPolarity::Positive) => PoleSemantic::Activate,
        (Primitive::Expression, EndpointPolarity::Negative) => PoleSemantic::Suppress,
        (Primitive::Transduction, EndpointPolarity::Positive) => PoleSemantic::Produce,
        (Primitive::Transduction, EndpointPolarity::Negative) => PoleSemantic::Inhibit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_is_involution() {
        assert_eq!(
            EndpointPolarity::Positive.flip().flip(),
            EndpointPolarity::Positive
        );
        assert_eq!(
            EndpointPolarity::Negative.flip().flip(),
            EndpointPolarity::Negative
        );
    }

    #[test]
    fn pole_semantics_table() {
        assert_eq!(
            pole_semantics(Primitive::Signal, EndpointPolarity::Positive),
            PoleSemantic::Emit
        );
        assert_eq!(
            pole_semantics(Primitive::Signal, EndpointPolarity::Negative),
            PoleSemantic::Absorb
        );
        assert_eq!(
            pole_semantics(Primitive::Receptor, EndpointPolarity::Positive),
            PoleSemantic::Open
        );
        assert_eq!(
            pole_semantics(Primitive::Receptor, EndpointPolarity::Negative),
            PoleSemantic::Blocked
        );
        assert_eq!(
            pole_semantics(Primitive::Expression, EndpointPolarity::Positive),
            PoleSemantic::Activate
        );
        assert_eq!(
            pole_semantics(Primitive::Expression, EndpointPolarity::Negative),
            PoleSemantic::Suppress
        );
        assert_eq!(
            pole_semantics(Primitive::Transduction, EndpointPolarity::Positive),
            PoleSemantic::Produce
        );
        assert_eq!(
            pole_semantics(Primitive::Transduction, EndpointPolarity::Negative),
            PoleSemantic::Inhibit
        );
    }
}
