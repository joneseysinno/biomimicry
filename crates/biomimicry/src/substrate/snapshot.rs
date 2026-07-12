//! Snapshot / branch abstractions for replay and counterfactual debugging.

/// Identifier of a persisted snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SnapshotId(pub u64);

/// Identifier of a branched counterfactual timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct BranchId(pub u64);

/// Metadata about a snapshot.
#[derive(Debug, Clone)]
pub struct SnapshotMeta {
    /// Snapshot id.
    pub id: SnapshotId,
    /// Optional parent branch.
    pub branch: BranchId,
    /// Human label.
    pub label: String,
}
