//! Pass B — import→export resolution (infer-when-unambiguous).

use std::collections::BTreeSet;

use crate::blocks::block::Block;
use crate::blocks::error::{LinkError, LinkWarning};
use crate::blocks::manifest::{ExplicitWire, Manifest};
use crate::blocks::name::BlockName;
use crate::blocks::port_spec::{LocalKind, PortSpec};
use crate::signal::ValueShape;

/// A resolved wire from one block's export to another's import.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolvedWire {
    /// Exporting block.
    pub export_block: BlockName,
    /// Export local kind.
    pub export_kind: LocalKind,
    /// Export shape.
    pub export_shape: ValueShape,
    /// Importing block.
    pub import_block: BlockName,
    /// Import local kind.
    pub import_kind: LocalKind,
    /// Import shape.
    pub import_shape: ValueShape,
}

/// Resolution outcome (wires + collected errors/warnings).
#[derive(Debug, Clone, Default)]
pub struct ResolveResult {
    /// Wires to synthesise as bridges.
    pub wires: Vec<ResolvedWire>,
    /// Structural errors.
    pub errors: Vec<LinkError>,
    /// Non-fatal notes.
    pub warnings: Vec<LinkWarning>,
}

/// Resolve imports to exports: explicit wires first, then unambiguous inference.
pub fn resolve(blocks: &[Block], manifest: &Manifest) -> ResolveResult {
    let mut result = ResolveResult::default();
    let mut wired_imports: BTreeSet<(BlockName, LocalKind)> = BTreeSet::new();
    let mut used_exports: BTreeSet<(BlockName, LocalKind)> = BTreeSet::new();

    // Apply explicit wires.
    for wire in &manifest.wires {
        match resolve_explicit(blocks, wire) {
            Ok(rw) => {
                wired_imports.insert((rw.import_block.clone(), rw.import_kind.clone()));
                used_exports.insert((rw.export_block.clone(), rw.export_kind.clone()));
                result.wires.push(rw);
            }
            Err(e) => result.errors.push(e),
        }
    }

    // Infer remaining required (and optional) imports.
    for block in blocks {
        for import in &block.imports {
            let key = (block.name.clone(), import.local_kind.clone());
            if wired_imports.contains(&key) {
                continue;
            }
            let candidates = find_candidates(blocks, &block.name, import);
            match candidates.len() {
                0 => {
                    if import.optional {
                        result.warnings.push(LinkWarning::UnsatisfiedOptionalImport {
                            block: block.name.clone(),
                            kind: import.local_kind.clone(),
                        });
                    } else {
                        result.errors.push(LinkError::UnsatisfiedImport {
                            block: block.name.clone(),
                            kind: import.local_kind.clone(),
                            shape: import.shape.clone(),
                        });
                    }
                }
                1 => {
                    let (export_block, export) = &candidates[0];
                    if export.shape.compatible(&import.shape) {
                        used_exports
                            .insert((export_block.clone(), export.local_kind.clone()));
                        wired_imports.insert(key);
                        result.wires.push(ResolvedWire {
                            export_block: export_block.clone(),
                            export_kind: export.local_kind.clone(),
                            export_shape: export.shape.clone(),
                            import_block: block.name.clone(),
                            import_kind: import.local_kind.clone(),
                            import_shape: import.shape.clone(),
                        });
                    } else {
                        result.errors.push(LinkError::ShapeMismatch {
                            export_block: export_block.clone(),
                            export_kind: export.local_kind.clone(),
                            import_block: block.name.clone(),
                            import_kind: import.local_kind.clone(),
                            expected: import.shape.clone(),
                            got: export.shape.clone(),
                        });
                    }
                }
                _ => {
                    result.errors.push(LinkError::AmbiguousExport {
                        import_block: block.name.clone(),
                        import_kind: import.local_kind.clone(),
                        candidates: candidates
                            .iter()
                            .map(|(b, p)| (b.clone(), p.local_kind.clone()))
                            .collect(),
                    });
                }
            }
        }
    }

    // Unused export warnings.
    for block in blocks {
        for export in &block.exports {
            let key = (block.name.clone(), export.local_kind.clone());
            if !used_exports.contains(&key) {
                result.warnings.push(LinkWarning::UnusedExport {
                    block: block.name.clone(),
                    kind: export.local_kind.clone(),
                });
            }
        }
    }

    // Deterministic wire order.
    result.wires.sort_by(|a, b| {
        (
            &a.export_block,
            &a.export_kind,
            &a.import_block,
            &a.import_kind,
        )
            .cmp(&(
                &b.export_block,
                &b.export_kind,
                &b.import_block,
                &b.import_kind,
            ))
    });
    result
}

