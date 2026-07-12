//! Gene and GeneId; complement-gene generation.
//!
//! `Gene ≡ Cistron` plus a content-addressed [`GeneId`] and [`GeneOrigin`].
//! Spread / weight are derived annotations and never feed identity.

use super::{Cistron, Grn};

/// Content-addressed gene identity: BLAKE3₁₂₈ of the cistron's canonical form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeneId(pub u128);

impl GeneId {
    /// Compute the id for a cistron's canonical form.
    #[must_use]
    pub fn of(edge: &Cistron) -> Self {
        Self(edge.content_id())
    }

    /// Compute the id using a GRN (same as [`Self::of`] —
    /// retained for call-site clarity at compile time).
    #[must_use]
    pub fn of_in_graph(edge: &Cistron, _graph: &Grn) -> Self {
        Self::of(edge)
    }
}

impl std::fmt::Display for GeneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

/// How a gene entered the genome catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeneOrigin {
    /// Registered by traversing a DNA cistron.
    Traversed,
    /// Genome-level derived gene ensuring complement closure.
    Complement(GeneId),
}

/// A gene: a valid cistron in the DNA, addressed by [`GeneId`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gene {
    /// Content-addressed identifier.
    pub id: GeneId,
    /// Underlying cistron shape.
    pub cistron: Cistron,
    /// Traversal vs complement-derived origin.
    pub origin: GeneOrigin,
}

impl Gene {
    /// Build a traversed gene from a validated cistron.
    #[must_use]
    pub fn traversed(edge: Cistron, graph: &Grn) -> Self {
        let id = GeneId::of_in_graph(&edge, graph);
        Self {
            id,
            cistron: edge,
            origin: GeneOrigin::Traversed,
        }
    }

    /// Build a complement-derived gene of `of`.
    #[must_use]
    pub fn complement_of(of: GeneId, edge: Cistron, graph: &Grn) -> Self {
        let id = GeneId::of_in_graph(&edge, graph);
        Self {
            id,
            cistron: edge,
            origin: GeneOrigin::Complement(of),
        }
    }

    /// Produce the complement gene (polarity-flipped cistron, fresh id).
    ///
    /// Origin is set to [`GeneOrigin::Complement`] of `self.id`. The caller
    /// decides whether to register it.
    #[must_use]
    pub fn complement(&self, graph: &Grn) -> Self {
        let edge = self.cistron.complement();
        Self::complement_of(self.id, edge, graph)
    }

    /// Whether polarity-flip yields the same canonical id (homeostatically neutral).
    #[must_use]
    pub fn is_self_complement(&self, graph: &Grn) -> bool {
        let flipped = self.cistron.complement();
        GeneId::of_in_graph(&flipped, graph) == self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genesis::{DimensionVector, EndpointPolarity, Primitive, PrimitiveNode, endpoint};

    #[test]
    fn complement_involution_on_ids() {
        let mut g = Grn::new();
        let n = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([0]));
        g.add_node(n.clone()).unwrap();
        let edge = Cistron::new(
            "x",
            vec![endpoint(&n, EndpointPolarity::Positive, "e", None)],
        );
        let gene = Gene::traversed(edge, &g);
        let c = gene.complement(&g);
        let cc = c.complement(&g);
        assert_eq!(cc.id, gene.id);
        assert_ne!(c.id, gene.id);
    }
}
