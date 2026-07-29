//! Pass A — `requires` dependency graph: cycles fatal, ranges checked.

use std::collections::{BTreeMap, BTreeSet};

use crate::blocks::block::Block;
use crate::blocks::error::LinkError;
use crate::blocks::manifest::{Manifest, pin_map};
use crate::blocks::name::BlockName;

/// Validate the requires graph: unknown blocks, version ranges, cycles.
///
/// Collects every error; does not stop at the first.
pub fn check_requires(blocks: &[Block], manifest: &Manifest) -> Vec<LinkError> {
    let mut errors = Vec::new();
    let pins = pin_map(manifest);
    let by_name: BTreeMap<BlockName, &Block> = blocks.iter().map(|b| (b.name.clone(), b)).collect();

    // Manifest pins must resolve to provided blocks with matching versions.
    for pin in &manifest.blocks {
        match by_name.get(&pin.name) {
            None => errors.push(LinkError::UnknownBlock {
                name: pin.name.clone(),
            }),
            Some(b) if b.version != pin.version => errors.push(LinkError::VersionConflict {
                block: pin.name.clone(),
                required: pin.name.clone(),
                range: crate::blocks::name::VersionRange::parse(&format!("={}", pin.version))
                    .unwrap_or_else(|_| {
                        crate::blocks::name::VersionRange::parse("*").expect("star")
                    }),
                pinned: b.version.clone(),
            }),
            Some(_) => {}
        }
    }

    // Each requires range must be satisfied by the pinned version.
    for block in blocks {
        for req in &block.requires {
            let Some(pinned) = pins.get(&req.name) else {
                errors.push(LinkError::UnknownBlock {
                    name: req.name.clone(),
                });
                continue;
            };
            if !req.range.matches(pinned) {
                errors.push(LinkError::VersionConflict {
                    block: block.name.clone(),
                    required: req.name.clone(),
                    range: req.range.clone(),
                    pinned: pinned.clone(),
                });
            }
            // Dependency must be present in the link set.
            if !by_name.contains_key(&req.name) {
                errors.push(LinkError::UnknownBlock {
                    name: req.name.clone(),
                });
            }
        }
    }

    errors.extend(find_cycles(blocks));
    errors
}

/// DFS cycle detection on the requires graph.
fn find_cycles(blocks: &[Block]) -> Vec<LinkError> {
    let mut adj: BTreeMap<BlockName, Vec<BlockName>> = BTreeMap::new();
    for b in blocks {
        adj.entry(b.name.clone()).or_default();
        for req in &b.requires {
            adj.entry(b.name.clone())
                .or_default()
                .push(req.name.clone());
            adj.entry(req.name.clone()).or_default();
        }
    }

    let mut errors = Vec::new();
    let mut visited = BTreeSet::new();
    let mut stack = Vec::new();
    let mut on_stack = BTreeSet::new();

    for start in adj.keys().cloned().collect::<Vec<_>>() {
        if visited.contains(&start) {
            continue;
        }
        dfs(
            &start,
            &adj,
            &mut visited,
            &mut stack,
            &mut on_stack,
            &mut errors,
        );
    }
    errors
}

fn dfs(
    node: &BlockName,
    adj: &BTreeMap<BlockName, Vec<BlockName>>,
    visited: &mut BTreeSet<BlockName>,
    stack: &mut Vec<BlockName>,
    on_stack: &mut BTreeSet<BlockName>,
    errors: &mut Vec<LinkError>,
) {
    visited.insert(node.clone());
    stack.push(node.clone());
    on_stack.insert(node.clone());

    if let Some(neigh) = adj.get(node) {
        for next in neigh {
            if !visited.contains(next) {
                dfs(next, adj, visited, stack, on_stack, errors);
            } else if on_stack.contains(next) {
                let start = stack.iter().position(|n| n == next).unwrap_or(0);
                let mut cycle: Vec<BlockName> = stack[start..].to_vec();
                cycle.push(next.clone());
                errors.push(LinkError::CyclicRequire { cycle });
            }
        }
    }

    stack.pop();
    on_stack.remove(node);
}
