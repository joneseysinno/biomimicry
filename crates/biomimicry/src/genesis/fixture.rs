//! M1 integration fixture: toy DNA including `sensory_spike`.

use crate::genesis::{
    DimensionVector, EndpointPolarity, EndpointRef, Hyperedge, Primitive, PrimitiveNode,
    PrimitiveNodeId, SpatialHypergraph, endpoint,
};
use crate::signal::Scope;

/// Build the design's sensory_spike gene plus two small valid genes.
///
/// Endpoints for sensory_spike (declaration order):
/// `Receptor+`, `Expression+`, `Signal+ scope=Systemwide`, `Expression−`, `Transduction−`.
#[must_use]
pub fn toy_dna() -> SpatialHypergraph {
    let mut g = SpatialHypergraph::new();

    let receptor = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 0]));
    let expr = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 0]));
    let signal = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([4, 0]));
    let transduction = PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, 0]));
    let r2 = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 10]));
    let e2 = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([1, 10]));
    let t_a = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([20, 0]));
    let t_b = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([20, 0]));
    assert_eq!(t_a.id, t_b.id);

    for n in [
        receptor.clone(),
        expr.clone(),
        signal.clone(),
        transduction.clone(),
        r2.clone(),
        e2.clone(),
        t_a.clone(),
    ] {
        g.add_node(n).expect("add node");
    }

    g.add_hyperedge(Hyperedge::new(
        "sensory_spike",
        vec![
            endpoint(&receptor, EndpointPolarity::Positive, "trigger", None),
            endpoint(&expr, EndpointPolarity::Positive, "activate", None),
            endpoint(
                &signal,
                EndpointPolarity::Positive,
                "emit",
                Some(Scope::Systemwide),
            ),
            endpoint(&expr, EndpointPolarity::Negative, "suppress", None),
            endpoint(&transduction, EndpointPolarity::Negative, "inhibit", None),
        ],
    ));

    g.add_hyperedge(Hyperedge::new(
        "local_gate",
        vec![
            endpoint(&r2, EndpointPolarity::Positive, "in", None),
            endpoint(&e2, EndpointPolarity::Positive, "out", None),
        ],
    ));

    g.add_hyperedge(Hyperedge::new(
        "homeostatic_neutral",
        vec![
            endpoint(&t_a, EndpointPolarity::Positive, "x", None),
            endpoint(&t_a, EndpointPolarity::Negative, "x", None),
        ],
    ));

    g
}

/// M4 cascade DNA: `cascade_path` has Receptor+/Expression+/Transduction+;
/// `effector` is a second gene activated by the rule network.
///
/// Endpoints for cascade_path: `Receptor+ trigger`, `Expression+`, `Transduction+`.
#[must_use]
pub fn cascade_dna() -> SpatialHypergraph {
    let mut g = SpatialHypergraph::new();

    let receptor = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 0]));
    let expr = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 0]));
    let transduction = PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, 0]));
    let r2 = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 10]));
    let e2 = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([1, 10]));

    for n in [
        receptor.clone(),
        expr.clone(),
        transduction.clone(),
        r2.clone(),
        e2.clone(),
    ] {
        g.add_node(n).expect("add node");
    }

    g.add_hyperedge(Hyperedge::new(
        "cascade_path",
        vec![
            endpoint(&receptor, EndpointPolarity::Positive, "trigger", None),
            endpoint(&expr, EndpointPolarity::Positive, "activate", None),
            endpoint(&transduction, EndpointPolarity::Positive, "produce", None),
        ],
    ));

    g.add_hyperedge(Hyperedge::new(
        "effector",
        vec![
            endpoint(&r2, EndpointPolarity::Positive, "gate", None),
            endpoint(&e2, EndpointPolarity::Positive, "out", None),
        ],
    ));

    g
}

/// Append a deliberately invalid (dangling) hyperedge to a clone of `base`.
#[must_use]
pub fn with_dangling(base: &SpatialHypergraph) -> SpatialHypergraph {
    let mut g = base.clone();
    let missing = PrimitiveNodeId(0xDEAD_BEEF_DEAD_BEEF_DEAD_BEEF_DEAD_BEEF);
    g.add_hyperedge(Hyperedge::new(
        "bogus",
        vec![EndpointRef::new(
            missing,
            Primitive::Signal,
            EndpointPolarity::Positive,
            "ghost",
            None,
        )],
    ));
    g
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::BiomimicryError;
    use crate::genesis::{GeneId, GeneOrigin, compile, to_dot};
    use crate::substrate::MemoryStore;

    #[test]
    fn integration_toy_dna_compiles_with_closure() {
        let dna = toy_dna();
        let genome = compile(&dna).expect("compile toy dna");

        assert_eq!(genome.len(), 5);

        let traversed: Vec<_> = genome
            .iter()
            .filter(|g| matches!(g.origin, GeneOrigin::Traversed))
            .collect();
        assert_eq!(traversed.len(), 3);

        let spike_id = genome
            .genes_of_kind("sensory_spike")
            .next()
            .expect("sensory_spike");
        let gate_id = genome
            .genes_of_kind("local_gate")
            .next()
            .expect("local_gate");
        let neutral_id = genome
            .genes_of_kind("homeostatic_neutral")
            .next()
            .expect("neutral");

        let edges: Vec<_> = dna.iter_hyperedges().cloned().collect();
        assert_eq!(GeneId::of_in_graph(&edges[0], &dna), spike_id);
        assert_eq!(GeneId::of_in_graph(&edges[1], &dna), gate_id);
        assert_eq!(GeneId::of_in_graph(&edges[2], &dna), neutral_id);

        for gene in genome.iter() {
            let cid = genome.complement_id(gene, &dna);
            assert!(genome.contains(cid), "missing complement of {}", gene.id);
        }

        let neutral = genome.get(neutral_id).unwrap();
        assert!(neutral.is_self_complement(&dna));
        assert_eq!(genome.complement_id(neutral, &dna), neutral_id);

        let dot = to_dot(&dna, &genome);
        assert!(dot.contains("sensory_spike"));
        assert!(dot.contains("digraph"));
    }

    #[test]
    fn dangling_endpoint_rejected_genome_unchanged() {
        let dna = toy_dna();
        let bad = with_dangling(&dna);
        let err = compile(&bad).expect_err("dangling must fail");
        assert!(matches!(err, BiomimicryError::DanglingEndpoint { .. }));
    }

    #[test]
    fn store_round_trip_equal_genome() {
        let dna = toy_dna();
        let mut store = MemoryStore::new();
        dna.persist(&mut store).unwrap();
        let loaded = SpatialHypergraph::load(&store).unwrap();
        assert_eq!(loaded, dna);
        let g1 = compile(&dna).unwrap();
        let g2 = compile(&loaded).unwrap();
        assert_eq!(g1.traversed_ids(), g2.traversed_ids());
        assert_eq!(g1.len(), g2.len());
        for gene in g1.iter() {
            assert_eq!(g2.get(gene.id).map(|g| &g.hyperedge), Some(&gene.hyperedge));
        }
    }
}
