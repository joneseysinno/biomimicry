//! M1 integration fixture: toy DNA including `sensory_spike`.

use crate::effector::EffectorId;
use crate::genesis::{
    Cistron, DimensionVector, EndpointPolarity, EndpointRef, Grn, Primitive, PrimitiveNode,
    PrimitiveNodeId, endpoint,
};
use crate::signal::{Scope, SignalKind};
use crate::transduction::{ArithOp, FoldSpec, TransductionFnSpec, TransductionSpec};

/// Build the design's sensory_spike gene plus two small valid genes.
///
/// Endpoints for sensory_spike (declaration order):
/// `Receptor+`, `Expression+`, `Signal+ scope=Systemwide`, `Expression−`, `Transduction−`.
#[must_use]
pub fn toy_dna() -> Grn {
    let mut g = Grn::new();

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

    g.add_cistron(Cistron::new(
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

    g.add_cistron(Cistron::new(
        "local_gate",
        vec![
            endpoint(&r2, EndpointPolarity::Positive, "in", None),
            endpoint(&e2, EndpointPolarity::Positive, "out", None),
        ],
    ));

    g.add_cistron(Cistron::new(
        "homeostatic_neutral",
        vec![
            endpoint(&t_a, EndpointPolarity::Positive, "x", None),
            endpoint(&t_a, EndpointPolarity::Negative, "x", None),
        ],
    ));

    g
}

/// M4 cascade DNA: `cascade_path` has Receptor+/Expression+/Transduction+;
/// `downstream` is a second gene activated by the rule network.
///
/// Endpoints for cascade_path: `Receptor+ trigger`, `Expression+`, `Transduction+`.
#[must_use]
pub fn cascade_dna() -> Grn {
    let mut g = Grn::new();

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

    g.add_cistron(Cistron::new(
        "cascade_path",
        vec![
            endpoint(&receptor, EndpointPolarity::Positive, "trigger", None),
            endpoint(&expr, EndpointPolarity::Positive, "activate", None),
            endpoint(&transduction, EndpointPolarity::Positive, "produce", None),
        ],
    ));

    g.add_cistron(Cistron::new(
        "downstream",
        vec![
            endpoint(&r2, EndpointPolarity::Positive, "gate", None),
            endpoint(&e2, EndpointPolarity::Positive, "out", None),
        ],
    ));

    g
}

/// M11 arith DNA: three cells compute `(a + b) × c` and write through an effector.
///
/// - `sum_cell`: `Receptor+ a|b` → `Fold(Add, Arity(2))` → emit `total` (Cluster)
/// - `scale_cell`: `Receptor+ total|c` → `Arith(Mul)` → emit `scaled` (Cluster)
/// - `sink_cell`: `Receptor+ scaled` → `Effect(result)`
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn arith_dna() -> Grn {
    let mut g = Grn::new();

    let r_a = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 0]));
    let r_b = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 1]));
    let r_total = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 2]));
    let r_c = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 3]));
    let r_scaled = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 4]));
    let e_sum = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 0]));
    let e_scale = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 1]));
    let e_sink = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 2]));
    let t_sum = PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, 0]));
    let t_scale = PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, 1]));
    let t_sink = PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, 2]));
    let s_total = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([4, 0]));
    let s_scaled = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([4, 1]));

    for n in [
        r_a.clone(),
        r_b.clone(),
        r_total.clone(),
        r_c.clone(),
        r_scaled.clone(),
        e_sum.clone(),
        e_scale.clone(),
        e_sink.clone(),
        t_sum.clone(),
        t_scale.clone(),
        t_sink.clone(),
        s_total.clone(),
        s_scaled.clone(),
    ] {
        g.add_node(n).expect("add node");
    }

    let kind_a = SignalKind::qualified("arith", "a");
    let kind_b = SignalKind::qualified("arith", "b");
    let kind_c = SignalKind::qualified("arith", "c");
    let kind_total = SignalKind::qualified("arith", "total");
    let kind_scaled = SignalKind::qualified("arith", "scaled");

    let sum_spec = TransductionSpec::single(
        TransductionFnSpec::fold(
            "sum",
            FoldSpec::arity(ArithOp::Add, 2),
            kind_total.clone(),
        )
        .with_scope(Scope::Cluster),
    );
    let scale_spec = TransductionSpec::single(
        TransductionFnSpec::arith("scale", ArithOp::Mul, kind_scaled.clone())
            .with_scope(Scope::Cluster),
    );
    let sink_spec = TransductionSpec::single(TransductionFnSpec::effect(
        "write_result",
        EffectorId::named("arith.result"),
    ));

    g.add_cistron(
        Cistron::new(
            "sum_cell",
            vec![
                endpoint(&r_a, EndpointPolarity::Positive, kind_a.as_str(), None),
                endpoint(&r_b, EndpointPolarity::Positive, kind_b.as_str(), None),
                endpoint(&e_sum, EndpointPolarity::Positive, "activate", None),
                endpoint(&t_sum, EndpointPolarity::Positive, "produce", None),
                endpoint(
                    &s_total,
                    EndpointPolarity::Positive,
                    kind_total.as_str(),
                    Some(Scope::Cluster),
                ),
            ],
        )
        .with_transduction(sum_spec),
    );

    g.add_cistron(
        Cistron::new(
            "scale_cell",
            vec![
                endpoint(
                    &r_total,
                    EndpointPolarity::Positive,
                    kind_total.as_str(),
                    None,
                ),
                endpoint(&r_c, EndpointPolarity::Positive, kind_c.as_str(), None),
                endpoint(&e_scale, EndpointPolarity::Positive, "activate", None),
                endpoint(&t_scale, EndpointPolarity::Positive, "produce", None),
                endpoint(
                    &s_scaled,
                    EndpointPolarity::Positive,
                    kind_scaled.as_str(),
                    Some(Scope::Cluster),
                ),
            ],
        )
        .with_transduction(scale_spec),
    );

    g.add_cistron(
        Cistron::new(
            "sink_cell",
            vec![
                endpoint(
                    &r_scaled,
                    EndpointPolarity::Positive,
                    kind_scaled.as_str(),
                    None,
                ),
                endpoint(&e_sink, EndpointPolarity::Positive, "activate", None),
                endpoint(&t_sink, EndpointPolarity::Positive, "produce", None),
            ],
        )
        .with_transduction(sink_spec),
    );

    g
}

