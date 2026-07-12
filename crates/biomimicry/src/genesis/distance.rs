//! Structural / semantic / functional distance between primitive nodes.
//!
//! All distances are integer millis. Semantic and functional modes are typed-
//! unavailable stubs until signal samples / learned weights land.

use super::{Cistron, DimensionVector, Grn, PrimitiveNode};
use crate::error::{BiomimicryError, Result};

/// How relational distance is measured for scoping and diffusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DistanceMode {
    /// Topology / coordinate metric over the DNA GRN (default).
    #[default]
    Structural,
    /// Semantic similarity of gene roles / payloads (unavailable in M1).
    Semantic,
    /// Functional similarity of transduction behavior (unavailable in M1).
    Functional,
}

/// Metric over pairs of primitive nodes.
pub trait Distance {
    /// Distance in integer millis between two nodes.
    ///
    /// # Errors
    ///
    /// Returns [`BiomimicryError::DistanceUnavailable`] when the mode cannot
    /// be computed yet, or other genesis errors on bad input.
    fn between(&self, a: &PrimitiveNode, b: &PrimitiveNode) -> Result<i32>;
}

/// Chebyshev (`L∞`) structural distance over [`DimensionVector`] coords.
///
/// Hilbert-curve prefix sharding is a documented seam for M7 (infinite-db
/// indexing optimization): it must not change results, only speed. M1 walks
/// coords directly.
#[derive(Debug, Clone, Copy, Default)]
pub struct StructuralDistance;

impl StructuralDistance {
    /// Chebyshev distance between two coordinate vectors (millis).
    ///
    /// Missing trailing dimensions are treated as `0`.
    #[must_use]
    pub fn chebyshev(a: &DimensionVector, b: &DimensionVector) -> i32 {
        let n = a.len().max(b.len());
        let mut max = 0i32;
        for i in 0..n {
            let av = a.as_slice().get(i).copied().unwrap_or(0);
            let bv = b.as_slice().get(i).copied().unwrap_or(0);
            max = max.max((av - bv).abs());
        }
        max
    }

    /// Manhattan (`L1`) distance between two coordinate vectors (millis).
    #[must_use]
    pub fn manhattan(a: &DimensionVector, b: &DimensionVector) -> i32 {
        let n = a.len().max(b.len());
        let mut sum = 0i32;
        for i in 0..n {
            let av = a.as_slice().get(i).copied().unwrap_or(0);
            let bv = b.as_slice().get(i).copied().unwrap_or(0);
            sum = sum.saturating_add((av - bv).abs());
        }
        sum
    }
}

impl Distance for StructuralDistance {
    fn between(&self, a: &PrimitiveNode, b: &PrimitiveNode) -> Result<i32> {
        Ok(Self::chebyshev(&a.coord, &b.coord))
    }
}

/// Semantic distance — unavailable until signal samples exist.
#[derive(Debug, Clone, Copy, Default)]
pub struct SemanticDistance;

impl Distance for SemanticDistance {
    fn between(&self, _a: &PrimitiveNode, _b: &PrimitiveNode) -> Result<i32> {
        Err(BiomimicryError::DistanceUnavailable {
            mode: DistanceMode::Semantic,
        })
    }
}

/// Functional distance — unavailable until learned `weight_milli` exists.
#[derive(Debug, Clone, Copy, Default)]
pub struct FunctionalDistance;

impl Distance for FunctionalDistance {
    fn between(&self, _a: &PrimitiveNode, _b: &PrimitiveNode) -> Result<i32> {
        Err(BiomimicryError::DistanceUnavailable {
            mode: DistanceMode::Functional,
        })
    }
}

/// Dispatch distance by [`DistanceMode`].
///
/// # Errors
///
/// Propagates mode-specific unavailability or structural errors.
pub fn distance(a: &PrimitiveNode, b: &PrimitiveNode, mode: DistanceMode) -> Result<i32> {
    match mode {
        DistanceMode::Structural => StructuralDistance.between(a, b),
        DistanceMode::Semantic => SemanticDistance.between(a, b),
        DistanceMode::Functional => FunctionalDistance.between(a, b),
    }
}

