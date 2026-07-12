//! Continuous validation / malformed-state flagging.

use crate::cell::Cell;
use crate::ganglion::Ganglion;

/// Immune-layer finding about organism integrity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImmuneFlag {
    /// Short machine-readable code.
    pub code: String,
    /// Human-readable detail.
    pub detail: String,
}

impl ImmuneFlag {
    fn new(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            detail: detail.into(),
        }
    }
}

/// Scan population + ganglia for malformed state.
#[must_use]
pub fn validate_integrity(population: &[Cell], ganglia: &[Ganglion]) -> Vec<ImmuneFlag> {
    let mut flags = Vec::new();
    for g in ganglia {
        if g.members.len() > g.capacity {
            flags.push(ImmuneFlag::new(
                "over_capacity",
                format!(
                    "ganglion {:?} has {} members > capacity {}",
                    g.handle,
                    g.members.len(),
                    g.capacity
                ),
            ));
        }
        let mut seen = Vec::new();
        for id in &g.members {
            if seen.contains(id) {
                flags.push(ImmuneFlag::new(
                    "duplicate_member",
                    format!("ganglion {:?} duplicates cell {id:?}", g.handle),
                ));
            } else {
                seen.push(*id);
            }
            if population.iter().all(|c| c.id != *id) {
                flags.push(ImmuneFlag::new(
                    "dangling_member",
                    format!("ganglion {:?} references missing cell {id:?}", g.handle),
                ));
            }
        }
    }
    flags
}
