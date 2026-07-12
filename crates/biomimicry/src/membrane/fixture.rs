//! M8 echo protocol fixture — matched SignalKind surfaces only (no adapter).

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::cell::CellId;
use crate::expression::{NetworkRegulator, RegulatoryRule, RuleCondition, RuleNetwork};
use crate::genesis::{
    DimensionVector, EndpointPolarity, GeneId, Grn, Primitive, PrimitiveNode, compile, endpoint,
};
use crate::membrane::BoundaryCellTemplate;
use crate::metabolism::Cadence;
use crate::organism::OrganismBuilder;
use crate::signal::{CausalStamp, Payload, Scope, Signal, SignalKind, SignalType};
use crate::substrate::MemoryStore;
use crate::transduction::{Cascade, CascadeTransducer, TransductionFn};

/// Compiled echo genome handles.
#[derive(Debug, Clone)]
pub struct EchoHandles {
    /// Shared genome.
    pub genome: Arc<crate::genesis::Genome>,
    /// Boundary gene: Receptor+(echo.request) + Expression+ + Transduction+.
    pub echo_path: GeneId,
    /// Secondary effector activated by the rule network.
    pub effector: GeneId,
    /// Inbound protocol kind.
    pub request_kind: &'static str,
    /// Outbound protocol kind.
    pub reply_kind: &'static str,
}

/// DNA for the toy echo protocol (matched signaling only).
#[must_use]
pub fn echo_dna() -> Grn {
    let mut g = Grn::new();

    let receptor = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 0]));
    let expr = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 0]));
    let transduction = PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, 0]));
    let r2 = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 10]));
    let e2 = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([1, 10]));

    for n in [
        receptor.clone(),
        expr.clone(),
        transduction.clone(),
        r2.clone(),
        e2.clone(),
    ] {
        g.add_node(n).expect("add node");
    }

    g.add_cistron(crate::genesis::Cistron::new(
        "echo_path",
        vec![
            endpoint(&receptor, EndpointPolarity::Positive, "echo.request", None),
            endpoint(&expr, EndpointPolarity::Positive, "activate", None),
            endpoint(&transduction, EndpointPolarity::Positive, "produce", None),
        ],
    ));

    g.add_cistron(crate::genesis::Cistron::new(
        "echo_effector",
        vec![
            endpoint(&r2, EndpointPolarity::Positive, "gate", None),
            endpoint(&e2, EndpointPolarity::Positive, "out", None),
        ],
    ));

    g
}

/// Compile echo DNA and resolve gene ids.
#[must_use]
pub fn echo_handles() -> EchoHandles {
    let dna = echo_dna();
    let genome = compile(&dna).expect("compile echo_dna");
    let echo_path = genome
        .iter()
        .find(|g| {
            g.cistron.kind.as_str() == "echo_path"
                && g.cistron.endpoints.iter().any(|ep| {
                    ep.primitive == Primitive::Receptor && ep.polarity == EndpointPolarity::Positive
                })
        })
        .map(|g| g.id)
        .expect("echo_path");
    let effector = genome
        .iter()
        .find(|g| {
            g.cistron.kind.as_str() == "echo_effector"
                && g.cistron.endpoints.iter().any(|ep| {
                    ep.primitive == Primitive::Receptor && ep.polarity == EndpointPolarity::Positive
                })
        })
        .map(|g| g.id)
        .expect("echo_effector");
    EchoHandles {
        genome,
        echo_path,
        effector,
        request_kind: "echo.request",
        reply_kind: "echo.reply",
    }
}

/// Rule network: on echo.request → activate effector.
#[must_use]
pub fn echo_network(effector: GeneId) -> RuleNetwork {
    RuleNetwork::new().with_rule(
        RegulatoryRule::new("activate_echo_effector")
            .with_condition(RuleCondition::SignalKind(SignalKind::new("echo.request")))
            .with_express([effector]),
    )
}

/// Cascade: emit Systemwide `echo.reply` for `echo_path`.
#[must_use]
pub fn echo_transducer(echo_path: GeneId) -> CascadeTransducer {
    let cascade = Cascade::new().with_step(
        TransductionFn::identity_echo("echo_reply", "echo.reply").with_scope(Scope::Systemwide),
    );
    CascadeTransducer::new().with_cascade(echo_path, cascade)
}

/// Boundary template for the echo surface.
#[must_use]
pub fn echo_boundary_template(handles: &EchoHandles) -> BoundaryCellTemplate {
    BoundaryCellTemplate::new()
        .with_receptors([handles.echo_path])
        .with_secretions([handles.echo_path])
        .with_escalation_strength_milli(800)
}

