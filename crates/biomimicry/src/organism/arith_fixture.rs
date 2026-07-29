//! M11 integration fixture: three-cell arith ganglion via `arith_dna`.

use std::sync::Arc;

use crate::cell::{CellId, LifecycleState};
use crate::effector::EffectorId;
use crate::ganglion::port::GanglionPort;
use crate::ganglion::{Ganglion, GanglionHandle};
use crate::genesis::{GeneId, GeneOrigin, arith_dna, compile};
use crate::metabolism::{Cadence, SpaceConfig};
use crate::organism::{Organism, OrganismBuilder};
use crate::signal::{Scope, SignalKind, Value, ValueShape};
use crate::substrate::MemoryStore;

/// Resolved gene ids for [`arith_dna`].
#[derive(Debug, Clone)]
pub struct ArithHandles {
    /// Compiled genome.
    pub genome: Arc<crate::genesis::Genome>,
    /// Fold(Add) gene.
    pub sum_cell: GeneId,
    /// Arith(Mul) gene.
    pub scale_cell: GeneId,
    /// Effect(result) gene.
    pub sink_cell: GeneId,
    /// Content-addressed result effector.
    pub result: EffectorId,
}

/// Compile arith DNA and resolve traversed gene ids.
#[must_use]
pub fn arith_handles() -> ArithHandles {
    let dna = arith_dna();
    let genome = compile(&dna).expect("compile arith_dna");
    let find = |kind: &str| {
        genome
            .iter()
            .find(|g| {
                g.cistron.kind.as_str() == kind && matches!(g.origin, GeneOrigin::Traversed)
            })
            .map_or_else(|| panic!("missing traversed gene {kind}"), |g| g.id)
    };
    ArithHandles {
        sum_cell: find("sum_cell"),
        scale_cell: find("scale_cell"),
        sink_cell: find("sink_cell"),
        result: EffectorId::named("arith.result"),
        genome,
    }
}

/// Build a three-cell arith organism with ports; cascades come from the genome only.
#[must_use]
pub fn arith_organism(seed: u64) -> (Organism<MemoryStore>, ArithHandles, GanglionHandle) {
    let handles = arith_handles();
    let mut org = OrganismBuilder::new()
        .genome(Arc::clone(&handles.genome))
        .seed(seed)
        .cadence(Cadence::new(4))
        .population_size(3)
        .seed_gene(handles.sum_cell)
        .without_pop_loop()
        .build()
        .expect("build arith organism");

    // Per-cell differentiation: one enzyme gene each (no host cascade registration).
    let genes = [handles.sum_cell, handles.scale_cell, handles.sink_cell];
    for (i, cell) in org.population.cells_mut().iter_mut().enumerate() {
        for g in genes {
            cell.suppress(g);
        }
        cell.activate(genes[i]);
        assert_eq!(cell.lifecycle(), LifecycleState::Active);
    }

    let handle = GanglionHandle(1);
    let kind_a = SignalKind::qualified("arith", "a");
    let kind_b = SignalKind::qualified("arith", "b");
    let kind_c = SignalKind::qualified("arith", "c");
    let kind_scaled = SignalKind::qualified("arith", "scaled");
    let mut g = Ganglion::new(handle, "arith", 4)
        .with_space(SpaceConfig { k: 4 })
        .with_ports(vec![
            GanglionPort::input(kind_a, Scope::Cluster, ValueShape::Int),
            GanglionPort::input(kind_b, Scope::Cluster, ValueShape::Int),
            GanglionPort::input(kind_c, Scope::Cluster, ValueShape::Int),
            GanglionPort::output(kind_scaled, Scope::Cluster, ValueShape::Int),
        ]);
    for id in [CellId(1), CellId(2), CellId(3)] {
        assert!(g.try_add(id));
    }
    g.refresh_health(org.population.cells());
    org.ganglia.push(g);

    (org, handles, handle)
}

/// Stimulus record `{a:3000, b:4000, c:2000}` → expected scaled `14000`.
#[must_use]
pub fn arith_stimulus() -> Value {
    Value::record_from([
        ("a", Value::Int(3000)),
        ("b", Value::Int(4000)),
        ("c", Value::Int(2000)),
    ])
    .expect("stimulus record")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attractor::SettleStatus;
    use crate::ganglion::{inputs, stimulate};
    use crate::signal::Value;

    #[test]
    fn a1_arith_computes() {
        let (mut org, handles, gh) = arith_organism(42);
        let resp = stimulate(&mut org, gh, arith_stimulus(), 32).expect("stimulate");
        assert_eq!(resp.status, SettleStatus::Converged, "status={resp:?}");
        let scaled = resp
            .outputs
            .iter()
            .find(|(k, _)| k.local_name() == "scaled")
            .map(|(_, v)| v.clone());
        assert_eq!(scaled, Some(Value::Int(14000)), "outputs={:?}", resp.outputs);
        assert_eq!(
            resp.effects.get(&handles.result),
            Some(&Value::Int(14000)),
            "effects={:?}",
            resp.effects
        );
    }

    #[test]
    fn a2_arith_replays() {
        let run = |seed| {
            let (mut org, handles, gh) = arith_organism(seed);
            let resp = stimulate(&mut org, gh, arith_stimulus(), 32).expect("stimulate");
            (
                org.scheduler.log.events().to_vec(),
                org.effects(),
                resp,
                handles.result,
            )
        };
        let (log1, sink1, resp1, result) = run(7);
        let (log2, sink2, resp2, _) = run(7);
        assert_eq!(log1, log2);
        assert_eq!(sink1, sink2);
        assert_eq!(resp1.effects.get(&result), Some(&Value::Int(14000)));
        assert_eq!(resp2.effects.get(&result), Some(&Value::Int(14000)));

        let (log3, _, resp3, _) = run(99);
        assert_eq!(resp3.effects.get(&result), Some(&Value::Int(14000)));
        assert_ne!(log1, log3);
    }

    #[test]
    fn a3_port_is_derived() {
        let (mut org, handles, _gh) = arith_organism(3);
        let port_c = org.ganglia[0]
            .ports
            .iter()
            .find(|p| p.kind.local_name() == "c")
            .cloned()
            .expect("c port");
        let before = inputs(&org.ganglia[0], org.population.cells(), &port_c);
        assert!(before.contains(&CellId(2)), "scale cell in c port: {before:?}");

        if let Some(cell) = org.population.get_mut(CellId(2)) {
            cell.suppress(handles.scale_cell);
        }
        let after = inputs(&org.ganglia[0], org.population.cells(), &port_c);
        assert!(
            !after.contains(&CellId(2)),
            "scale cell left c port without ganglion mutation: {after:?}"
        );

        if let Some(cell) = org.population.get_mut(CellId(2)) {
            cell.activate(handles.scale_cell);
        }
        let rejoined = inputs(&org.ganglia[0], org.population.cells(), &port_c);
        assert!(rejoined.contains(&CellId(2)), "rejoined: {rejoined:?}");
    }
}
