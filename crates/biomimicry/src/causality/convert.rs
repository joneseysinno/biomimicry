//! Bridge [`CausalEventLog`] → [`CausalDag`].

use super::{CausalDag, CausalEdgeKind, CausalEventLog, CausalNode};

/// Convert an ordered event log into a causal DAG.
///
/// Each event becomes one node. When `parent` is set, predecessors are the
/// stamps of **prior** log events whose `child` equals that parent. Converter
/// always emits [`CausalEdgeKind::Single`] (no joint-causation inference).
#[must_use]
pub fn log_to_dag(log: &CausalEventLog) -> CausalDag {
    let events = log.events();
    let mut dag = CausalDag::new();
    for (i, event) in events.iter().enumerate() {
        let mut predecessors = Vec::new();
        if let Some(pid) = event.parent {
            for prior in &events[..i] {
                if prior.child == pid {
                    predecessors.push(prior.stamp);
                }
            }
        }
        dag.append(CausalNode {
            stamp: event.stamp,
            predecessors,
            kind: CausalEdgeKind::Single,
            signal_id: event.child,
            tag: event.tag.clone(),
        });
    }
    dag
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causality::{CausalEvent, CausalStamp};
    use crate::cell::CellId;
    use crate::signal::SignalId;

    #[test]
    fn p1_log_to_dag_deterministic() {
        let mut log = CausalEventLog::new();
        log.push(CausalEvent {
            parent: None,
            child: SignalId(1),
            cell: CellId(1),
            stamp: CausalStamp(1),
            tag: "emit".into(),
        });
        log.push(CausalEvent {
            parent: Some(SignalId(1)),
            child: SignalId(2),
            cell: CellId(2),
            stamp: CausalStamp(2),
            tag: "deliver".into(),
        });
        let a = log_to_dag(&log);
        let b = log_to_dag(&log);
        assert_eq!(a, b);
        assert_eq!(a.len(), 2);
        assert_eq!(a.nodes()[1].predecessors, vec![CausalStamp(1)]);
        assert_eq!(a.nodes()[1].kind, CausalEdgeKind::Single);
    }
}