/// Append a deliberately invalid (dangling) cistron to a clone of `base`.
#[must_use]
pub fn with_dangling(base: &Grn) -> Grn {
    let mut g = base.clone();
    let missing = PrimitiveNodeId(0xDEAD_BEEF_DEAD_BEEF_DEAD_BEEF_DEAD_BEEF);
    g.add_cistron(Cistron::new(
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

        let edges: Vec<_> = dna.iter_cistrons().cloned().collect();
        let spike_id = GeneId::of_in_graph(&edges[0], &dna);
        let gate_id = GeneId::of_in_graph(&edges[1], &dna);
        let neutral_id = GeneId::of_in_graph(&edges[2], &dna);

        assert!(genome.contains(spike_id));
        assert!(genome.contains(gate_id));
        assert!(genome.contains(neutral_id));
        assert_eq!(
            genome.get(spike_id).map(|g| g.cistron.kind.as_str()),
            Some("sensory_spike")
        );
        assert_eq!(
            genome.get(gate_id).map(|g| g.cistron.kind.as_str()),
            Some("local_gate")
        );
        assert_eq!(
            genome.get(neutral_id).map(|g| g.cistron.kind.as_str()),
            Some("homeostatic_neutral")
        );

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
        let loaded = Grn::load(&store).unwrap();
        assert_eq!(loaded, dna);
        let g1 = compile(&dna).unwrap();
        let g2 = compile(&loaded).unwrap();
        assert_eq!(g1.traversed_ids(), g2.traversed_ids());
        assert_eq!(g1.len(), g2.len());
        for gene in g1.iter() {
            assert_eq!(g2.get(gene.id).map(|g| &g.cistron), Some(&gene.cistron));
        }
    }
}
