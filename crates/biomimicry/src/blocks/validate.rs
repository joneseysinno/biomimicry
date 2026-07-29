//! Full link-error battery — collect every error, order deterministically.

use std::collections::BTreeSet;

use crate::blocks::block::Block;
use crate::blocks::error::{LinkError, LinkWarning};
use crate::blocks::manifest::Manifest;
use crate::blocks::name::BlockName;
use crate::blocks::namespace::assert_qualification_total;
use crate::blocks::relocate::Relocated;
use crate::blocks::requires::check_requires;
use crate::blocks::resolve::{ResolveResult, resolve};
use crate::genesis::{Grn, validate_cistron};

/// Validation report before merge / after merge.
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    /// All link errors (deterministically ordered).
    pub errors: Vec<LinkError>,
    /// Warnings.
    pub warnings: Vec<LinkWarning>,
}

/// Run pre-merge validation: duplicates, requires, resolve.
pub fn validate_pre_merge(blocks: &[Block], manifest: &Manifest) -> (ResolveResult, ValidationReport) {
    let mut report = ValidationReport::default();

    // Duplicate block names.
    let mut seen = BTreeSet::new();
    for b in blocks {
        if !seen.insert(b.name.clone()) {
            report.errors.push(LinkError::DuplicateBlock {
                name: b.name.clone(),
            });
        }
    }

    report.errors.extend(check_requires(blocks, manifest));
    let resolved = resolve(blocks, manifest);
    report.errors.extend(resolved.errors.clone());
    report.warnings.extend(resolved.warnings.clone());

    sort_errors(&mut report.errors);
    (resolved, report)
}

/// Post-merge checks: cistron validity, qualification totality, orphan wires.
pub fn validate_post_merge(
    relocated: &Relocated,
    resolve: &ResolveResult,
    report: &mut ValidationReport,
) {
    let mut grn = Grn::new();
    for n in &relocated.nodes {
        if let Err(e) = grn.add_node(n.clone()) {
            report.errors.push(LinkError::ManifestParse {
                reason: format!("node merge: {e}"),
            });
        }
    }
    for c in &relocated.cistrons {
        grn.add_cistron(c.clone());
        if let Err(e) = validate_cistron(c, &grn) {
            report.errors.push(LinkError::ManifestParse {
                reason: format!("cistron {}: {e}", c.kind.as_str()),
            });
        }
    }

    if !assert_qualification_total(&relocated.cistrons) {
        report.errors.push(LinkError::ManifestParse {
            reason: "qualification not total after link".into(),
        });
    }

    // P5: every wire end present as a role in the GRN.
    for wire in &resolve.wires {
        let export_q = format!("{}::{}", wire.export_block, wire.export_kind.as_str());
        let import_q = format!("{}::{}", wire.import_block, wire.import_kind.as_str());
        if !role_present(&relocated.cistrons, &export_q) {
            report.errors.push(LinkError::DanglingWire {
                wire: format!("missing export role {export_q}"),
            });
        }
        if !role_present(&relocated.cistrons, &import_q) {
            report.errors.push(LinkError::DanglingWire {
                wire: format!("missing import role {import_q}"),
            });
        }
    }

    sort_errors(&mut report.errors);
}

fn role_present(cistrons: &[crate::genesis::Cistron], role: &str) -> bool {
    cistrons
        .iter()
        .any(|c| c.endpoints.iter().any(|ep| ep.role.as_str() == role))
}

fn sort_errors(errors: &mut [LinkError]) {
    errors.sort_by_key(|e| format!("{e:?}"));
}

/// Whether `name` appears in the manifest pins.
#[must_use]
pub fn manifest_has(manifest: &Manifest, name: &BlockName) -> bool {
    manifest.blocks.iter().any(|p| &p.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::fixture::{ambiguous_manifest, missing_manifest, mistyped_blocks, mistyped_manifest, pipeline_blocks, pipeline_manifest};
    use crate::blocks::error::LinkError;
    use crate::signal::ValueShape;

    #[test]
    fn missing_yields_unsatisfied_import() {
        // Selected set omits sum (as `link` does when the manifest omits it).
        let blocks: Vec<_> = pipeline_blocks()
            .into_iter()
            .filter(|b| b.name.as_str() != "sum")
            .collect();
        let (_res, report) = validate_pre_merge(&blocks, &missing_manifest());
        assert!(
            report.errors.iter().any(|e| matches!(
                e,
                LinkError::UnsatisfiedImport { block, kind, shape }
                if block.as_str() == "scale" && kind.as_str() == "total" && *shape == ValueShape::Int
            )),
            "errors={:?}",
            report.errors
        );
    }

    #[test]
    fn ambiguous_yields_ambiguous_export() {
        let mut blocks = pipeline_blocks();
        // Inject a second total exporter.
        let dup = crate::blocks::fixture::alt_total_block();
        blocks.push(dup);
        let manifest = ambiguous_manifest();
        let (_res, report) = validate_pre_merge(&blocks, &manifest);
        assert!(
            report.errors.iter().any(|e| matches!(e, LinkError::AmbiguousExport { .. })),
            "errors={:?}",
            report.errors
        );
    }

    #[test]
    fn mistyped_yields_shape_mismatch() {
        let blocks = mistyped_blocks();
        let (_res, report) = validate_pre_merge(&blocks, &mistyped_manifest());
        assert!(
            report.errors.iter().any(|e| matches!(e, LinkError::ShapeMismatch { .. })),
            "errors={:?}",
            report.errors
        );
    }

    #[test]
    fn well_formed_has_no_errors() {
        let blocks = pipeline_blocks();
        let (_res, report) = validate_pre_merge(&blocks, &pipeline_manifest());
        assert!(report.errors.is_empty(), "errors={:?}", report.errors);
    }
}
