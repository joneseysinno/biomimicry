//! Gene and GeneId; complement-gene generation.
//!
//! `Gene ≡ Hyperedge` plus a content-addressed [`GeneId`] and [`GeneOrigin`].
//! Spread / weight are derived annotations and never feed identity.

use super::{Hyperedge, SpatialHypergraph};

/// Content-addressed gene identity: BLAKE3₁₂₈ of the hyperedge's canonical form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeneId(pub u128);

impl GeneId {
    /// Compute the id for a hyperedge's canonical form.
    #[must_use]
    pub fn of(edge: &Hyperedge) -> Self {
        Self(edge.content_id())
    }

    /// Compute the id using a spatial hypergraph (same as [`Self::of`] —
    /// retained for call-site clarity at compile time).
    #[must_use]
    pub fn of_in_graph(edge: &Hyperedge, _graph: &SpatialHypergraph) -> Self {
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
    /// Registered by traversing a DNA hyperedge.
    Traversed,
    /// Genome-level derived gene ensuring complement closure.
    Complement(GeneId),
}

/// A gene: a valid hyperedge in the DNA, addressed by [`GeneId`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gene {
    /// Content-addressed identifier.
    pub id: GeneId,
    /// Underlying hyperedge shape.
    pub hyperedge: Hyperedge,
    /// Traversal vs complement-derived origin.
    pub origin: GeneOrigin,
}

impl Gene {
    /// Build a traversed gene from a validated hyperedge.
    #[must_use]
    pub fn traversed(edge: Hyperedge, graph: &SpatialHypergraph) -> Self {
        let id = GeneId::of_in_graph(&edge, graph);
        Self {
            id,
            hyperedge: edge,
            origin: GeneOrigin::Traversed,
        }
    }

    /// Build a complement-derived gene of `of`.
    #[must_use]
    pub fn complement_of(of: GeneId, edge: Hyperedge, graph: &SpatialHypergraph) -> Self {
        let id = GeneId::of_in_graph(&edge, graph);
        Self {
            id,
            hyperedge: edge,
            origin: GeneOrigin::Complement(of),
        }
    }

    /// Produce the complement gene (polarity-flipped hyperedge, fresh id).
    ///
    /// Origin is set to [`GeneOrigin::Complement`] of `self.id`. The caller
    /// decides whether to register it.
    #[must_use]
    pub fn complement(&self, graph: &SpatialHypergraph) -> Self {
        let edge = self.hyperedge.complement();
        Self::complement_of(self.id, edge, graph)
    }

    /// Whether polarity-flip yields the same canonical id (homeostatically neutral).
    #[must_use]
    pub fn is_self_complement(&self, graph: &SpatialHypergraph) -> bool {
        let flipped = self.hyperedge.complement();
        GeneId::of_in_graph(&flipped, graph) == self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genesis::{DimensionVector, EndpointPolarity, Primitive, PrimitiveNode, endpoint};

    #[test]
    fn complement_involution_on_ids() {
        let mut g = SpatialHypergraph::new();
        let n = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([0]));
        g.add_node(n.clone()).unwrap();
        let edge = Hyperedge::new(
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
