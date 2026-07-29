//! Deterministic disjoint dimension-prefix allocation by [`BlockId`].

use crate::blocks::block::Block;
use crate::blocks::bridge::BridgeFragment;
use crate::blocks::name::BlockId;
use crate::genesis::{Cistron, DimensionVector, EndpointRef, PrimitiveNode, Role};

/// Prefix stride between blocks in the first dimension (millis).
pub const BLOCK_PREFIX_STRIDE: i32 = 1_000_000;

/// Relocated DNA ready to merge into one [`crate::genesis::Grn`].
#[derive(Debug, Clone)]
pub struct Relocated {
    /// All nodes with global coordinates.
    pub nodes: Vec<PrimitiveNode>,
    /// All cistrons with endpoint node ids rewritten.
    pub cistrons: Vec<Cistron>,
}

/// Relocate blocks (ordered by [`BlockId`]) and bridges into one address space.
///
/// Blocks become tissue regions: same-block cells share a dimension prefix.
#[must_use]
pub fn relocate(blocks: &[Block], bridges: &[BridgeFragment]) -> Relocated {
    let mut ordered: Vec<(BlockId, &Block)> = blocks.iter().map(|b| (b.id(), b)).collect();
    ordered.sort_by(|a, b| a.0.cmp(&b.0));

    let mut nodes = Vec::new();
    let mut cistrons = Vec::new();

    for (index, (_id, block)) in ordered.iter().enumerate() {
        let prefix = i32::try_from(index).unwrap_or(0).saturating_mul(BLOCK_PREFIX_STRIDE);
        let (n, c) = relocate_fragment(&block.nodes, &block.cistrons, prefix);
        nodes.extend(n);
        cistrons.extend(c);
    }

    // Bridges get prefixes after all blocks.
    let bridge_base = i32::try_from(ordered.len())
        .unwrap_or(0)
        .saturating_mul(BLOCK_PREFIX_STRIDE);
    for (i, bridge) in bridges.iter().enumerate() {
        let prefix = bridge_base
            + i32::try_from(i).unwrap_or(0).saturating_mul(BLOCK_PREFIX_STRIDE / 100);
        let (n, c) = relocate_fragment(&bridge.nodes, std::slice::from_ref(&bridge.cistron), prefix);
        nodes.extend(n);
        cistrons.extend(c);
    }

    Relocated { nodes, cistrons }
}

fn relocate_fragment(
    nodes: &[PrimitiveNode],
    cistrons: &[Cistron],
    prefix: i32,
) -> (Vec<PrimitiveNode>, Vec<Cistron>) {
    let mut id_map = std::collections::BTreeMap::new();
    let mut new_nodes = Vec::with_capacity(nodes.len());
    for n in nodes {
        let mut coords = vec![prefix];
        coords.extend_from_slice(n.coord.as_slice());
        let relocated = PrimitiveNode::new(n.primitive, DimensionVector::new(coords));
        id_map.insert(n.id, relocated.id);
        new_nodes.push(relocated);
    }

    let new_cistrons = cistrons
        .iter()
        .map(|c| {
            let endpoints = c
                .endpoints
                .iter()
                .map(|ep| {
                    let new_id = id_map.get(&ep.node).copied().unwrap_or(ep.node);
                    EndpointRef::new(
                        new_id,
                        ep.primitive,
                        ep.polarity,
                        Role::new(ep.role.as_str()),
                        ep.scope,
                    )
                })
                .collect();
            let mut out = Cistron::new(c.kind.as_str(), endpoints)
                .with_directionality(c.directionality);
            if let Some(w) = c.weight_milli {
                out = out.with_weight_milli(w);
            }
            if let Some(spec) = &c.transduction {
                out = out.with_transduction(spec.clone());
            }
            out
        })
        .collect();

    (new_nodes, new_cistrons)
}
