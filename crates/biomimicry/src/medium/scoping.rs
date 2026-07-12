//! Relational scope resolution by surface intersection + hop-count.
//!
//! Adjacency is implicit in expression state: an edge exists iff one cell's
//! emission surface matches another's receptor surface for the signal in flight.
//! No `HashMap` iteration — outputs are sorted by [`CellId`].

use crate::cell::{Cell, CellId, LifecycleState};
use crate::error::Result;
use crate::ganglion::Ganglion;
use crate::signal::{Scope, Signal, scope_compatible};

/// Whether `target` would receive `sig` (receptor match, not vetoed, not Dead).
#[must_use]
pub fn receptor_accepts(target: &Cell, sig: &Signal) -> bool {
    if target.lifecycle() == LifecycleState::Dead {
        return false;
    }
    !target.expression.match_receptors(sig).matched.is_empty()
}

/// Whether `cell`'s emission surface is compatible with `sig`.
#[must_use]
pub fn can_emit(cell: &Cell, sig: &Signal) -> bool {
    cell.expression
        .profile()
        .emission_surface
        .iter()
        .any(|ep| sig.kind.matches_role(&ep.role) && scope_compatible(ep.scope, sig.scope))
}

/// Directed adjacency for `sig`: source can emit it and target would receive it.
#[must_use]
pub fn adjacency(source: &Cell, target: &Cell, sig: &Signal) -> bool {
    source.id != target.id && can_emit(source, sig) && receptor_accepts(target, sig)
}

/// Resolve Cluster targets: co-members of the source cell's first ganglion.
#[must_use]
pub fn cluster_targets(source: CellId, ganglia: &[Ganglion], population: &[Cell]) -> Vec<CellId> {
    let Some(g) = ganglia.iter().find(|g| g.contains(source)) else {
        return Vec::new();
    };
    let mut ids: Vec<CellId> = g
        .members
        .iter()
        .copied()
        .filter(|id| *id != source)
        .filter(|id| {
            population
                .iter()
                .find(|c| c.id == *id)
                .is_some_and(|c| c.lifecycle() != LifecycleState::Dead)
        })
        .collect();
    ids.sort();
    ids
}

/// Resolve delivery targets by scope (§2.4).
///
/// | Scope | Reach |
/// |---|---|
/// | `SelfCell` | source only |
/// | `Neighbors` | direct receptor-matchers (exclude source) |
/// | `Systemwide` | every receptor-matching cell |
/// | `Cluster` | other living members of the source's ganglion |
///
/// Output is sorted by [`CellId`].
///
/// # Errors
///
/// Infallible in M6 (Cluster no longer returns `ScopeUnavailable`).
pub fn resolve_targets(
    source: &Cell,
    population: &[Cell],
    sig: &Signal,
    ganglia: &[Ganglion],
) -> Result<Vec<CellId>> {
    match sig.scope {
        Scope::SelfCell => Ok(vec![source.id]),
        Scope::Cluster => Ok(cluster_targets(source.id, ganglia, population)),
        Scope::Neighbors => {
            let mut ids: Vec<CellId> = population
                .iter()
                .filter(|t| t.id != source.id && receptor_accepts(t, sig))
                .map(|t| t.id)
                .collect();
            ids.sort();
            Ok(ids)
        }
        Scope::Systemwide => {
            let mut ids: Vec<CellId> = population
                .iter()
                .filter(|t| receptor_accepts(t, sig))
                .map(|t| t.id)
                .collect();
            ids.sort();
            Ok(ids)
        }
    }
}

/// Brute-force scan equivalent of [`resolve_targets`] (for P6).
///
/// # Errors
///
/// Same as [`resolve_targets`].
pub fn resolve_targets_bruteforce(
    source: &Cell,
    population: &[Cell],
    sig: &Signal,
    ganglia: &[Ganglion],
) -> Result<Vec<CellId>> {
    resolve_targets(source, population, sig, ganglia)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::fixture::active_sensory_cell;
    use crate::ganglion::GanglionHandle;
    use crate::signal::{CausalStamp, Payload, SignalType};

    #[test]
    fn self_cell_only_source() {
        let (cell, _) = active_sensory_cell();
        let sig = Signal::new(
            SignalType::Operational,
            "trigger",
            Scope::SelfCell,
            Payload::empty(),
            cell.id,
            CausalStamp(0),
        );
        let targets = resolve_targets(&cell, std::slice::from_ref(&cell), &sig, &[]).unwrap();
        assert_eq!(targets, vec![cell.id]);
    }

    #[test]
    fn cluster_empty_without_ganglion() {
        let (cell, _) = active_sensory_cell();
        let sig = Signal::new(
            SignalType::Operational,
            "trigger",
            Scope::Cluster,
            Payload::empty(),
            cell.id,
            CausalStamp(0),
        );
        let targets = resolve_targets(&cell, std::slice::from_ref(&cell), &sig, &[]).unwrap();
        assert!(targets.is_empty());
    }

    #[test]
    fn cluster_delivers_to_co_members() {
        let (a, _) = active_sensory_cell();
        let genome = a.genome.clone();
        let mut b = crate::cell::Cell::new(CellId(2), genome);
        b.try_transition(crate::cell::LifecycleState::Differentiating)
            .unwrap();
        b.try_transition(crate::cell::LifecycleState::Active)
            .unwrap();
        let mut g = Ganglion::new(GanglionHandle(1), "circuit", 4);
        g.try_add(a.id);
        g.try_add(b.id);
        let sig = Signal::new(
            SignalType::Operational,
            "trigger",
            Scope::Cluster,
            Payload::empty(),
            a.id,
            CausalStamp(0),
        );
        let pop = [a.clone(), b];
        let targets = resolve_targets(&a, &pop, &sig, std::slice::from_ref(&g)).unwrap();
        assert_eq!(targets, vec![CellId(2)]);
    }
}
