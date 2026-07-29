//! Bridge cistron synthesis — interneurons between qualified ports.

use crate::blocks::name::BlockName;
use crate::blocks::port_spec::LocalKind;
use crate::blocks::resolve::ResolvedWire;
use crate::genesis::{
    Cistron, DimensionVector, EndpointPolarity, Primitive, PrimitiveNode, endpoint,
};
use crate::signal::{Scope, SignalKind};
use crate::transduction::{TransductionFnSpec, TransductionSpec};

/// Provenance for a synthesised bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeInfo {
    /// Cistron kind label.
    pub kind: String,
    /// Exporting block.
    pub export_block: BlockName,
    /// Export local kind.
    pub export_kind: LocalKind,
    /// Importing block.
    pub import_block: BlockName,
    /// Import local kind.
    pub import_kind: LocalKind,
}

/// One bridge's DNA fragment (nodes + cistron) before relocation merge.
#[derive(Debug, Clone)]
pub struct BridgeFragment {
    /// Provenance.
    pub info: BridgeInfo,
    /// Local nodes (coords temporary; relocated later).
    pub nodes: Vec<PrimitiveNode>,
    /// The bridge cistron.
    pub cistron: Cistron,
}

/// Synthesise a bridge for every resolved wire.
///
/// Each bridge: `Receptor+(export::kind) → Forward → Signal+(import::kind, Neighbors)`.
/// Neighbors scope crosses per-block ganglion boundaries; Cluster would not.
#[must_use]
pub fn synthesise_bridges(wires: &[ResolvedWire]) -> Vec<BridgeFragment> {
    wires
        .iter()
        .enumerate()
        .map(|(i, w)| synthesise_one(i, w))
        .collect()
}

fn synthesise_one(index: usize, wire: &ResolvedWire) -> BridgeFragment {
    let export_q = SignalKind::qualified(wire.export_block.as_str(), wire.export_kind.as_str());
    let import_q = SignalKind::qualified(wire.import_block.as_str(), wire.import_kind.as_str());
    let kind = format!(
        "bridge::{}::{}→{}::{}",
        wire.export_block,
        wire.export_kind.as_str(),
        wire.import_block,
        wire.import_kind.as_str()
    );

    // Temporary coords — relocate assigns the real prefix later.
    let base = i32::try_from(index).unwrap_or(0).saturating_mul(10);
    let receptor = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, base]));
    let expr = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, base]));
    let transduction =
        PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, base]));
    let signal = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([4, base]));

    let spec = TransductionSpec::single(
        TransductionFnSpec::forward("bridge_forward", import_q.clone())
            .with_scope(Scope::Neighbors),
    );

    let cistron = Cistron::new(
        kind.clone(),
        vec![
            endpoint(
                &receptor,
                EndpointPolarity::Positive,
                export_q.as_str(),
                None,
            ),
            endpoint(&expr, EndpointPolarity::Positive, "activate", None),
            endpoint(&transduction, EndpointPolarity::Positive, "produce", None),
            endpoint(
                &signal,
                EndpointPolarity::Positive,
                import_q.as_str(),
                Some(Scope::Neighbors),
            ),
        ],
    )
    .with_transduction(spec);

    BridgeFragment {
        info: BridgeInfo {
            kind,
            export_block: wire.export_block.clone(),
            export_kind: wire.export_kind.clone(),
            import_block: wire.import_block.clone(),
            import_kind: wire.import_kind.clone(),
        },
        nodes: vec![receptor, expr, transduction, signal],
        cistron,
    }
}
