//! Organism membrane surface — ingress, boundary attach, scaling, escalation inbox.

use crate::cell::{CellId, LifecycleState};
use crate::error::{BiomimicryError, Result};
use crate::ganglion::GanglionHandle;
use crate::membrane::{
    BoundaryCellTemplate, EscalationPacket, MembranePolicy, ResponseMode, ScalingStrategy,
    choose_scaling, classify,
};
use crate::metabolism::SpaceConfig;
use crate::organism::Organism;
use crate::signal::Signal;
use crate::substrate::Store;

impl<S: Store> Organism<S> {
    /// Attach a boundary template to a cell (activate genes + record policy).
    ///
    /// # Errors
    ///
    /// Returns an error if the cell id is unknown.
    pub fn attach_boundary(
        &mut self,
        cell_id: CellId,
        template: BoundaryCellTemplate,
    ) -> Result<()> {
        let cell = self.population.get_mut(cell_id).ok_or_else(|| {
            BiomimicryError::Organism(format!("attach_boundary: unknown cell {cell_id:?}"))
        })?;
        template.apply_to(cell);
        let policy = template.to_policy();
        self.boundary_policies.insert(cell_id, policy);
        self.default_boundary_template = Some(template);
        Ok(())
    }

    /// Classify an inbound external stimulus and either perturb (reflex) or escalate.
    ///
    /// Escalation fills [`Self::escalation_inbox`] and does **not** open the commit gate.
    ///
    /// # Errors
    ///
    /// Returns an error if reflex perturb fails.
    pub fn ingress(&mut self, signal: Signal) -> Result<ResponseMode> {
        let policy = self.ingress_policy();
        let mode = classify(&signal, &policy);
        match mode {
            ResponseMode::Reflex => {
                self.perturb(signal)?;
            }
            ResponseMode::Escalation => {
                let options = (self.escalation_builder)(&signal);
                self.escalation_inbox.push(EscalationPacket {
                    stimulus_id: signal.id,
                    options,
                });
            }
        }
        Ok(mode)
    }

    /// Drain queued escalation packets.
    pub fn drain_escalations(&mut self) -> Vec<EscalationPacket> {
        std::mem::take(&mut self.escalation_inbox)
    }

    /// Choose and apply membrane scaling (breadth = recruit+boundary, depth = ganglion).
    ///
    /// # Errors
    ///
    /// Returns an error if recruitment fails.
    pub fn scale_membrane(
        &mut self,
        inbound_rate_milli: u32,
        depth_pressure_milli: u32,
    ) -> Result<ScalingStrategy> {
        let strategy = choose_scaling(inbound_rate_milli, depth_pressure_milli);
        match strategy {
            ScalingStrategy::Breadth => {
                self.apply_recruit(1)?;
                if let Some(tmpl) = self.default_boundary_template.clone() {
                    // Newest living cell (highest id) becomes an additional boundary.
                    let newest = self
                        .population
                        .cells()
                        .iter()
                        .filter(|c| c.lifecycle() != LifecycleState::Dead)
                        .map(|c| c.id)
                        .max();
                    if let Some(id) = newest {
                        self.attach_boundary(id, tmpl)?;
                    }
                }
            }
            ScalingStrategy::Depth => {
                let has_nonempty = self.ganglia.iter().any(|g| !g.members.is_empty());
                if !has_nonempty {
                    let members: Vec<CellId> = self
                        .boundary_policies
                        .keys()
                        .copied()
                        .chain(
                            self.population
                                .cells()
                                .iter()
                                .filter(|c| c.lifecycle() != LifecycleState::Dead)
                                .map(|c| c.id),
                        )
                        .collect::<std::collections::BTreeSet<_>>()
                        .into_iter()
                        .collect();
                    let handle = GanglionHandle(
                        self.ganglia
                            .iter()
                            .map(|g| g.handle.0)
                            .max()
                            .unwrap_or(0)
                            .saturating_add(1),
                    );
                    self.attach_ganglion(
                        handle,
                        "membrane_depth",
                        members.len().saturating_add(4).max(4),
                        SpaceConfig { k: 2 },
                        members,
                    );
                }
            }
        }
        Ok(strategy)
    }

    fn ingress_policy(&self) -> MembranePolicy {
        // Prefer lowest living boundary cell policy; else default / empty.
        let mut boundary_ids: Vec<_> = self.boundary_policies.keys().copied().collect();
        boundary_ids.sort();
        for id in boundary_ids {
            if self
                .population
                .get(id)
                .is_some_and(|c| c.lifecycle() != LifecycleState::Dead)
            {
                return self.boundary_policies[&id];
            }
        }
        self.default_boundary_template
            .as_ref()
            .map(BoundaryCellTemplate::to_policy)
            .unwrap_or_default()
    }
}
