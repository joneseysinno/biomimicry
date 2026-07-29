//! Perturb-and-settle a ganglion as a unit (`stimulate`).

use std::collections::BTreeMap;

use smol_str::SmolStr;

use crate::attractor::SettleStatus;
use crate::cell::Operation;
use crate::effector::EffectorId;
use crate::error::{BiomimicryError, Result};
use crate::ganglion::port::{GanglionPort, PortDirection, inputs};
use crate::ganglion::GanglionHandle;
use crate::medium::ScheduledOp;
use crate::organism::Organism;
use crate::signal::{CausalStamp, Payload, Signal, SignalKind, SignalType, Value};
use crate::substrate::Store;

/// Result of a ganglion stimulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GanglionResponse {
    /// Settle outcome.
    pub status: SettleStatus,
    /// Values read from output ports (kind, value).
    pub outputs: Vec<(SignalKind, Value)>,
    /// Effector sink diff across the stimulation.
    pub effects: BTreeMap<EffectorId, Value>,
    /// Outer cycles consumed by settle.
    pub cycles: u32,
}

/// Perturb a ganglion with `input`, settle up to `cap` ticks, and read ports / effects.
///
/// # Errors
///
/// - [`BiomimicryError::StimulateReentered`] if already inside `stimulate`
/// - [`BiomimicryError::PortUnsatisfied`] if an input port has no member cells
/// - [`BiomimicryError::PortShapeMismatch`] if a split field / value fails the port shape
/// - Organism / settle failures
#[allow(clippy::needless_pass_by_value)] // owned `Value` is the public stimulate API
pub fn stimulate<S: Store>(
    org: &mut Organism<S>,
    handle: GanglionHandle,
    input: Value,
    cap: u32,
) -> Result<GanglionResponse> {
    if org.stimulating {
        return Err(BiomimicryError::StimulateReentered);
    }
    org.stimulating = true;
    let result = stimulate_inner(org, handle, &input, cap);
    org.stimulating = false;
    result
}

#[allow(clippy::too_many_lines)]
fn stimulate_inner<S: Store>(
    org: &mut Organism<S>,
    handle: GanglionHandle,
    input: &Value,
    cap: u32,
) -> Result<GanglionResponse> {
    let prior_effects = org.effects();
    let cycles_before = org.scheduler.outer_cycles;

    let (in_ports, out_ports) = {
        let g = org
            .ganglia
            .iter()
            .find(|g| g.handle == handle)
            .ok_or_else(|| BiomimicryError::Organism(format!("unknown ganglion {handle:?}")))?;
        let in_ports: Vec<GanglionPort> = g
            .ports
            .iter()
            .filter(|p| p.direction == PortDirection::In)
            .cloned()
            .collect();
        let out_ports: Vec<GanglionPort> = g
            .ports
            .iter()
            .filter(|p| p.direction == PortDirection::Out)
            .cloned()
            .collect();
        (in_ports, out_ports)
    };

    let record_has_port_fields = matches!(input, Value::Record(map) if {
        in_ports.iter().any(|p| record_has_port_key(map, p))
    });

    for port in &in_ports {
        // Partial records (composition): only inject ports whose keys are present.
        if record_has_port_fields {
            if let Value::Record(map) = input {
                if !record_has_port_key(map, port) {
                    continue;
                }
            }
        }
        let value = port_input_value(port, input, record_has_port_fields)?;
        if !port.shape.matches(&value) {
            return Err(BiomimicryError::PortShapeMismatch {
                kind: port.kind.as_str().into(),
                expected: format!("{:?}", port.shape),
                got: format!("{:?}", value.shape()),
            });
        }
        let targets = {
            let g = org
                .ganglia
                .iter()
                .find(|g| g.handle == handle)
                .expect("ganglion checked");
            inputs(g, org.population.cells(), port)
        };
        if targets.is_empty() {
            return Err(BiomimicryError::PortUnsatisfied {
                kind: port.kind.as_str().into(),
            });
        }
        let sig = Signal::new(
            SignalType::Operational,
            port.kind.clone(),
            port.scope,
            Payload::of(value),
            targets[0],
            CausalStamp(0),
        );
        for cell in targets {
            org.scheduler.inject(ScheduledOp {
                cell,
                op: Operation::Receive(sig.clone()),
            });
        }
    }

    let status = org.settle(u64::from(cap))?;
    // Expression may converge while Phase-2 work (held Transduce → Effect) is
    // still in flight. Finish the queues so effector writes land.
    let mut guard = 0u32;
    while !org.scheduler.is_drained() && guard < cap {
        org.scheduler.delivery_ganglia = org.ganglia.clone();
        org.scheduler.cadence.k = org.effective_k();
        org.scheduler.outer_cycle(&mut org.population)?;
        org.drain_effects_into_sink();
        guard = guard.saturating_add(1);
    }
    let cycles = org.scheduler.outer_cycles.saturating_sub(cycles_before);

    let last = org.scheduler.cascade_last_outputs();
    let mut out_values = Vec::new();
    for port in &out_ports {
        let value = last
            .get(port.kind.as_str())
            .or_else(|| {
                last.iter()
                    .find(|(k, _)| SignalKind::new(k.as_str()).local_name() == port.kind.local_name())
                    .map(|(_, v)| v)
            })
            .cloned();
        if let Some(value) = value {
            out_values.push((port.kind.clone(), value));
        }
    }

    let effects = org.effect_diff(&prior_effects);
    Ok(GanglionResponse {
        status,
        outputs: out_values,
        effects,
        cycles,
    })
}

