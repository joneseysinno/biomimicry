//! Active gene set with cached signaling profile and receptor matching.
//!
//! `expression_state` is the single source of truth; receptor / veto / emission
//! surfaces are derived and recomputed on every mutation.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::genesis::{EndpointPolarity, EndpointRef, GeneId, Genome, Primitive, Role};
use crate::signal::{Scope, Signal, scope_compatible};

/// One receptor / veto / emission endpoint exposed by the active gene set.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SurfaceEndpoint {
    /// Gene contributing this endpoint.
    pub gene: GeneId,
    /// Role / match label.
    pub role: Role,
    /// Optional scope constraint.
    pub scope: Option<Scope>,
}

impl SurfaceEndpoint {
    /// From a cistron endpoint + owning gene.
    #[must_use]
    pub fn from_endpoint(gene: GeneId, ep: &EndpointRef) -> Self {
        Self {
            gene,
            role: ep.role.clone(),
            scope: ep.scope,
        }
    }
}

/// Derived signaling interface of a cell (neighborhood is implicit here).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SignalingProfile {
    /// Active `Receptor+` endpoints (match surface).
    pub receptor_surface: Vec<SurfaceEndpoint>,
    /// Active `Receptor−` endpoints (veto surface).
    pub veto_surface: Vec<SurfaceEndpoint>,
    /// Active `Signal+` endpoints (what the cell can emit).
    pub emission_surface: Vec<SurfaceEndpoint>,
}

impl SignalingProfile {
    /// Recompute from the active set and genome.
    #[must_use]
    pub fn from_active(active: &BTreeSet<GeneId>, genome: &Genome) -> Self {
        let mut receptor_surface = Vec::new();
        let mut veto_surface = Vec::new();
        let mut emission_surface = Vec::new();

        for &id in active {
            let Some(gene) = genome.get(id) else {
                continue;
            };
            for ep in &gene.cistron.endpoints {
                match (ep.primitive, ep.polarity) {
                    (Primitive::Receptor, EndpointPolarity::Positive) => {
                        receptor_surface.push(SurfaceEndpoint::from_endpoint(id, ep));
                    }
                    (Primitive::Receptor, EndpointPolarity::Negative) => {
                        veto_surface.push(SurfaceEndpoint::from_endpoint(id, ep));
                    }
                    (Primitive::Signal, EndpointPolarity::Positive) => {
                        emission_surface.push(SurfaceEndpoint::from_endpoint(id, ep));
                    }
                    _ => {}
                }
            }
        }

        receptor_surface.sort();
        veto_surface.sort();
        emission_surface.sort();

        Self {
            receptor_surface,
            veto_surface,
            emission_surface,
        }
    }
}

/// Result of matching a signal against the active receptor surface.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReceptorMatch {
    /// Genes whose `Receptor+` matched (and were not vetoed).
    pub matched: Vec<GeneId>,
    /// Genes that would have matched but were vetoed by an active `Receptor−`.
    pub vetoed: Vec<GeneId>,
}

/// Which genes are currently expressed in a cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionState {
    active: BTreeSet<GeneId>,
    profile: SignalingProfile,
    genome: Arc<Genome>,
}

impl ExpressionState {
    /// Create an empty expression state bound to a genome.
    #[must_use]
    pub fn new(genome: Arc<Genome>) -> Self {
        Self {
            active: BTreeSet::new(),
            profile: SignalingProfile::default(),
            genome,
        }
    }

    /// Borrow the bound genome.
    #[must_use]
    pub fn genome(&self) -> &Genome {
        &self.genome
    }

    /// Borrow the cached signaling profile.
    #[must_use]
    pub fn profile(&self) -> &SignalingProfile {
        &self.profile
    }

