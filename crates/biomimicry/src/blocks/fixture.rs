//! M12 fixture blocks — sum / scale / sink pipeline (M11 arith vocabulary).

use crate::blocks::block::Block;
use crate::blocks::manifest::{Manifest, Pin};
use crate::blocks::name::Version;
use crate::blocks::port_spec::PortSpec;
use crate::effector::EffectorId;
use crate::genesis::{
    Cistron, DimensionVector, EndpointPolarity, Grn, Primitive, PrimitiveNode, endpoint,
};
use crate::signal::{Scope, SignalKind, Value, ValueShape};
use crate::transduction::{ArithOp, FoldSpec, TransductionFnSpec, TransductionSpec};

/// `sum@1.0.0` — imports `a`,`b`; exports `total`.
#[must_use]
pub fn sum_block() -> Block {
    let r_a = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 0]));
    let r_b = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 1]));
    let e_sum = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 0]));
    let t_sum = PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, 0]));
    let s_total = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([4, 0]));

    let kind_a = SignalKind::new("a");
    let kind_b = SignalKind::new("b");
    let kind_total = SignalKind::new("total");
    let sum_spec = TransductionSpec::single(
        TransductionFnSpec::fold(
            "sum",
            FoldSpec::arity(ArithOp::Add, 2),
            kind_total.clone(),
        )
        .with_scope(Scope::Cluster),
    );
    let cistron = Cistron::new(
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
    .with_transduction(sum_spec);

    Block::new("sum", Version::parse("1.0.0").expect("semver"))
        .with_nodes(vec![r_a, r_b, e_sum, t_sum, s_total])
        .with_cistrons(vec![cistron])
        // `a`/`b` are organism entry ports (stimulated externally) — optional at link.
        .with_imports(vec![
            PortSpec::optional("a", ValueShape::Int, Scope::Cluster),
            PortSpec::optional("b", ValueShape::Int, Scope::Cluster),
        ])
        .with_exports(vec![PortSpec::int("total")])
}

/// `scale@1.0.0` — imports `total`,`factor`; exports `scaled`.
#[must_use]
pub fn scale_block() -> Block {
    let r_total = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 0]));
    let r_factor = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 1]));
    let e_scale = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 0]));
    let t_scale = PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, 0]));
    let s_scaled = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([4, 0]));

    let kind_total = SignalKind::new("total");
    let kind_factor = SignalKind::new("factor");
    let kind_scaled = SignalKind::new("scaled");
    let scale_spec = TransductionSpec::single(
        TransductionFnSpec::arith("scale", ArithOp::Mul, kind_scaled.clone())
            .with_scope(Scope::Cluster),
    );
    let cistron = Cistron::new(
        "scale_cell",
        vec![
            endpoint(
                &r_total,
                EndpointPolarity::Positive,
                kind_total.as_str(),
                None,
            ),
            endpoint(
                &r_factor,
                EndpointPolarity::Positive,
                kind_factor.as_str(),
                None,
            ),
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
    .with_transduction(scale_spec);

    Block::new("scale", Version::parse("1.0.0").expect("semver"))
        .with_nodes(vec![r_total, r_factor, e_scale, t_scale, s_scaled])
        .with_cistrons(vec![cistron])
        .with_imports(vec![
            PortSpec::int("total"),
            // `factor` is an organism entry port — optional at link.
            PortSpec::optional("factor", ValueShape::Int, Scope::Cluster),
        ])
        .with_exports(vec![PortSpec::int("scaled")])
}

/// `sink@1.0.0` — imports `scaled`; writes effector `sink.result`.
#[must_use]
pub fn sink_block() -> Block {
    let r_scaled = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 0]));
    let e_sink = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 0]));
    let t_sink = PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, 0]));

    let kind_scaled = SignalKind::new("scaled");
    let sink_spec = TransductionSpec::single(TransductionFnSpec::effect(
        "write_result",
        EffectorId::named("sink.result"),
    ));
    let cistron = Cistron::new(
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
    .with_transduction(sink_spec);

    Block::new("sink", Version::parse("1.0.0").expect("semver"))
        .with_nodes(vec![r_scaled, e_sink, t_sink])
        .with_cistrons(vec![cistron])
        .with_imports(vec![PortSpec::int("scaled")])
}

