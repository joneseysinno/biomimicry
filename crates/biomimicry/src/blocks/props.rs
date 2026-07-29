//! Property tests P1–P9 for the blocks linker.

use proptest::prelude::*;

use crate::blocks::fixture::{
    grn_canonical_bytes, pipeline_blocks, pipeline_manifest, scale_block, sum_block,
};
use crate::blocks::link::{self, link_and_compile};
use crate::blocks::manifest::{Manifest, Pin};
use crate::blocks::name::Version;
use crate::blocks::namespace::assert_qualification_total;
use crate::blocks::relocate::BLOCK_PREFIX_STRIDE;
use crate::genesis::{StructuralDistance, compile};
use crate::signal::ValueShape;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// P1 — link determinism / order independence.
    #[test]
    fn p1_order_independence(seed in 0u64..1000) {
        let _ = seed;
        let mut blocks = pipeline_blocks();
        let manifest = pipeline_manifest();
        let (a, _) = link_and_compile(&blocks, &manifest).unwrap();
        blocks.swap(0, 2);
        let (b, _) = link_and_compile(&blocks, &manifest).unwrap();
        assert_eq!(a.genotype, b.genotype);
        assert_eq!(grn_canonical_bytes(&a.grn), grn_canonical_bytes(&b.grn));
    }

    /// P3 — link either yields a compilable GRN or errors; never a bad GRN.
    #[test]
    fn p3_linker_last_line(include_sum in proptest::bool::ANY, include_scale in proptest::bool::ANY) {
        let mut blocks = Vec::new();
        let mut pins = Vec::new();
        if include_sum {
            blocks.push(sum_block());
            pins.push(Pin::new("sum", Version::parse("1.0.0").unwrap()));
        }
        if include_scale {
            blocks.push(scale_block());
            pins.push(Pin::new("scale", Version::parse("1.0.0").unwrap()));
        }
        let manifest = Manifest::new().with_blocks(pins);
        match link::link(&blocks, &manifest) {
            Ok(linked) => {
                assert!(compile(&linked.grn).is_ok(), "linked GRN must compile");
            }
            Err(errors) => {
                assert!(!errors.is_empty());
            }
        }
    }

    /// P4 — qualification totality after successful link.
    #[test]
    fn p4_qualification_total(_x in 0u8..8) {
        let (linked, _) = link_and_compile(&pipeline_blocks(), &pipeline_manifest()).unwrap();
        let cistrons: Vec<_> = linked.grn.iter_cistrons().cloned().collect();
        assert!(assert_qualification_total(&cistrons));
    }

    /// P8 — shape enforcement.
    #[test]
    fn p8_shape_enforcement(compat in proptest::bool::ANY) {
        use crate::blocks::block::Block;
        use crate::blocks::port_spec::PortSpec;
        use crate::signal::Scope;

        let exporter = Block::new("ex", Version::parse("1.0.0").unwrap()).with_exports(vec![
            PortSpec::required("x", ValueShape::Int, Scope::Cluster),
        ]);
        let shape = if compat {
            ValueShape::Int
        } else {
            ValueShape::Text
        };
        let importer = Block::new("im", Version::parse("1.0.0").unwrap()).with_imports(vec![
            PortSpec::required("x", shape, Scope::Cluster),
        ]);
        let manifest = Manifest::new().with_blocks(vec![
            Pin::new("ex", Version::parse("1.0.0").unwrap()),
            Pin::new("im", Version::parse("1.0.0").unwrap()),
        ]);
        let result = link::link(&[exporter, importer], &manifest);
        if compat {
            assert!(result.is_ok(), "{result:?}");
        } else {
            assert!(result.is_err());
        }
    }

    /// P9 — BlockId invariant under linkage.
    #[test]
    fn p9_identity_stability(_x in 0u8..4) {
        let sum = sum_block();
        let id_alone = sum.id();
        let blocks = pipeline_blocks();
        let _ = link::link(&blocks, &pipeline_manifest()).unwrap();
        assert_eq!(sum.id(), id_alone);
        assert_eq!(blocks[0].id(), id_alone);
    }
}

#[test]
fn p2_relocation_disjoint_and_local() {
    let (linked, _) = link_and_compile(&pipeline_blocks(), &pipeline_manifest()).unwrap();
    let nodes: Vec<_> = linked.grn.nodes().cloned().collect();
    // Disjoint prefixes: first coord component differs across blocks' stride buckets.
    let mut prefixes = std::collections::BTreeSet::new();
    for n in &nodes {
        if let Some(&p) = n.coord.as_slice().first() {
            prefixes.insert(p / BLOCK_PREFIX_STRIDE);
        }
    }
    assert!(prefixes.len() >= 3, "prefixes={prefixes:?}");

    // Locality: mean within-block distance < mean cross-block (by prefix bucket).
    let mut within = Vec::new();
    let mut across = Vec::new();
    for (i, a) in nodes.iter().enumerate() {
        for b in nodes.iter().skip(i + 1) {
            let pa = a.coord.as_slice().first().copied().unwrap_or(0) / BLOCK_PREFIX_STRIDE;
            let pb = b.coord.as_slice().first().copied().unwrap_or(0) / BLOCK_PREFIX_STRIDE;
            let d = StructuralDistance::manhattan(&a.coord, &b.coord);
            if pa == pb {
                within.push(d);
            } else {
                across.push(d);
            }
        }
    }
    if !within.is_empty() && !across.is_empty() {
        let mean_w = f64::from(within.iter().sum::<i32>()) / within.len() as f64;
        let mean_a = f64::from(across.iter().sum::<i32>()) / across.len() as f64;
        assert!(mean_w < mean_a, "within={mean_w} across={mean_a}");
    }
}

