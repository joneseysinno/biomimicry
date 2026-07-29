//! Link driver — qualify → requires → resolve → bridge → relocate → merge → validate.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::blocks::block::Block;
use crate::blocks::bridge::{BridgeInfo, synthesise_bridges};
use crate::blocks::error::{LinkError, LinkWarning};
use crate::blocks::ganglion_template::{GanglionTemplate, templates_from_blocks};
use crate::blocks::manifest::{Manifest, OrganismGenotype};
use crate::blocks::name::{BlockId, BlockName};
use crate::blocks::namespace::{apply_renames, qualify_block, renames_for};
use crate::blocks::relocate::{Relocated, relocate};
use crate::blocks::resolve::ResolvedWire;
use crate::blocks::validate::{validate_post_merge, validate_pre_merge};
use crate::genesis::Grn;

/// Counts successful `compile` attempts after link (tests assert failures never reach compile).
static COMPILE_REACHED: AtomicUsize = AtomicUsize::new(0);

/// Reset the compile-reached counter (test helper).
pub fn reset_compile_counter() {
    COMPILE_REACHED.store(0, Ordering::SeqCst);
}

/// How many times [`note_compile_reached`] was called since the last reset.
#[must_use]
pub fn compile_reached_count() -> usize {
    COMPILE_REACHED.load(Ordering::SeqCst)
}

/// Mark that `compile` was invoked on a linked GRN (call from tests / wrappers).
pub fn note_compile_reached() {
    COMPILE_REACHED.fetch_add(1, Ordering::SeqCst);
}

/// Successful link product — one GRN indistinguishable from hand-authored DNA.
#[derive(Debug, Clone)]
pub struct Linked {
    /// Merged gene regulatory network.
    pub grn: Grn,
    /// Content-addressed organism identity.
    pub genotype: OrganismGenotype,
    /// Per-block ganglion templates.
    pub ganglia: Vec<GanglionTemplate>,
    /// Non-fatal composition notes.
    pub warnings: Vec<LinkWarning>,
    /// Resolved wires (for DOT / bridge provenance).
    pub wires: Vec<ResolvedWire>,
    /// Bridge provenance.
    pub bridges: Vec<BridgeInfo>,
    /// Block ids in link order (sorted by id).
    pub block_ids: Vec<BlockId>,
    /// Cistron kinds belonging to each block (pre-bridge).
    pub block_cistrons: BTreeMap<BlockName, Vec<String>>,
}

/// Link blocks under a manifest into one GRN (or every error).
///
/// # Errors
///
/// Returns a deterministically ordered [`LinkError`] list. Never fails on the first alone.
pub fn link(blocks: &[Block], manifest: &Manifest) -> Result<Linked, Vec<LinkError>> {
    // Select blocks matching manifest pins (by name); verify versions in requires pass.
    let mut selected: Vec<Block> = Vec::new();
    let mut errors = Vec::new();
    let by_name: BTreeMap<BlockName, &Block> = {
        let mut m = BTreeMap::new();
        for b in blocks {
            if m.insert(b.name.clone(), b).is_some() {
                errors.push(LinkError::DuplicateBlock {
                    name: b.name.clone(),
                });
            }
        }
        m
    };

    for pin in &manifest.blocks {
        match by_name.get(&pin.name) {
            Some(b) => selected.push((*b).clone()),
            None => errors.push(LinkError::UnknownBlock {
                name: pin.name.clone(),
            }),
        }
    }

    if !errors.is_empty() {
        errors.sort_by_key(|e| format!("{e:?}"));
        // Still try to validate what we have for fuller diagnostics.
    }

    // Apply renames then qualify.
    for block in &mut selected {
        let renames = renames_for(&block.name, &manifest.renames);
        apply_renames(block, &renames);
    }
    let qualified: Vec<Block> = selected.iter().map(qualify_block).collect();

    // Record block cistron kinds (from original, before qualify changes kinds only in roles).
    let mut block_cistrons = BTreeMap::new();
    for b in &selected {
        block_cistrons.insert(
            b.name.clone(),
            b.cistrons.iter().map(|c| c.kind.as_str().to_owned()).collect(),
        );
    }

    // Pre-merge validate on qualified blocks (ports still local; resolve uses local kinds).
    // Resolve matches on local_kind — use selected (unqualified ports) for resolve,
    // but DNA must be the qualified copies for merge.
    let (resolved, mut report) = validate_pre_merge(&selected, manifest);
    report.errors.extend(errors);

    if !report.errors.is_empty() {
        report.errors.sort_by_key(|e| format!("{e:?}"));
        return Err(report.errors);
    }

    let bridges = synthesise_bridges(&resolved.wires);
    let bridge_infos: Vec<BridgeInfo> = bridges.iter().map(|b| b.info.clone()).collect();

    // Relocate qualified DNA + bridges.
    let relocated: Relocated = relocate(&qualified, &bridges);
    validate_post_merge(&relocated, &resolved, &mut report);

    if !report.errors.is_empty() {
        report.errors.sort_by_key(|e| format!("{e:?}"));
        return Err(report.errors);
    }

    let mut grn = Grn::new();
    for n in &relocated.nodes {
        grn.add_node(n.clone()).map_err(|e| {
            vec![LinkError::ManifestParse {
                reason: format!("merge node: {e}"),
            }]
        })?;
    }
    for c in relocated.cistrons {
        grn.add_cistron(c);
    }

    let mut block_ids: Vec<BlockId> = selected.iter().map(Block::id).collect();
    block_ids.sort();
    let genotype = OrganismGenotype::of(&block_ids, manifest);
    let capacity = manifest.ganglion_capacity.unwrap_or(8);
    let ganglia = templates_from_blocks(&selected, capacity);

    Ok(Linked {
        grn,
        genotype,
        ganglia,
        warnings: report.warnings,
        wires: resolved.wires,
        bridges: bridge_infos,
        block_ids,
        block_cistrons,
    })
}

/// Link then compile — notes compile-reached for A2 assertions.
///
/// # Errors
///
/// Link errors, or a single-element vec wrapping compile failure as [`LinkError::ManifestParse`].
pub fn link_and_compile(
    blocks: &[Block],
    manifest: &Manifest,
) -> Result<(Linked, std::sync::Arc<crate::genesis::Genome>), Vec<LinkError>> {
    let linked = link(blocks, manifest)?;
    note_compile_reached();
    let genome = crate::genesis::compile(&linked.grn).map_err(|e| {
        vec![LinkError::ManifestParse {
            reason: format!("compile after link: {e}"),
        }]
    })?;
    Ok((linked, genome))
}
