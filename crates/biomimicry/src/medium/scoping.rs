//! Relational scope resolution by surface intersection + hop-count.
//!
//! Adjacency is implicit in expression state: an edge exists iff one cell's
//! emission surface matches another's receptor surface for the signal in flight.
//! No `HashMap` iteration — outputs are sorted by [`CellId`].

use crate::cell::{Cell, CellId, LifecycleState};
use crate::error::{BiomimicryError, Result};
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

/// Resolve delivery targets by scope (§2.4).
///
/// | Scope | Reach |
/// |---|---|
/// | `SelfCell` | source only |
/// | `Neighbors` | direct receptor-matchers (exclude source) |
/// | `Systemwide` | every receptor-matching cell |
/// | `Cluster` | [`BiomimicryError::ScopeUnavailable`] until M6 |
///
/// Output is sorted by [`CellId`].
///
/// # Errors
///
/// Returns `ScopeUnavailable` for `Cluster`.
pub fn resolve_targets(source: &Cell, population: &[Cell], sig: &Signal) -> Result<Vec<CellId>> {
    match sig.scope {
        Scope::SelfCell => Ok(vec![source.id]),
        Scope::Cluster => Err(BiomimicryError::ScopeUnavailable {
            scope: Scope::Cluster,
            since_milestone: 6,
        }),
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
) -> Result<Vec<CellId>> {
    resolve_targets(source, population, sig)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::fixture::active_sensory_cell;
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
        let targets = resolve_targets(&cell, std::slice::from_ref(&cell), &sig).unwrap();
        assert_eq!(targets, vec![cell.id]);
    }

    #[test]
    fn cluster_unavailable() {
        let (cell, _) = active_sensory_cell();
        let sig = Signal::new(
            SignalType::Operational,
            "trigger",
            Scope::Cluster,
            Payload::empty(),
            cell.id,
            CausalStamp(0),
        );
        let err = resolve_targets(&cell, std::slice::from_ref(&cell), &sig).unwrap_err();
        assert!(matches!(
            err,
            BiomimicryError::ScopeUnavailable {
                scope: Scope::Cluster,
                since_milestone: 6
            }
        ));
    }
}
