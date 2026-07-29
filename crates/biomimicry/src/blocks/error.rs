//! Link-time errors and warnings — all collected, never fail-fast.

use crate::blocks::name::{BlockName, Version, VersionRange};
use crate::blocks::port_spec::LocalKind;
use crate::signal::ValueShape;

/// One structural failure discovered while linking.
///
/// Large variants are intentional (carry [`ValueShape`] for precise diagnostics).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkError {
    /// A required import has no matching export.
    UnsatisfiedImport {
        /// Block that declared the import.
        block: BlockName,
        /// Local kind.
        kind: LocalKind,
        /// Expected shape.
        shape: ValueShape,
    },
    /// Two or more exports match one import; manifest must name the wire.
    AmbiguousExport {
        /// Importing block.
        import_block: BlockName,
        /// Import local kind.
        import_kind: LocalKind,
        /// Candidate `(export_block, export_kind)` pairs.
        candidates: Vec<(BlockName, LocalKind)>,
    },
    /// Explicit or inferred wire joins incompatible shapes.
    ShapeMismatch {
        /// Exporting block.
        export_block: BlockName,
        /// Export local kind.
        export_kind: LocalKind,
        /// Importing block.
        import_block: BlockName,
        /// Import local kind.
        import_kind: LocalKind,
        /// Shape promised by the export.
        expected: ValueShape,
        /// Shape required by the import.
        got: ValueShape,
    },
    /// Two blocks share the same name in the link set.
    DuplicateBlock {
        /// Duplicated name.
        name: BlockName,
    },
    /// Pinned version does not satisfy a `requires` range.
    VersionConflict {
        /// Block that declared the requirement.
        block: BlockName,
        /// Required dependency name.
        required: BlockName,
        /// Range that was required.
        range: VersionRange,
        /// Version pinned in the manifest.
        pinned: Version,
    },
    /// Cycle in the `requires` dependency graph.
    CyclicRequire {
        /// Block names in cycle order.
        cycle: Vec<BlockName>,
    },
    /// Explicit wire names an endpoint that does not exist.
    DanglingWire {
        /// Wire description.
        wire: String,
    },
    /// Manifest names a block not present in the link set.
    UnknownBlock {
        /// Missing name.
        name: BlockName,
    },
    /// Manifest would require a version solver (typed deferral).
    VersionSolveUnavailable {
        /// Milestone that will implement solving.
        since_milestone: u32,
    },
    /// Manifest parse / serialise failure.
    ManifestParse {
        /// Human-readable reason.
        reason: String,
    },
}

/// Non-fatal composition notes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkWarning {
    /// An export is never consumed.
    UnusedExport {
        /// Exporting block.
        block: BlockName,
        /// Local kind.
        kind: LocalKind,
    },
    /// An optional import was left unwired.
    UnsatisfiedOptionalImport {
        /// Importing block.
        block: BlockName,
        /// Local kind.
        kind: LocalKind,
    },
    /// A tissue / gene cannot reach an effector or output port.
    UnreachableTissue {
        /// Cistron kind or tissue label.
        label: String,
    },
}