/// Echo request signal (reflex-strength by default).
#[must_use]
pub fn echo_request(strength_milli: u32) -> Signal {
    Signal::new(
        SignalType::Operational,
        "echo.request",
        Scope::Systemwide,
        Payload::empty().with_strength(strength_milli),
        CellId(0),
        CausalStamp(0),
    )
}

/// Small organism with boundary cell matching the echo protocol.
#[must_use]
pub fn echo_ready(seed: u64) -> crate::organism::Organism<MemoryStore> {
    let handles = echo_handles();
    let mut org = OrganismBuilder::new()
        .genome(Arc::clone(&handles.genome))
        .seed(seed)
        .cadence(Cadence::new(2))
        .population_size(2)
        .target_population(2)
        .seed_gene(handles.echo_path)
        .without_pop_loop()
        .regulator(NetworkRegulator::new(echo_network(handles.effector)))
        .transducer(echo_transducer(handles.echo_path))
        .build()
        .expect("build echo organism");
    let tmpl = echo_boundary_template(&handles);
    org.attach_boundary(CellId(1), tmpl)
        .expect("attach boundary");
    org
}

/// A4: fixture exposes no Protocol adapter type — only SignalKind strings + genes.
#[must_use]
pub fn echo_protocol_kinds() -> BTreeMap<&'static str, &'static str> {
    let mut m = BTreeMap::new();
    m.insert("request", "echo.request");
    m.insert("reply", "echo.reply");
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attractor::SettleStatus;
    use crate::membrane::{ResponseMode, ScalingStrategy, choose_scaling};
    use crate::membrane::{build_echo_options, classify};

    #[test]
    fn m8_a1_echo_relationship() {
        let handles = echo_handles();
        let mut org = echo_ready(42);
        let mode = org.ingress(echo_request(100)).unwrap();
        assert_eq!(mode, ResponseMode::Reflex);
        assert_eq!(org.settle(32).unwrap(), SettleStatus::Converged);
        let tags: Vec<&str> = org
            .scheduler
            .log
            .events()
            .iter()
            .map(|e| e.tag.as_str())
            .collect();
        assert!(tags.contains(&"receive"), "expected receive: {tags:?}");
        assert!(
            tags.iter().any(|t| *t == "emit" || *t == "transduce"),
            "expected emit/transduce for echo.reply cascade: {tags:?}"
        );
        assert_eq!(handles.request_kind, "echo.request");
        assert_eq!(handles.reply_kind, "echo.reply");
    }

    #[test]
    fn m8_a2_reflex_vs_escalate() {
        let mut org = echo_ready(7);
        let reflex = org.ingress(echo_request(100)).unwrap();
        assert_eq!(reflex, ResponseMode::Reflex);
        assert!(org.drain_escalations().is_empty());

        let mut org = echo_ready(8);
        let hot = echo_request(900);
        let esc = org.ingress(hot).unwrap();
        assert_eq!(esc, ResponseMode::Escalation);
        let inbox = org.drain_escalations();
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].options.len(), 2);
        assert!(!org.commit_gate().open);

        let mut org = echo_ready(9);
        let tagged = Signal::new(
            SignalType::Operational,
            "echo.request",
            Scope::Systemwide,
            Payload::empty()
                .with_strength(100)
                .with_meta(crate::membrane::DECISION_REQUIRED, "1"),
            CellId(0),
            CausalStamp(0),
        );
        assert_eq!(org.ingress(tagged).unwrap(), ResponseMode::Escalation);
    }

    #[test]
    fn m8_a3_scaling() {
        assert_eq!(choose_scaling(0, 500), ScalingStrategy::Depth);
        let mut org = echo_ready(3);
        assert!(org.ganglia.is_empty());
        let strat = org.scale_membrane(0, 500).unwrap();
        assert_eq!(strat, ScalingStrategy::Depth);
        assert!(!org.ganglia.is_empty());

        let mut org = echo_ready(4);
        let before = org.living_count();
        let strat = org.scale_membrane(500, 0).unwrap();
        assert_eq!(strat, ScalingStrategy::Breadth);
        assert!(org.living_count() > before);
    }

    #[test]
    fn m8_a4_no_adapter_kinds_only() {
        let kinds = echo_protocol_kinds();
        assert_eq!(kinds.get("request"), Some(&"echo.request"));
        assert_eq!(kinds.get("reply"), Some(&"echo.reply"));
        // Compile-time: EchoHandles has no ProtocolAdapter field — kinds + GeneIds only.
        let h = echo_handles();
        assert_eq!(h.request_kind, "echo.request");
        let _ = (classify, build_echo_options);
    }
}
