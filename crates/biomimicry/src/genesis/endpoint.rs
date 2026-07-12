//! Primitive nodes, coordinates, and cistron endpoint references.
//!
//! [`PrimitiveNodeId`] is the content hash of `(primitive, coord)`. Two nodes
//! with the same primitive type and structural coordinate *are* the same node.

use blake3::Hasher;
use smol_str::SmolStr;

use super::hash::{finalize_u128, update_i32, update_u32};
use super::{EndpointPolarity, Primitive};
use crate::signal::Scope;

/// Logical, relative coordinates in integer millis — no absolute space.
///
/// Everything is relative (resolved design fork); values are fixed-point millis
/// for deterministic, cross-platform hashing and distance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct DimensionVector(pub Vec<i32>);

impl DimensionVector {
    /// Create from millis components.
    #[must_use]
    pub fn new(components: impl Into<Vec<i32>>) -> Self {
        Self(components.into())
    }

    /// Number of dimensions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are no components.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Borrow the millis components.
    #[must_use]
    pub fn as_slice(&self) -> &[i32] {
        &self.0
    }
}

/// Content-addressed identity of a primitive node.
///
/// `id = BLAKE3₁₂₈(primitive.type_id ‖ coord…)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PrimitiveNodeId(pub u128);

impl PrimitiveNodeId {
    /// Compute the content id for a `(primitive, coord)` pair.
    #[must_use]
    pub fn of(primitive: Primitive, coord: &DimensionVector) -> Self {
        let mut hasher = Hasher::new();
        update_u32(&mut hasher, primitive.type_id());
        update_u32(
            &mut hasher,
            u32::try_from(coord.len()).expect("coord length fits u32"),
        );
        for &c in coord.as_slice() {
            update_i32(&mut hasher, c);
        }
        Self(finalize_u128(&hasher))
    }
}

/// A node in the DNA gene regulatory network.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrimitiveNode {
    /// Content-addressed id.
    pub id: PrimitiveNodeId,
    /// Which of the four primitives this node is.
    pub primitive: Primitive,
    /// Relative structural coordinates (millis).
    pub coord: DimensionVector,
}

impl PrimitiveNode {
    /// Construct a node; id is derived from `(primitive, coord)`.
    #[must_use]
    pub fn new(primitive: Primitive, coord: DimensionVector) -> Self {
        let id = PrimitiveNodeId::of(primitive, &coord);
        Self {
            id,
            primitive,
            coord,
        }
    }
}

/// Opaque role label within a gene (data, never behavior).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Role(pub SmolStr);

impl Role {
    /// Intern-friendly constructor from any stringy value.
    #[must_use]
    pub fn new(label: impl AsRef<str>) -> Self {
        Self(SmolStr::new(label.as_ref()))
    }

    /// Borrow the role label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for Role {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for Role {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// One pole of a gene cistron: a node reference plus polarity / role / scope.
///
/// `primitive` is denormalized from the node so runtime matching (M2+) can
/// classify endpoints from `Genome` alone without retaining the grn.
/// It must match the node's primitive; identity hashing still keys on `node`.
///
/// Only [`Primitive::Signal`] endpoints typically populate `scope`; others leave
/// it `None`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EndpointRef {
    /// Content-addressed node this endpoint attaches to.
    pub node: PrimitiveNodeId,
    /// Primitive of the referenced node (denormalized for genome-only queries).
    pub primitive: Primitive,
    /// Endpoint polarity.
    pub polarity: EndpointPolarity,
    /// Role label within the gene.
    pub role: Role,
    /// Optional delivery scope (Signal endpoints).
    pub scope: Option<Scope>,
}

impl EndpointRef {
    /// Construct an endpoint reference.
    #[must_use]
    pub fn new(
        node: PrimitiveNodeId,
        primitive: Primitive,
        polarity: EndpointPolarity,
        role: impl Into<Role>,
        scope: Option<Scope>,
    ) -> Self {
        Self {
            node,
            primitive,
            polarity,
            role: role.into(),
            scope,
        }
    }

    /// Same endpoint with polarity flipped (`+ ↔ −`).
    #[must_use]
    pub fn with_flipped_polarity(&self) -> Self {
        Self {
            node: self.node,
            primitive: self.primitive,
            polarity: self.polarity.flip(),
            role: self.role.clone(),
            scope: self.scope,
        }
    }

    /// Canonical sort key using the denormalized primitive type id.
    #[must_use]
    pub fn sort_key(&self) -> EndpointSortKey<'_> {
        EndpointSortKey {
            primitive_type_id: self.primitive.type_id(),
            node: self.node,
            polarity: self.polarity,
            role: self.role.as_str(),
            scope: self.scope,
        }
    }
}

/// Borrowed key used to canonically order endpoints before hashing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EndpointSortKey<'a> {
    /// Primitive type id of the referenced node.
    pub primitive_type_id: u32,
    /// Node content id.
    pub node: PrimitiveNodeId,
    /// Polarity.
    pub polarity: EndpointPolarity,
    /// Role label.
    pub role: &'a str,
    /// Optional scope.
    pub scope: Option<Scope>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_is_content_addressed() {
        let a = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([0, 0]));
        let b = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([0, 0]));
        let c = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 0]));
        assert_eq!(a.id, b.id);
        assert_ne!(a.id, c.id);
    }
}