    /// Number of active genes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.active.len()
    }

    /// Whether no genes are active.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.active.is_empty()
    }

    /// Activate a gene and recompute the signaling profile.
    pub fn activate(&mut self, gene: GeneId) {
        if self.active.insert(gene) {
            self.recompute_profile();
        }
    }

    /// Remove a gene from the active set (simple deactivation).
    pub fn suppress(&mut self, gene: GeneId) {
        if self.active.remove(&gene) {
            self.recompute_profile();
        }
    }

    /// Express the complement of `gene` (active inhibition).
    ///
    /// Uses M1's complement closure: `complement_id(g)` is always present.
    /// Also removes `gene` from the active set if present.
    pub fn suppress_by_complement(&mut self, gene: GeneId) {
        let removed = self.active.remove(&gene);
        let mut added = false;
        if let Some(g) = self.genome.get(gene) {
            // Need grn for complement_id — Genome::complement_id requires graph.
            // Use Gene::is path: flip cistron and hash.
            let flipped = g.cistron.complement();
            let cid = GeneId::of(&flipped);
            if self.genome.contains(cid) {
                added = self.active.insert(cid);
            }
        }
        if removed || added {
            self.recompute_profile();
        }
    }

    /// Whether a gene is currently active.
    #[must_use]
    pub fn is_active(&self, gene: GeneId) -> bool {
        self.active.contains(&gene)
    }

    /// Iterate active genes.
    pub fn active_genes(&self) -> impl Iterator<Item = GeneId> + '_ {
        self.active.iter().copied()
    }

    /// Match a signal against the cached receptor surface, applying vetoes.
    ///
    /// Mechanical predicate: `receptor.role == signal.kind` AND
    /// `scope_compatible(receptor.scope, signal.scope)`.
    ///
    /// An active `Receptor−` matching the same predicate **vetoes the whole
    /// signal** (global deafness) — every positive match is moved to `vetoed`.
    /// This is what lets homeostasis switch a cell deaf by expressing a
    /// complement receptor gene.
    #[must_use]
    pub fn match_receptors(&self, signal: &Signal) -> ReceptorMatch {
        let veto_hit = self
            .profile
            .veto_surface
            .iter()
            .any(|ep| surface_matches(ep, signal));

        let mut matched = Vec::new();
        let mut vetoed = Vec::new();
        let mut seen = BTreeSet::new();

        for ep in &self.profile.receptor_surface {
            if !surface_matches(ep, signal) {
                continue;
            }
            if !seen.insert(ep.gene) {
                continue;
            }
            if veto_hit {
                vetoed.push(ep.gene);
            } else {
                matched.push(ep.gene);
            }
        }

        ReceptorMatch { matched, vetoed }
    }

    /// Freshly recompute profile (for coherence tests).
    #[must_use]
    pub fn recompute_profile_fresh(&self) -> SignalingProfile {
        SignalingProfile::from_active(&self.active, &self.genome)
    }

    fn recompute_profile(&mut self) {
        self.profile = SignalingProfile::from_active(&self.active, &self.genome);
    }
}

fn surface_matches(ep: &SurfaceEndpoint, signal: &Signal) -> bool {
    signal.kind.matches_role(&ep.role) && scope_compatible(ep.scope, signal.scope)
}

/// Enqueue plan for a matched gene: inspect its endpoints for mechanism verbs.
#[must_use]
pub fn operations_for_matched_gene(
    gene_id: GeneId,
    genome: &Genome,
    inbound: &Signal,
) -> Vec<crate::cell::Operation> {
    use crate::cell::Operation;
    use crate::signal::{Payload, SignalType};

    let Some(gene) = genome.get(gene_id) else {
        return Vec::new();
    };

    let mut ops = Vec::new();
    let mut saw_expr_pos = false;
    let mut saw_trans_pos = false;
    let mut emit_ep: Option<&EndpointRef> = None;

    for ep in &gene.cistron.endpoints {
        match (ep.primitive, ep.polarity) {
            (Primitive::Expression, EndpointPolarity::Positive) => saw_expr_pos = true,
            (Primitive::Transduction, EndpointPolarity::Positive) => saw_trans_pos = true,
            (Primitive::Signal, EndpointPolarity::Positive) => {
                emit_ep = Some(ep);
            }
            _ => {}
        }
    }

    if saw_expr_pos {
        ops.push(Operation::Express {
            gene: gene_id,
            on: true,
        });
    }
    if saw_trans_pos {
        ops.push(Operation::Transduce {
            gene: gene_id,
            input: inbound.clone(),
        });
    }
    // When Transduction+ is present the cascade owns emission; skip the
    // empty Signal+ template emit so cascade payloads are not overwritten.
    if let Some(ep) = emit_ep {
        if !saw_trans_pos {
            let scope = ep.scope.unwrap_or(inbound.scope);
            let outbound = Signal::new(
                SignalType::Operational,
                signal_kind_from_role(&ep.role),
                scope,
                Payload::empty(),
                inbound.source,
                inbound.stamp,
            );
            ops.push(Operation::Emit(outbound));
        }
    }

    ops
}

fn signal_kind_from_role(role: &Role) -> crate::signal::SignalKind {
    crate::signal::SignalKind::new(role.as_str())
}