/// Cistron **spread**: max pairwise structural distance among participating
/// nodes — how "effortful / specialized" the gene is.
///
/// # Errors
///
/// Returns an error if an endpoint's node cannot be resolved.
pub fn spread(edge: &Cistron, graph: &Grn) -> Result<i32> {
    let mut nodes: Vec<&PrimitiveNode> = Vec::with_capacity(edge.endpoints.len());
    for ep in &edge.endpoints {
        let node = graph
            .resolve(ep.node)
            .ok_or(BiomimicryError::DanglingEndpoint { node: ep.node })?;
        nodes.push(node);
    }
    if nodes.len() < 2 {
        return Ok(0);
    }
    let mut max = 0i32;
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let d = StructuralDistance.between(nodes[i], nodes[j])?;
            max = max.max(d);
        }
    }
    Ok(max)
}

/// Seed a gene's initial functional weight from its structural spread.
///
/// Far-flung assemblies get heavier (rarer) weights. Learning overwrites this
/// at M7; M2–M6 get a non-null weight to reason about.
#[must_use]
pub fn weight_from_spread(spread_milli: i32) -> i32 {
    // 1:1 seed — simple, deterministic bridge from structural → functional.
    spread_milli
}

/// Hilbert-index seam (M7): placeholder so call sites can swap in prefix
/// sharding without changing the structural metric's results.
#[derive(Debug, Clone, Copy, Default)]
pub struct HilbertIndexSeam;

impl HilbertIndexSeam {
    /// Documented no-op: returns the same Chebyshev distance as a direct walk.
    #[must_use]
    pub fn structural_distance(&self, a: &PrimitiveNode, b: &PrimitiveNode) -> i32 {
        StructuralDistance::chebyshev(&a.coord, &b.coord)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genesis::{EndpointPolarity, Primitive, endpoint};

    #[test]
    fn structural_distance_symmetric_nonnegative_zero_iff_same() {
        let a = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([0, 0]));
        let b = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([3, -4]));
        let d = StructuralDistance;
        assert_eq!(d.between(&a, &b).unwrap(), 4); // chebyshev
        assert_eq!(d.between(&b, &a).unwrap(), 4);
        assert_eq!(d.between(&a, &a).unwrap(), 0);
        assert!(d.between(&a, &b).unwrap() >= 0);
    }

    #[test]
    fn semantic_and_functional_unavailable() {
        let a = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([0]));
        let b = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([1]));
        assert!(matches!(
            SemanticDistance.between(&a, &b),
            Err(BiomimicryError::DistanceUnavailable {
                mode: DistanceMode::Semantic
            })
        ));
        assert!(matches!(
            FunctionalDistance.between(&a, &b),
            Err(BiomimicryError::DistanceUnavailable {
                mode: DistanceMode::Functional
            })
        ));
    }

    #[test]
    fn spread_is_max_pairwise() {
        let mut g = Grn::new();
        let n0 = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0]));
        let n1 = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([5]));
        let n2 = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([10]));
        g.add_node(n0.clone()).unwrap();
        g.add_node(n1.clone()).unwrap();
        g.add_node(n2.clone()).unwrap();
        let edge = Cistron::new(
            "t",
            vec![
                endpoint(&n0, EndpointPolarity::Positive, "a", None),
                endpoint(&n1, EndpointPolarity::Positive, "b", None),
                endpoint(&n2, EndpointPolarity::Positive, "c", None),
            ],
        );
        assert_eq!(spread(&edge, &g).unwrap(), 10);
    }

    #[test]
    fn hilbert_seam_matches_direct() {
        let a = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([1, 2]));
        let b = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([4, 0]));
        assert_eq!(
            HilbertIndexSeam.structural_distance(&a, &b),
            StructuralDistance::chebyshev(&a.coord, &b.coord)
        );
    }
}
