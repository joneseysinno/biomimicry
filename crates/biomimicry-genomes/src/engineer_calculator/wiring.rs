//! Compile calculator DNA and wire Phase 1 / Phase 2 brains.

use std::sync::Arc;

use biomimicry::expression::{NetworkRegulator, RegulatoryRule, RuleCondition, RuleNetwork};
use biomimicry::genesis::{EndpointPolarity, GeneId, Primitive, compile};
use biomimicry::metabolism::Cadence;
use biomimicry::organism::{Organism, OrganismBuilder};
use biomimicry::signal::{
    CausalStamp, MetaValue, Payload, Scope, Signal, SignalKind, SignalType,
};
use biomimicry::substrate::MemoryStore;
use biomimicry::transduction::{Cascade, CascadeTransducer, TransductionFn, TransductionKernel};
use biomimicry::cell::CellId;

use crate::engineer_calculator::dna::calculator_dna;
use crate::engineer_calculator::kernels::{kernel_add, kernel_mul};
use crate::engineer_calculator::kinds::{
    META_VALUE, OPERAND_A, OPERAND_B, OP_ADD, OP_MUL, RESULT,
};

/// Compiled calculator genome handles.
#[derive(Debug, Clone)]
pub struct CalculatorHandles {
    /// Shared genome.
    pub genome: Arc<biomimicry::genesis::Genome>,
    /// Add sensor gene.
    pub sensor_add: GeneId,
    /// Mul sensor gene.
    pub sensor_mul: GeneId,
    /// Shared gate gene.
    pub gate: GeneId,
    /// Add reducer gene.
    pub reducer_add: GeneId,
    /// Mul reducer gene.
    pub reducer_mul: GeneId,
    /// Bound (self-complement) gene.
    pub bound: GeneId,
    /// Boundary emit gene.
    pub boundary: GeneId,
}

/// Compile calculator DNA and resolve gene ids.
#[must_use]
pub fn calculator_handles() -> CalculatorHandles {
    let dna = calculator_dna();
    let genome = compile(&dna).expect("compile calculator_dna");
    CalculatorHandles {
        genome: Arc::clone(&genome),
        sensor_add: find_receptor_gene(&genome, "sensor_add"),
        sensor_mul: find_receptor_gene(&genome, "sensor_mul"),
        gate: find_receptor_gene(&genome, "gate"),
        reducer_add: find_receptor_gene(&genome, "reducer_add"),
        reducer_mul: find_receptor_gene(&genome, "reducer_mul"),
        bound: find_kind_gene(&genome, "bound"),
        boundary: find_kind_gene(&genome, "boundary"),
    }
}

fn find_receptor_gene(genome: &biomimicry::genesis::Genome, kind: &str) -> GeneId {
    genome
        .iter()
        .find(|g| {
            g.cistron.kind.as_str() == kind
                && g.cistron.endpoints.iter().any(|ep| {
                    ep.primitive == Primitive::Receptor
                        && ep.polarity == EndpointPolarity::Positive
                })
        })
        .map_or_else(|| panic!("missing receptor gene {kind}"), |g| g.id)
}

fn find_kind_gene(genome: &biomimicry::genesis::Genome, kind: &str) -> GeneId {
    genome
        .genes_of_kind(kind)
        .next()
        .unwrap_or_else(|| panic!("missing gene kind {kind}"))
}

/// Phase 1 rule network: operator kinds keep gate expressed alongside sensors.
#[must_use]
pub fn calculator_network(handles: &CalculatorHandles) -> RuleNetwork {
    RuleNetwork::new()
        .with_rule(
            RegulatoryRule::new("gate_on_add")
                .with_condition(RuleCondition::SignalKind(SignalKind::new(OP_ADD)))
                .with_express([handles.gate, handles.sensor_add]),
        )
        .with_rule(
            RegulatoryRule::new("gate_on_mul")
                .with_condition(RuleCondition::SignalKind(SignalKind::new(OP_MUL)))
                .with_express([handles.gate, handles.sensor_mul]),
        )
}

/// Phase 2 cascades: reducer genes recruit arithmetic kernels, then lift to result.
#[must_use]
pub fn calculator_transducer(handles: &CalculatorHandles) -> CascadeTransducer {
    let add_cascade = Cascade::new()
        .with_step(kernel_add())
        .with_step(result_lift_step());
    let mul_cascade = Cascade::new()
        .with_step(kernel_mul())
        .with_step(result_lift_step());
    CascadeTransducer::new()
        .with_cascade(handles.reducer_add, add_cascade)
        .with_cascade(handles.reducer_mul, mul_cascade)
}

fn result_lift_step() -> TransductionFn {
    TransductionFn::identity_echo("calc.readout", RESULT)
        .with_scope(Scope::Systemwide)
        .with_payload(Payload::empty().with_observation("calc.result"))
        .with_kernel(TransductionKernel::Forward)
}

/// Binary operation perturbation (`operand.a` / `operand.b` meta as decimal strings).
#[must_use]
pub fn binary_op_signal(op_kind: &str, a: i64, b: i64) -> Signal {
    Signal::new(
        SignalType::Operational,
        op_kind,
        Scope::Systemwide,
        Payload::empty()
            .with_strength(100)
            .with_meta(OPERAND_A, MetaValue::new(a.to_string()))
            .with_meta(OPERAND_B, MetaValue::new(b.to_string())),
        CellId(0),
        CausalStamp(0),
    )
}

/// Organism ready for calculator smoke settles (two cells, reducer_add seeded).
#[must_use]
pub fn calculator_ready(seed: u64) -> Organism<MemoryStore> {
    let handles = calculator_handles();
    OrganismBuilder::new()
        .genome(Arc::clone(&handles.genome))
        .seed(seed)
        .cadence(Cadence::new(2))
        .population_size(2)
        .target_population(2)
        .seed_gene(handles.reducer_add)
        .without_pop_loop()
        .regulator(NetworkRegulator::new(calculator_network(&handles)))
        .transducer(calculator_transducer(&handles))
        .build()
        .expect("build calculator organism")
}

/// Read the first numeric [`META_VALUE`] from observation payloads after settle.
#[must_use]
pub fn readout_value(org: &Organism<MemoryStore>) -> Option<i64> {
    for sample in org.collector.samples() {
        if let Some(v) = sample.payload.metadata.get(&META_VALUE.into()) {
            if let Ok(n) = v.as_str().parse() {
                return Some(n);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn g2_calculator_handles_resolve_every_gene() {
        let h = calculator_handles();
        assert_ne!(h.sensor_add, h.sensor_mul);
        assert_ne!(h.reducer_add, h.reducer_mul);
        assert_ne!(h.gate, h.bound);
        assert_ne!(h.boundary, h.gate);
        let _ = calculator_network(&h);
        let _ = calculator_transducer(&h);
    }
}