#[test]
fn p5_no_silent_orphans() {
    let (linked, _) = link_and_compile(&pipeline_blocks(), &pipeline_manifest()).unwrap();
    for wire in &linked.wires {
        let export_q = format!("{}::{}", wire.export_block, wire.export_kind.as_str());
        let import_q = format!("{}::{}", wire.import_block, wire.import_kind.as_str());
        assert!(
            linked
                .grn
                .iter_cistrons()
                .any(|c| c.endpoints.iter().any(|e| e.role.as_str() == export_q))
        );
        assert!(
            linked
                .grn
                .iter_cistrons()
                .any(|c| c.endpoints.iter().any(|e| e.role.as_str() == import_q))
        );
    }
}

#[test]
fn p6_cycle_discrimination() {
    use crate::blocks::block::Block;
    use crate::blocks::name::{BlockReq, VersionRange};

    // Signal-graph cycle (feedback) is legal — not a CyclicRequire.
    let manifest = Manifest::new().with_blocks(vec![
        Pin::new("a", Version::parse("1.0.0").unwrap()),
        Pin::new("b", Version::parse("1.0.0").unwrap()),
    ]);
    let a = sum_like_export("a", "x", "y");
    let b = sum_like_export("b", "y", "x");
    let result = link::link(&[a, b], &manifest);
    // Feedback topologies must not produce CyclicRequire.
    if let Err(errors) = &result {
        assert!(
            !errors
                .iter()
                .any(|e| matches!(e, crate::blocks::error::LinkError::CyclicRequire { .. })),
            "{errors:?}"
        );
    }

    // Requires cycle is fatal.
    let mut c1 = Block::new("c1", Version::parse("1.0.0").unwrap());
    c1.requires.push(BlockReq::new(
        "c2",
        VersionRange::parse("^1").unwrap(),
    ));
    let mut c2 = Block::new("c2", Version::parse("1.0.0").unwrap());
    c2.requires.push(BlockReq::new(
        "c1",
        VersionRange::parse("^1").unwrap(),
    ));
    let manifest = Manifest::new().with_blocks(vec![
        Pin::new("c1", Version::parse("1.0.0").unwrap()),
        Pin::new("c2", Version::parse("1.0.0").unwrap()),
    ]);
    let err = link::link(&[c1, c2], &manifest).expect_err("requires cycle");
    assert!(
        err.iter()
            .any(|e| matches!(e, crate::blocks::error::LinkError::CyclicRequire { .. })),
        "{err:?}"
    );
}

fn sum_like_export(name: &str, export: &str, import: &str) -> crate::blocks::block::Block {
    use crate::blocks::port_spec::PortSpec;
    use crate::genesis::{
        Cistron, DimensionVector, EndpointPolarity, Primitive, PrimitiveNode, endpoint,
    };
    use crate::signal::{Scope, SignalKind};
    use crate::transduction::{TransductionFnSpec, TransductionSpec};

    let r = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 0]));
    let e = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 0]));
    let t = PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, 0]));
    let s = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([4, 0]));
    let kind_in = SignalKind::new(import);
    let kind_out = SignalKind::new(export);
    let spec = TransductionSpec::single(
        TransductionFnSpec::forward("fwd", kind_out.clone()).with_scope(Scope::Cluster),
    );
    let cistron = Cistron::new(
        format!("{name}_cell"),
        vec![
            endpoint(&r, EndpointPolarity::Positive, kind_in.as_str(), None),
            endpoint(&e, EndpointPolarity::Positive, "activate", None),
            endpoint(&t, EndpointPolarity::Positive, "produce", None),
            endpoint(
                &s,
                EndpointPolarity::Positive,
                kind_out.as_str(),
                Some(Scope::Cluster),
            ),
        ],
    )
    .with_transduction(spec);
    crate::blocks::block::Block::new(name, Version::parse("1.0.0").unwrap())
        .with_nodes(vec![r, e, t, s])
        .with_cistrons(vec![cistron])
        .with_imports(vec![PortSpec::int(import)])
        .with_exports(vec![PortSpec::int(export)])
}

#[test]
fn p7_bridge_transparency_identity() {
    // Bridges forward payload — Forward step preserves value (unit-level).
    use crate::signal::{Payload, Signal, SignalKind, SignalType, Scope, CausalStamp};
    use crate::transduction::{TransductionFnSpec, TransductionKind};
    use crate::cell::CellId;

    let step = TransductionFnSpec::forward("b", SignalKind::new("out")).with_scope(Scope::Neighbors);
    assert!(matches!(step.kind, TransductionKind::Forward));
    let fn_ = crate::transduction::fn_from_spec(&step);
    let input = Signal::new(
        SignalType::Operational,
        SignalKind::new("in"),
        Scope::Cluster,
        Payload::of(crate::signal::Value::Int(42)),
        CellId(1),
        CausalStamp(0),
    );
    let out = fn_.call(&input).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].payload.value().unwrap(), crate::signal::Value::Int(42));
    assert_eq!(out[0].kind.as_str(), "out");
}
