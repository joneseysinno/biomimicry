//! Reflex (automatic) vs escalation (compute options, route out).

use crate::signal::{Signal, SignalId, Tag};

/// How a boundary cell handles an inbound stimulus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResponseMode {
    /// Automatic cascade — no external decision.
    Reflex,
    /// Compute fully-informed options and route them out.
    Escalation,
}

/// Milli policy for membrane classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MembranePolicy {
    /// Escalate when strength ≥ this milli (0 = strength never escalates alone).
    pub escalation_strength_milli: u32,
}

impl MembranePolicy {
    /// Construct a policy.
    #[must_use]
    pub fn new(escalation_strength_milli: u32) -> Self {
        Self {
            escalation_strength_milli,
        }
    }
}

/// One costed alternative produced at an escalation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscalationOption {
    /// Human-readable label.
    pub label: String,
    /// Relative cost in milli-units.
    pub cost_milli: i64,
    /// Short impact tag for downstream agents.
    pub impact_tag: String,
}

/// Packet routed to an external decision agent (human or otherwise).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscalationPacket {
    /// Stimulus that triggered escalation.
    pub stimulus_id: SignalId,
    /// Fully-informed options (engine does not choose).
    pub options: Vec<EscalationOption>,
}

/// Metadata tag requesting an external decision.
pub const DECISION_REQUIRED: &str = "decision_required";

/// Classify whether a stimulus should reflex or escalate.
#[must_use]
pub fn classify(stimulus: &Signal, policy: &MembranePolicy) -> ResponseMode {
    let decision = stimulus
        .payload
        .metadata
        .contains_key(&Tag::new(DECISION_REQUIRED));
    let strength_hit = policy.escalation_strength_milli > 0
        && stimulus.payload.strength_milli >= policy.escalation_strength_milli;
    if decision || strength_hit {
        ResponseMode::Escalation
    } else {
        ResponseMode::Reflex
    }
}

/// Build two fixed costed options for the toy echo conflict surface.
#[must_use]
pub fn build_echo_options(stimulus: &Signal) -> Vec<EscalationOption> {
    let _ = stimulus;
    vec![
        EscalationOption {
            label: "accept_echo".into(),
            cost_milli: 100,
            impact_tag: "reply".into(),
        },
        EscalationOption {
            label: "defer_echo".into(),
            cost_milli: 500,
            impact_tag: "hold".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::CellId;
    use crate::signal::{CausalStamp, Payload, Scope, SignalType};

    fn stim(payload: Payload) -> Signal {
        Signal::new(
            SignalType::Operational,
            "echo.request",
            Scope::Systemwide,
            payload,
            CellId(1),
            CausalStamp(0),
        )
    }

    #[test]
    fn p1_classify_deterministic() {
        let policy = MembranePolicy::new(800);
        let a = stim(Payload::empty().with_strength(500));
        let b = stim(Payload::empty().with_strength(500));
        assert_eq!(classify(&a, &policy), classify(&b, &policy));
        assert_eq!(classify(&a, &policy), ResponseMode::Reflex);
        let hot = stim(Payload::empty().with_strength(900));
        assert_eq!(classify(&hot, &policy), ResponseMode::Escalation);
        let tagged = stim(Payload::empty().with_meta(DECISION_REQUIRED, "1"));
        assert_eq!(classify(&tagged, &policy), ResponseMode::Escalation);
    }
}