fn record_has_port_key(
    map: &std::collections::BTreeMap<SmolStr, Value>,
    port: &GanglionPort,
) -> bool {
    map.contains_key(port.kind.as_str()) || map.contains_key(port.kind.local_name())
}

fn port_input_value(
    port: &GanglionPort,
    input: &Value,
    record_has_port_fields: bool,
) -> Result<Value> {
    match input {
        Value::Record(map) if record_has_port_fields => {
            map.get(port.kind.as_str())
                .or_else(|| map.get(port.kind.local_name()))
                .cloned()
                .ok_or_else(|| BiomimicryError::PortShapeMismatch {
                    kind: port.kind.as_str().into(),
                    expected: format!(
                        "Record field `{}` or `{}`",
                        port.kind.as_str(),
                        port.kind.local_name()
                    ),
                    got: "missing field".into(),
                })
        }
        other => Ok(other.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ganglion::port::GanglionPort;
    use crate::ganglion::{Ganglion, GanglionHandle};
    use crate::genesis::{GeneId, compile, toy_dna};
    use crate::metabolism::SpaceConfig;
    use crate::organism::OrganismBuilder;
    use crate::signal::{Scope, ValueShape};

    #[test]
    fn empty_input_port_is_unsatisfied() {
        let genome = compile(&toy_dna()).unwrap();
        let seed = genome.iter().next().map(|g| g.id).unwrap();
        let mut org = OrganismBuilder::new()
            .genome(genome)
            .seed_gene(seed)
            .population_size(1)
            .without_pop_loop()
            .build()
            .unwrap();
        let handle = GanglionHandle(1);
        let mut g = Ganglion::new(handle, "empty", 2).with_space(SpaceConfig { k: 2 });
        g = g.with_port(GanglionPort::input("missing", Scope::Cluster, ValueShape::Int));
        let _ = g.try_add(org.cells()[0].id);
        org.ganglia.push(g);

        let err = stimulate(&mut org, handle, Value::Int(1), 4).unwrap_err();
        assert!(matches!(err, BiomimicryError::PortUnsatisfied { .. }));
    }

    #[test]
    fn reentrant_stimulate_errors() {
        let genome = compile(&toy_dna()).unwrap();
        let seed = genome.iter().next().map(|g| g.id).unwrap();
        let mut org = OrganismBuilder::new()
            .genome(genome)
            .seed_gene(seed)
            .population_size(1)
            .without_pop_loop()
            .build()
            .unwrap();
        org.stimulating = true;
        let err = stimulate(&mut org, GanglionHandle(1), Value::Unit, 1).unwrap_err();
        assert_eq!(err, BiomimicryError::StimulateReentered);
        let _ = GeneId(0); // silence unused in some cfgs
    }
}
