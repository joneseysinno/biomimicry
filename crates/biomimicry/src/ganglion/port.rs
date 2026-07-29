//! Ganglion ports — derived roles over member receptor / emission surfaces.

use crate::cell::{Cell, CellId};
use crate::ganglion::Ganglion;
use crate::signal::{Scope, SignalKind, ValueShape};

/// Whether a port is an input (receptor role) or output (emission role).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortDirection {
    /// Members whose receptor surface matches `kind`.
    In,
    /// Members whose emission surface carries `kind`.
    Out,
}

/// Declarative, hashable port contract (M12 import/export surface).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GanglionPort {
    /// Signal kind that defines the port role.
    pub kind: SignalKind,
    /// Scope predicate for stimulation / delivery.
    pub scope: Scope,
    /// Input vs output.
    pub direction: PortDirection,
    /// Value shape contract (checked at stimulation).
    pub shape: ValueShape,
}

impl GanglionPort {
    /// Input port builder.
    #[must_use]
    pub fn input(kind: impl Into<SignalKind>, scope: Scope, shape: ValueShape) -> Self {
        Self {
            kind: kind.into(),
            scope,
            direction: PortDirection::In,
            shape,
        }
    }

    /// Output port builder.
    #[must_use]
    pub fn output(kind: impl Into<SignalKind>, scope: Scope, shape: ValueShape) -> Self {
        Self {
            kind: kind.into(),
            scope,
            direction: PortDirection::Out,
            shape,
        }
    }
}

/// Member cells whose receptor surface matches `port.kind` (stable `CellId` order).
#[must_use]
pub fn inputs(ganglion: &Ganglion, population: &[Cell], port: &GanglionPort) -> Vec<CellId> {
    derive_members(ganglion, population, port, PortDirection::In)
}

/// Member cells whose emission surface carries `port.kind` (stable `CellId` order).
#[must_use]
pub fn outputs(ganglion: &Ganglion, population: &[Cell], port: &GanglionPort) -> Vec<CellId> {
    derive_members(ganglion, population, port, PortDirection::Out)
}

fn derive_members(
    ganglion: &Ganglion,
    population: &[Cell],
    port: &GanglionPort,
    direction: PortDirection,
) -> Vec<CellId> {
    let mut ids: Vec<CellId> = ganglion
        .members
        .iter()
        .copied()
        .filter(|id| {
            let Some(cell) = population.iter().find(|c| c.id == *id) else {
                return false;
            };
            let profile = cell.expression.profile();
            match direction {
                PortDirection::In => profile
                    .receptor_surface
                    .iter()
                    .any(|ep| port.kind.matches_role(&ep.role)),
                PortDirection::Out => profile
                    .emission_surface
                    .iter()
                    .any(|ep| port.kind.matches_role(&ep.role)),
            }
        })
        .collect();
    ids.sort();
    ids.dedup();
    ids
}
