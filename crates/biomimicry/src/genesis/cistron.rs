//! Cistron — the unit of regulatory function (a "gene's shape").
//!
//! A cistron binds a set of primitive participations (receptor / expression /
//! signal / transduction, each at a polarity) into one self-contained
//! regulatory behaviour. It is the engine's biological name for what the
//! persistence layer stores as an infinite-db `Hyperedge`; the two never mix in
//! engine code — translation happens once, in `biomimicry-substrate`.
//!
//! Identity is the cistron's canonical content hash ([`crate::genesis::GeneId`]);
//! `weight_milli` / spread are annotations and never feed identity.

use blake3::Hasher;

use super::hash::{finalize_u128, update_str, update_u32};
use super::{EndpointPolarity, EndpointRef, Role};
use crate::signal::Scope;

/// Gene / cistron kind label (e.g. `"sensory_spike"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CistronKind(pub String);

impl CistronKind {
    /// Construct from any stringy value.
    #[must_use]
    pub fn new(kind: impl Into<String>) -> Self {
        Self(kind.into())
    }

    /// Borrow the kind string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Whether the kind is empty (invalid for compilation).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<&str> for CistronKind {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for CistronKind {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Whether a cistron is directed (genes default to directed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
#[repr(u8)]
pub enum Directionality {
    /// Directed cistron (default for genes).
    #[default]
    Directed = 0,
    /// Undirected cistron.
    Undirected = 1,
}

/// A cistron connecting multiple endpoints (a gene's topology).
///
/// Declaration order in `endpoints` is preserved for display; identity uses
/// [`Self::canonical_endpoints`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cistron {
    /// Kind label (must be non-empty to compile).
    pub kind: CistronKind,
    /// Endpoints in declaration order.
    pub endpoints: Vec<EndpointRef>,
    /// Optional functional weight in millis; seeded from spread when absent.
    pub weight_milli: Option<i32>,
    /// Directed vs undirected.
    pub directionality: Directionality,
}

impl Cistron {
    /// Create a directed cistron with no weight yet.
    #[must_use]
    pub fn new(kind: impl Into<CistronKind>, endpoints: Vec<EndpointRef>) -> Self {
        Self {
            kind: kind.into(),
            endpoints,
            weight_milli: None,
            directionality: Directionality::Directed,
        }
    }

    /// Builder: set weight.
    #[must_use]
    pub fn with_weight_milli(mut self, weight_milli: i32) -> Self {
        self.weight_milli = Some(weight_milli);
        self
    }

    /// Builder: set directionality.
    #[must_use]
    pub fn with_directionality(mut self, directionality: Directionality) -> Self {
        self.directionality = directionality;
        self
    }

    /// Endpoints sorted into the pinned canonical order.
    ///
    /// Sort key: `(primitive_type_id, node_id, polarity, role, scope)`.
    #[must_use]
    pub fn canonical_endpoints(&self) -> Vec<EndpointRef> {
        let mut eps = self.endpoints.clone();
        eps.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        eps
    }

    /// Content-addressed identity of this cistron's canonical form.
    ///
    /// Hash domain: `(kind, directionality, sorted endpoints)` where each
    /// endpoint contributes `(PrimitiveNodeId, polarity, role, scope)`.
    /// Spread / weight never participate.
    #[must_use]
    pub fn content_id(&self) -> u128 {
        let canonical = self.canonical_endpoints();
        let mut hasher = Hasher::new();
        update_str(&mut hasher, self.kind.as_str());
        hasher.update(&[self.directionality as u8]);
        update_u32(
            &mut hasher,
            u32::try_from(canonical.len()).expect("endpoint count fits u32"),
        );
        for ep in &canonical {
            hasher.update(&ep.node.0.to_le_bytes());
            hasher.update(&[ep.polarity as u8]);
            update_str(&mut hasher, ep.role.as_str());
            match ep.scope {
                None => hasher.update(&[0u8]),
                Some(scope) => hasher.update(&[scope.wire_tag()]),
            };
        }
        finalize_u128(&hasher)
    }

    /// Complement cistron: every endpoint polarity flipped.
    ///
    /// Kind, directionality, roles, scopes, and declaration-relative structure
    /// are preserved; weight is cleared (re-derived at compile from spread).
    #[must_use]
    pub fn complement(&self) -> Self {
        Self {
            kind: self.kind.clone(),
            endpoints: self
                .endpoints
                .iter()
                .map(EndpointRef::with_flipped_polarity)
                .collect(),
            weight_milli: None,
            directionality: self.directionality,
        }
    }

    /// Whether endpoints are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.endpoints.is_empty()
    }
}

/// Helper for tests / fixtures: build an endpoint from a node.
#[must_use]
pub fn endpoint(
    node: &super::PrimitiveNode,
    polarity: EndpointPolarity,
    role: &str,
    scope: Option<Scope>,
) -> EndpointRef {
    EndpointRef::new(node.id, node.primitive, polarity, Role::new(role), scope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genesis::{DimensionVector, Primitive, PrimitiveNode};

    #[test]
    fn complement_flips_all_polarities() {
        let n = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([1]));
        let edge = Cistron::new(
            "pair",
            vec![
                endpoint(&n, EndpointPolarity::Positive, "a", None),
                endpoint(&n, EndpointPolarity::Negative, "b", None),
            ],
        );
        let c = edge.complement();
        assert_eq!(c.endpoints[0].polarity, EndpointPolarity::Negative);
        assert_eq!(c.endpoints[1].polarity, EndpointPolarity::Positive);
    }

    #[test]
    fn declaration_order_does_not_change_content_id() {
        let a = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0]));
        let b = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([1]));
        let e1 = Cistron::new(
            "g",
            vec![
                endpoint(&a, EndpointPolarity::Positive, "r", None),
                endpoint(&b, EndpointPolarity::Positive, "s", Some(Scope::Systemwide)),
            ],
        );
        let e2 = Cistron::new(
            "g",
            vec![
                endpoint(&b, EndpointPolarity::Positive, "s", Some(Scope::Systemwide)),
                endpoint(&a, EndpointPolarity::Positive, "r", None),
            ],
        );
        assert_eq!(e1.content_id(), e2.content_id());
    }
}
