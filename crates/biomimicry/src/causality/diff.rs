//! Counterfactual DAG comparison.

use std::collections::BTreeSet;

use super::{CausalDag, CausalStamp};

/// Stamp-set difference between two causal DAGs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CausalDiff {
    /// Stamps present only in the first DAG.
    pub only_in_a: usize,
    /// Stamps present only in the second DAG.
    pub only_in_b: usize,
    /// Stamps present in both DAGs.
    pub shared: usize,
}

/// Diff two DAGs by unique stamp sets (enough for counterfactual divergence).
#[must_use]
pub fn diff_dags(a: &CausalDag, b: &CausalDag) -> CausalDiff {
    let set_a: BTreeSet<CausalStamp> = a.nodes().iter().map(|n| n.stamp).collect();
    let set_b: BTreeSet<CausalStamp> = b.nodes().iter().map(|n| n.stamp).collect();
    let shared = set_a.intersection(&set_b).count();
    CausalDiff {
        only_in_a: set_a.len().saturating_sub(shared),
        only_in_b: set_b.len().saturating_sub(shared),
        shared,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causality::{CausalEdgeKind, CausalNode};
    use crate::signal::SignalId;

    #[test]
    fn diff_reports_divergence() {
        let mut a = CausalDag::new();
        a.append(CausalNode::leaf(CausalStamp(1), SignalId(1), "a"));
        a.append(CausalNode {
            stamp: CausalStamp(2),
            predecessors: vec![CausalStamp(1)],
            kind: CausalEdgeKind::Single,
            signal_id: SignalId(2),
            tag: "shared-child".into(),
        });
        let mut b = CausalDag::new();
        b.append(CausalNode::leaf(CausalStamp(1), SignalId(1), "a"));
        b.append(CausalNode {
            stamp: CausalStamp(3),
            predecessors: vec![CausalStamp(1)],
            kind: CausalEdgeKind::Single,
            signal_id: SignalId(3),
            tag: "other".into(),
        });
        let d = diff_dags(&a, &b);
        assert_eq!(d.shared, 1);
        assert_eq!(d.only_in_a, 1);
        assert_eq!(d.only_in_b, 1);
    }
}
