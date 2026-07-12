//! Property tests P1–P6 for genesis compilation.

#![cfg(test)]

use std::collections::BTreeSet;

use proptest::prelude::*;

use crate::error::BiomimicryError;
use crate::genesis::fixture::{toy_dna, with_dangling};
use crate::genesis::{
    Cistron, DimensionVector, EndpointPolarity, EndpointRef, GeneId, GeneOrigin, Grn, Primitive,
    PrimitiveNode, compile, endpoint, validate_cistron,
};
use crate::signal::Scope;

/// Small fixed node pool shared by strategies.
fn node_pool() -> Vec<PrimitiveNode> {
    vec![
        PrimitiveNode::new(Primitive::Signal, DimensionVector::new([0, 0])),
        PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([1, 0])),
        PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 0])),
        PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([3, 0])),
        PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 1])),
        PrimitiveNode::new(Primitive::Signal, DimensionVector::new([0, 2])),
    ]
}

fn graph_with_pool() -> Grn {
    let mut g = Grn::new();
    for n in node_pool() {
        g.add_node(n).unwrap();
    }
    g
}

fn arb_polarity() -> impl Strategy<Value = EndpointPolarity> {
    prop_oneof![
        Just(EndpointPolarity::Positive),
        Just(EndpointPolarity::Negative),
    ]
}

fn arb_scope() -> impl Strategy<Value = Option<Scope>> {
    prop_oneof![
        Just(None),
        Just(Some(Scope::SelfCell)),
        Just(Some(Scope::Neighbors)),
        Just(Some(Scope::Cluster)),
        Just(Some(Scope::Systemwide)),
    ]
}

fn arb_endpoint() -> impl Strategy<Value = EndpointRef> {
    let pool = node_pool();
    let n = pool.len();
    (0..n, arb_polarity(), "[a-z]{1,4}", arb_scope())
        .prop_map(move |(i, polarity, role, scope)| endpoint(&pool[i], polarity, &role, scope))
}

fn arb_valid_cistron() -> impl Strategy<Value = Cistron> {
    (
        "[a-z][a-z0-9_]{0,12}",
        proptest::collection::vec(arb_endpoint(), 1..5),
    )
        .prop_filter_map("dedupe endpoints", |(kind, endpoints)| {
            let mut seen = BTreeSet::new();
            let mut unique = Vec::new();
            for ep in endpoints {
                let key = (ep.node, ep.polarity, ep.role.clone(), ep.scope);
                if seen.insert(key) {
                    unique.push(ep);
                }
            }
            if unique.is_empty() || kind.is_empty() {
                None
            } else {
                Some(Cistron::new(kind, unique))
            }
        })
}

fn arb_self_complement_edge() -> impl Strategy<Value = Cistron> {
    let pool = node_pool();
    (0..pool.len(), "[a-z]{1,3}").prop_map(move |(i, role)| {
        let node = &pool[i];
        Cistron::new(
            format!("sc_{role}"),
            vec![
                endpoint(node, EndpointPolarity::Positive, &role, None),
                endpoint(node, EndpointPolarity::Negative, &role, None),
            ],
        )
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// P1 · traversal soundness+completeness
    #[test]
    fn p1_traversal_registers_exactly_valid_cistrons(
        edges in proptest::collection::vec(arb_valid_cistron(), 0..6)
    ) {
        let mut g = graph_with_pool();
        let mut expected = BTreeSet::new();
        for edge in &edges {
            assert!(validate_cistron(edge, &g).is_ok());
            expected.insert(GeneId::of_in_graph(edge, &g));
            g.add_cistron(edge.clone());
        }
        let genome = compile(&g).unwrap();
        let traversed = genome.traversed_ids();
        prop_assert_eq!(traversed, expected);
    }

    /// P2 · involution
    #[test]
    fn p2_complement_involution(edge in arb_valid_cistron()) {
        let g = graph_with_pool();
        let c = edge.complement();
        let cc = c.complement();
        prop_assert_eq!(
            GeneId::of_in_graph(&edge, &g),
            GeneId::of_in_graph(&cc, &g)
        );
    }

    /// P3 · closure
    #[test]
    fn p3_genome_closed_under_complement(
        edges in proptest::collection::vec(arb_valid_cistron(), 1..5)
    ) {
        let mut g = graph_with_pool();
        for edge in edges {
            g.add_cistron(edge);
        }
        let genome = compile(&g).unwrap();
        for gene in genome.iter() {
            let cid = genome.complement_id(gene, &g);
            prop_assert!(genome.contains(cid));
        }
    }

    /// P4 · id determinism / order-independence
    #[test]
    fn p4_declaration_order_independent(
        edge in arb_valid_cistron(),
        seed in any::<u64>()
    ) {
        let g = graph_with_pool();
        let mut shuffled = edge.clone();
        let len = shuffled.endpoints.len();
        if len > 1 {
            let mut i = usize::try_from(seed % (len as u64)).expect("mod len fits usize");
            for k in 0..len {
                let j = (i + k * 3) % len;
                shuffled.endpoints.swap(k, j);
                i = j;
            }
        }
        prop_assert_eq!(
            GeneId::of_in_graph(&edge, &g),
            GeneId::of_in_graph(&shuffled, &g)
        );

        let mut g1 = g.clone();
        g1.add_cistron(edge);
        let mut g2 = g;
        g2.add_cistron(shuffled);
        let a = compile(&g1).unwrap();
        let b = compile(&g2).unwrap();
        prop_assert_eq!(a.traversed_ids(), b.traversed_ids());
    }

    /// P5 · self-complement fixpoint
    #[test]
    fn p5_self_complement_registered_once(edge in arb_self_complement_edge()) {
        let mut g = graph_with_pool();
        g.add_cistron(edge.clone());
        let genome = compile(&g).unwrap();
        let id = GeneId::of_in_graph(&edge, &g);
        prop_assert!(genome.contains(id));
        let gene = genome.get(id).unwrap();
        prop_assert!(gene.is_self_complement(&g));
        prop_assert_eq!(genome.complement_id(gene, &g), id);
        prop_assert_eq!(genome.genes_of_kind(edge.kind.as_str()).count(), 1);
        prop_assert!(matches!(gene.origin, GeneOrigin::Traversed));
    }

    /// P6 · referential integrity
    #[test]
    fn p6_dangling_rejected(_unit in Just(())) {
        let dna = toy_dna();
        let bad = with_dangling(&dna);
        let err = compile(&bad).expect_err("must reject");
        let is_dangling = matches!(err, BiomimicryError::DanglingEndpoint { .. });
        prop_assert!(is_dangling, "expected DanglingEndpoint, got {err:?}");
    }
}
