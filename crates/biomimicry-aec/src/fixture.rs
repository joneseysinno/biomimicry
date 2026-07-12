//! Wall-move organism fixture.

use std::sync::Arc;

use biomimicry::cell::CellId;
use biomimicry::expression::{NetworkRegulator, RegulatoryRule, RuleCondition, RuleNetwork};
use biomimicry::genesis::{EndpointPolarity, GeneId, Primitive, compile};
use biomimicry::membrane::BoundaryCellTemplate;
use biomimicry::metabolism::Cadence;
use biomimicry::organism::{Organism, OrganismBuilder};
use biomimicry::signal::{CausalStamp, Payload, Scope, Signal, SignalKind, SignalType};
use biomimicry::substrate::MemoryStore;
use biomimicry::transduction::{Cascade, CascadeTransducer, TransductionFn};

use crate::dna::aec_dna;
use crate::kinds::{BEAM_OVERSPAN, DISPLACE_MILLI, RECOMPUTE_KINDS, WALL_MOVE};
use crate::options::build_aec_beam_options;
use biomimicry::membrane::DECISION_REQUIRED;

/// Compiled AEC genome handles.
#[derive(Debug, Clone)]
pub struct AecHandles {
    /// Shared genome.
    pub genome: Arc<biomimicry::genesis::Genome>,
    /// Wall-move cascade gene.
    pub wall_move_path: GeneId,
    /// Secondary effector.
    pub effector: GeneId,
}

/// Compile AEC DNA and resolve gene ids.
#[must_use]
pub fn aec_handles() -> AecHandles {
    let dna = aec_dna();
    let genome = compile(&dna).expect("compile aec_dna");
    let wall_move_path = genome
        .iter()
        .find(|g| {
            g.cistron.kind.as_str() == "wall_move_path"
                && g.cistron.endpoints.iter().any(|ep| {
                    ep.primitive == Primitive::Receptor && ep.polarity == EndpointPolarity::Positive
                })
        })
        .map(|g| g.id)
        .expect("wall_move_path");
    let effector = genome
        .iter()
        .find(|g| {
            g.cistron.kind.as_str() == "aec_effector"
                && g.cistron.endpoints.iter().any(|ep| {
                    ep.primitive == Primitive::Receptor && ep.polarity == EndpointPolarity::Positive
                })
        })
        .map(|g| g.id)
        .expect("aec_effector");
    AecHandles {
        genome,
        wall_move_path,
        effector,
    }
}

/// Rule network: on wall move → activate effector.
#[must_use]
pub fn aec_network(effector: GeneId) -> RuleNetwork {
    RuleNetwork::new().with_rule(
        RegulatoryRule::new("activate_aec_effector")
            .with_condition(RuleCondition::SignalKind(SignalKind::new(WALL_MOVE)))
            .with_express([effector]),
    )
}

/// Cascade emitting the five Part VIII recompute kinds.
#[must_use]
pub fn aec_transducer(wall_move_path: GeneId) -> CascadeTransducer {
    let mut cascade = Cascade::new();
    for kind in RECOMPUTE_KINDS {
        cascade = cascade
            .with_step(TransductionFn::identity_echo(kind, kind).with_scope(Scope::Systemwide));
    }
    CascadeTransducer::new().with_cascade(wall_move_path, cascade)
}

/// Boundary template for the wall-move surface.
#[must_use]
pub fn aec_boundary_template(handles: &AecHandles) -> BoundaryCellTemplate {
    BoundaryCellTemplate::new()
        .with_receptors([handles.wall_move_path])
        .with_secretions([handles.wall_move_path])
        .with_escalation_strength_milli(0)
}

/// Wall-move perturbation (300mm as displace_milli meta).
#[must_use]
pub fn wall_move_signal() -> Signal {
    Signal::new(
        SignalType::Operational,
        WALL_MOVE,
        Scope::Systemwide,
        Payload::empty()
            .with_strength(100)
            .with_meta(DISPLACE_MILLI, "300"),
        CellId(0),
        CausalStamp(0),
    )
}

/// Overspan escalation stimulus.
#[must_use]
pub fn overspan_signal() -> Signal {
    Signal::new(
        SignalType::Operational,
        BEAM_OVERSPAN,
        Scope::Systemwide,
        Payload::empty()
            .with_strength(100)
            .with_meta(DECISION_REQUIRED, "1"),
        CellId(0),
        CausalStamp(0),
    )
}

/// Organism ready for the wall-move scenario.
#[must_use]
pub fn wall_move_ready(seed: u64) -> Organism<MemoryStore> {
    let handles = aec_handles();
    let mut org = OrganismBuilder::new()
        .genome(Arc::clone(&handles.genome))
        .seed(seed)
        .cadence(Cadence::new(2))
        .population_size(2)
        .target_population(2)
        .seed_gene(handles.wall_move_path)
        .without_pop_loop()
        .regulator(NetworkRegulator::new(aec_network(handles.effector)))
        .transducer(aec_transducer(handles.wall_move_path))
        .build()
        .expect("build aec organism");
    org.attach_boundary(CellId(1), aec_boundary_template(&handles))
        .expect("attach boundary");
    org.set_escalation_builder(build_aec_beam_options);
    org
}