#[allow(clippy::result_large_err)]
fn resolve_explicit(blocks: &[Block], wire: &ExplicitWire) -> Result<ResolvedWire, LinkError> {
    let export_block = blocks
        .iter()
        .find(|b| b.name == wire.from_block)
        .ok_or_else(|| LinkError::UnknownBlock {
            name: wire.from_block.clone(),
        })?;
    let import_block = blocks
        .iter()
        .find(|b| b.name == wire.to_block)
        .ok_or_else(|| LinkError::UnknownBlock {
            name: wire.to_block.clone(),
        })?;
    let export = export_block
        .exports
        .iter()
        .find(|p| p.local_kind == wire.from_kind)
        .ok_or_else(|| LinkError::DanglingWire {
            wire: format!(
                "{}::{} → {}::{}",
                wire.from_block, wire.from_kind.as_str(), wire.to_block, wire.to_kind.as_str()
            ),
        })?;
    let import = import_block
        .imports
        .iter()
        .find(|p| p.local_kind == wire.to_kind)
        .ok_or_else(|| LinkError::DanglingWire {
            wire: format!(
                "{}::{} → {}::{}",
                wire.from_block, wire.from_kind.as_str(), wire.to_block, wire.to_kind.as_str()
            ),
        })?;
    if !export.shape.compatible(&import.shape) {
        return Err(LinkError::ShapeMismatch {
            export_block: wire.from_block.clone(),
            export_kind: wire.from_kind.clone(),
            import_block: wire.to_block.clone(),
            import_kind: wire.to_kind.clone(),
            expected: import.shape.clone(),
            got: export.shape.clone(),
        });
    }
    Ok(ResolvedWire {
        export_block: wire.from_block.clone(),
        export_kind: wire.from_kind.clone(),
        export_shape: export.shape.clone(),
        import_block: wire.to_block.clone(),
        import_kind: wire.to_kind.clone(),
        import_shape: import.shape.clone(),
    })
}

/// Exports from other blocks matching `(local_kind, compatible shape)`.
fn find_candidates<'a>(
    blocks: &'a [Block],
    import_block: &BlockName,
    import: &PortSpec,
) -> Vec<(BlockName, &'a PortSpec)> {
    let mut out = Vec::new();
    for block in blocks {
        if &block.name == import_block {
            continue;
        }
        for export in &block.exports {
            if export.local_kind == import.local_kind && export.shape.compatible(&import.shape) {
                out.push((block.name.clone(), export));
            }
        }
    }
    // Also consider shape-mismatched same-name exports so ShapeMismatch can fire
    // when inference would otherwise see zero candidates of compatible shape but
    // a same-kind export exists (mistyped fixture).
    if out.is_empty() {
        for block in blocks {
            if &block.name == import_block {
                continue;
            }
            for export in &block.exports {
                if export.local_kind == import.local_kind {
                    // Same kind, incompatible shape — surface as ShapeMismatch via a
                    // dedicated path: return this single mismatched candidate wrapped
                    // by checking in resolve... Actually resolve only ShapeMismatch
                    // when len==1 and !compatible. So return mismatched same-kind.
                    out.push((block.name.clone(), export));
                }
            }
        }
    }
    out
}