/// Alternate block also exporting `total: Int` (ambiguity fixture).
#[must_use]
pub fn alt_total_block() -> Block {
    let r = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 0]));
    let e = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 0]));
    let t = PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, 0]));
    let s = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([4, 0]));
    let kind_total = SignalKind::new("total");
    let spec = TransductionSpec::single(
        TransductionFnSpec::const_value("lit", Value::Int(1), kind_total.clone())
            .with_scope(Scope::Cluster),
    );
    let cistron = Cistron::new(
        "alt_total_cell",
        vec![
            endpoint(&r, EndpointPolarity::Positive, "trigger", None),
            endpoint(&e, EndpointPolarity::Positive, "activate", None),
            endpoint(&t, EndpointPolarity::Positive, "produce", None),
            endpoint(
                &s,
                EndpointPolarity::Positive,
                kind_total.as_str(),
                Some(Scope::Cluster),
            ),
        ],
    )
    .with_transduction(spec);

    Block::new("alt_total", Version::parse("1.0.0").expect("semver"))
        .with_nodes(vec![r, e, t, s])
        .with_cistrons(vec![cistron])
        .with_exports(vec![PortSpec::int("total")])
}

/// Mistyped exporter: `total` as Record instead of Int.
#[must_use]
pub fn mistyped_total_block() -> Block {
    let r = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 0]));
    let e = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 0]));
    let t = PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, 0]));
    let s = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([4, 0]));
    let kind_total = SignalKind::new("total");
    let record = Value::record_from([("x", Value::Int(1))]).expect("record");
    let spec = TransductionSpec::single(
        TransductionFnSpec::const_value("lit", record, kind_total.clone())
            .with_scope(Scope::Cluster),
    );
    let cistron = Cistron::new(
        "mistyped_total_cell",
        vec![
            endpoint(&r, EndpointPolarity::Positive, "trigger", None),
            endpoint(&e, EndpointPolarity::Positive, "activate", None),
            endpoint(&t, EndpointPolarity::Positive, "produce", None),
            endpoint(
                &s,
                EndpointPolarity::Positive,
                kind_total.as_str(),
                Some(Scope::Cluster),
            ),
        ],
    )
    .with_transduction(spec);

    let shape = ValueShape::Record(
        [(smol_str::SmolStr::new("x"), ValueShape::Int)]
            .into_iter()
            .collect(),
    );
    Block::new("mistyped", Version::parse("1.0.0").expect("semver"))
        .with_nodes(vec![r, e, t, s])
        .with_cistrons(vec![cistron])
        .with_exports(vec![PortSpec::required("total", shape, Scope::Cluster)])
}

/// Three-block pipeline.
#[must_use]
pub fn pipeline_blocks() -> Vec<Block> {
    vec![sum_block(), scale_block(), sink_block()]
}

/// Manifest pinning sum/scale/sink @ 1.0.0.
#[must_use]
pub fn pipeline_manifest() -> Manifest {
    Manifest::new().with_blocks(vec![
        Pin::new("sum", Version::parse("1.0.0").expect("semver")),
        Pin::new("scale", Version::parse("1.0.0").expect("semver")),
        Pin::new("sink", Version::parse("1.0.0").expect("semver")),
    ])
}

/// Missing provider: sum omitted so scale's `total` is unsatisfied.
#[must_use]
pub fn missing_manifest() -> Manifest {
    Manifest::new().with_blocks(vec![
        Pin::new("scale", Version::parse("1.0.0").expect("semver")),
        Pin::new("sink", Version::parse("1.0.0").expect("semver")),
    ])
}

