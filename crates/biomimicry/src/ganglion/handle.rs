//! Stable lineage handle for targeted observation / control.

/// Stable handle identifying a ganglion lineage across settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GanglionHandle(pub u64);
