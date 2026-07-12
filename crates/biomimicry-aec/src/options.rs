//! Costed escalation options for overspanned beam (Part VIII narrative).

use biomimicry::membrane::EscalationOption;
use biomimicry::signal::Signal;

/// Two fixed options: upsize beam vs move wall back (milli-dollar costs).
#[must_use]
pub fn build_aec_beam_options(stimulus: &Signal) -> Vec<EscalationOption> {
    let _ = stimulus;
    vec![
        EscalationOption {
            label: "upsize_W14x53".into(),
            cost_milli: 4_200_000,
            impact_tag: "delay_days:2".into(),
        },
        EscalationOption {
            label: "move_wall_back_120mm".into(),
            cost_milli: 0,
            impact_tag: "corridor_min_width".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use biomimicry::cell::CellId;
    use biomimicry::signal::{CausalStamp, Payload, Scope, SignalType};

    #[test]
    fn p3_milli_costs_on_options() {
        let s = Signal::new(
            SignalType::Operational,
            crate::kinds::BEAM_OVERSPAN,
            Scope::Systemwide,
            Payload::empty(),
            CellId(0),
            CausalStamp(0),
        );
        let opts = build_aec_beam_options(&s);
        assert_eq!(opts.len(), 2);
        assert_eq!(opts[0].cost_milli, 4_200_000);
        assert_eq!(opts[1].cost_milli, 0);
    }
}