/// Ambiguous: two exporters of `total`.
#[must_use]
pub fn ambiguous_manifest() -> Manifest {
    Manifest::new().with_blocks(vec![
        Pin::new("sum", Version::parse("1.0.0").expect("semver")),
        Pin::new("alt_total", Version::parse("1.0.0").expect("semver")),
        Pin::new("scale", Version::parse("1.0.0").expect("semver")),
        Pin::new("sink", Version::parse("1.0.0").expect("semver")),
    ])
}

/// Mistyped: mistyped exporter instead of sum.
#[must_use]
pub fn mistyped_manifest() -> Manifest {
    Manifest::new().with_blocks(vec![
        Pin::new("mistyped", Version::parse("1.0.0").expect("semver")),
        Pin::new("scale", Version::parse("1.0.0").expect("semver")),
        Pin::new("sink", Version::parse("1.0.0").expect("semver")),
    ])
}

/// Blocks for the mistyped manifest.
#[must_use]
pub fn mistyped_blocks() -> Vec<Block> {
    vec![mistyped_total_block(), scale_block(), sink_block()]
}

/// Geometry-reflex smoke block (one AEC concern, not full decomposition).
#[must_use]
pub fn geometry_reflex_block() -> Block {
    let r = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 0]));
    let e = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 0]));
    let t = PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, 0]));
    let s = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([4, 0]));
    let kind_in = SignalKind::new("wall_move");
    let kind_out = SignalKind::new("area_changed");
    let spec = TransductionSpec::single(
        TransductionFnSpec::forward("geometry_forward", kind_out.clone())
            .with_scope(Scope::Cluster),
    );
    let cistron = Cistron::new(
        "geometry_reflex",
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

    Block::new("geometry", Version::parse("0.1.0").expect("semver"))
        .with_nodes(vec![r, e, t, s])
        .with_cistrons(vec![cistron])
        .with_imports(vec![PortSpec::optional(
            "wall_move",
            ValueShape::Int,
            Scope::Cluster,
        )])
        .with_exports(vec![PortSpec::required(
            "area_changed",
            ValueShape::Int,
            Scope::Cluster,
        )])
}

/// Canonical bytes of a linked GRN (nodes + cistron content ids).
#[must_use]
pub fn grn_canonical_bytes(grn: &Grn) -> Vec<u8> {
    use blake3::Hasher;
    use crate::genesis::hash::{finalize_u128, update_u32};
    let mut hasher = Hasher::new();
    let mut node_ids: Vec<_> = grn.nodes().map(|n| n.id.0).collect();
    node_ids.sort_unstable();
    update_u32(
        &mut hasher,
        u32::try_from(node_ids.len()).expect("fits"),
    );
    for id in node_ids {
        hasher.update(&id.to_le_bytes());
    }
    let mut cids: Vec<_> = grn.iter_cistrons().map(Cistron::content_id).collect();
    cids.sort_unstable();
    update_u32(&mut hasher, u32::try_from(cids.len()).expect("fits"));
    for id in cids {
        hasher.update(&id.to_le_bytes());
    }
    finalize_u128(&hasher).to_le_bytes().to_vec()
}

#[cfg(test)]
mod geometry_smoke {
    use super::*;
    use crate::blocks::link;
    use crate::blocks::manifest::{Manifest, Pin};
    use crate::genesis::compile;

    #[test]
    fn geometry_reflex_links_alone() {
        let block = geometry_reflex_block();
        let manifest = Manifest::new().with_blocks(vec![Pin::new(
            "geometry",
            Version::parse("0.1.0").unwrap(),
        )]);
        let linked = link::link(&[block], &manifest).expect("link geometry");
        compile(&linked.grn).expect("compile geometry");
        assert!(
            linked
                .ganglia
                .iter()
                .any(|g| g.name.as_str() == "geometry" && g.input_ports.len() == 1)
        );
    }
}
