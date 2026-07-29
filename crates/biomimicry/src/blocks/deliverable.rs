//! A1–A3 deliverable tests and organism builder for linked pipelines.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::blocks::ganglion_template::GanglionTemplate;
use crate::blocks::link::Linked;
use crate::blocks::name::BlockName;
use crate::cell::{CellId, LifecycleState};
use crate::ganglion::{Ganglion, GanglionHandle};
use crate::genesis::{GeneId, GeneOrigin, Genome};
use crate::metabolism::{Cadence, SpaceConfig};
use crate::organism::{Organism, OrganismBuilder};
use crate::substrate::MemoryStore;

/// Build an organism from a linked genome: one cell per traversed enzyme gene.
#[allow(clippy::needless_pass_by_value)] // owned Arc matches OrganismBuilder::genome
pub fn linked_organism(
    linked: &Linked,
    genome: Arc<Genome>,
    seed: u64,
) -> (Organism<MemoryStore>, BTreeMap<BlockName, GanglionHandle>) {
    let enzyme_genes: Vec<GeneId> = genome
        .iter()
        .filter(|g| {
            matches!(g.origin, GeneOrigin::Traversed) && g.cistron.transduction.is_some()
        })
        .map(|g| g.id)
        .collect();

    let n = enzyme_genes.len().max(1);
    let mut org = OrganismBuilder::new()
        .genome(Arc::clone(&genome))
        .seed(seed)
        .cadence(Cadence::new(4))
        .population_size(n)
        .seed_gene(enzyme_genes[0])
        .without_pop_loop()
        .build()
        .expect("build linked organism");

    // One enzyme gene per cell.
    for (i, cell) in org.population.cells_mut().iter_mut().enumerate() {
        for g in &enzyme_genes {
            cell.suppress(*g);
        }
        if let Some(gene) = enzyme_genes.get(i) {
            cell.activate(*gene);
        }
        assert_eq!(cell.lifecycle(), LifecycleState::Active);
    }

    // Map cistron kind → cell id.
    let mut kind_to_cell: BTreeMap<String, CellId> = BTreeMap::new();
    for (i, gene_id) in enzyme_genes.iter().enumerate() {
        if let Some(gene) = genome.get(*gene_id) {
            let cell_id = org.population.cells()[i].id;
            kind_to_cell.insert(gene.cistron.kind.as_str().to_owned(), cell_id);
        }
    }

    let mut handles = BTreeMap::new();
    for (gi, template) in linked.ganglia.iter().enumerate() {
        let handle = GanglionHandle(u64::try_from(gi + 1).unwrap_or(1));
        let mut g = Ganglion::new(
            handle,
            template.name.as_str(),
            usize::try_from(template.capacity).unwrap_or(8),
        )
        .with_space(SpaceConfig { k: 4 })
        .with_ports(template.ports());

        // Members: cells whose gene belongs to this block.
        if let Some(kinds) = linked.block_cistrons.get(&template.name) {
            for kind in kinds {
                if let Some(cell) = kind_to_cell.get(kind) {
                    let _ = g.try_add(*cell);
                }
            }
        }
        // Bridge cells that listen to this block's exports join the producer ganglion
        // so Cluster-scoped exports reach them.
        for bridge in &linked.bridges {
            if bridge.export_block == template.name {
                if let Some(cell) = kind_to_cell.get(&bridge.kind) {
                    let _ = g.try_add(*cell);
                }
            }
        }

        g.refresh_health(org.population.cells());
        org.ganglia.push(g);
        handles.insert(template.name.clone(), handle);
    }

    // Shared delivery ganglion covering all cells.
    let shared = GanglionHandle(100);
    let mut all = Ganglion::new(shared, "tissue", 64).with_space(SpaceConfig { k: 4 });
    for cell in org.population.cells() {
        let _ = all.try_add(cell.id);
    }
    all.refresh_health(org.population.cells());
    org.ganglia.push(all);

    let _ = template_capacity(linked);
    (org, handles)
}

fn template_capacity(linked: &Linked) -> Option<&GanglionTemplate> {
    linked.ganglia.first()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attractor::SettleStatus;
    use crate::blocks::error::LinkError;
    use crate::blocks::fixture::{
        alt_total_block, ambiguous_manifest, grn_canonical_bytes, missing_manifest,
        mistyped_blocks, mistyped_manifest, pipeline_blocks, pipeline_manifest,
    };
    use crate::blocks::link::{link, link_and_compile};
    use crate::effector::EffectorId;
    use crate::ganglion::stimulate;
    use crate::signal::{Value, ValueShape};

    #[test]
    fn a1_composition() {
        let blocks = pipeline_blocks();
        let manifest = pipeline_manifest();
        let (linked, genome) = link_and_compile(&blocks, &manifest).expect("link+compile");
        let (mut org, handles) = linked_organism(&linked, genome, 42);
        let sum_h = *handles.get(&BlockName::new("sum")).expect("sum ganglion");
        let scale_h = *handles.get(&BlockName::new("scale")).expect("scale ganglion");

        let ab = Value::record_from([("a", Value::Int(3000)), ("b", Value::Int(4000))])
            .expect("ab");
        let r1 = stimulate(&mut org, sum_h, ab, 64).expect("stimulate sum");
        assert_eq!(r1.status, SettleStatus::Converged, "sum status={:?}", r1.status);

        let factor = Value::record_from([("factor", Value::Int(2000))]).expect("factor");
        let r2 = stimulate(&mut org, scale_h, factor, 64).expect("stimulate scale");
        assert_eq!(
            r2.status,
            SettleStatus::Converged,
            "scale status={:?}",
            r2.status
        );

        let result = EffectorId::named("sink.result");
        assert_eq!(
            org.effects().get(&result),
            Some(&Value::Int(14000)),
            "effects={:?} r1={r1:?} r2={r2:?}",
            org.effects()
        );
    }

    #[test]
    fn a2_link_time_failures() {
        // Failures use `link` only — `compile` is never invoked (link has no
        // compile call site; `link_and_compile` is the sole wrapper that does).
        let blocks = pipeline_blocks();
        let err = link(&blocks, &missing_manifest()).expect_err("missing");
        assert!(
            err.iter().any(|e| matches!(
                e,
                LinkError::UnsatisfiedImport { block, kind, shape }
                if block.as_str() == "scale" && kind.as_str() == "total" && *shape == ValueShape::Int
            )),
            "errors={err:?}"
        );

        let mut blocks = pipeline_blocks();
        blocks.push(alt_total_block());
        let err = link(&blocks, &ambiguous_manifest()).expect_err("ambiguous");
        assert!(
            err.iter().any(|e| match e {
                LinkError::AmbiguousExport { candidates, .. } => candidates.len() >= 2,
                _ => false,
            }),
            "errors={err:?}"
        );

        let err = link(&mistyped_blocks(), &mistyped_manifest()).expect_err("mistyped");
        assert!(
            err.iter().any(|e| matches!(e, LinkError::ShapeMismatch { .. })),
            "errors={err:?}"
        );
    }

    #[test]
    fn a3_reproducibility() {
        let manifest = pipeline_manifest();
        let mut blocks_a = pipeline_blocks();
        let (l1, g1) = link_and_compile(&blocks_a, &manifest).unwrap();
        blocks_a.reverse();
        let (l2, g2) = link_and_compile(&blocks_a, &manifest).unwrap();
        assert_eq!(l1.genotype, l2.genotype);
        assert_eq!(grn_canonical_bytes(&l1.grn), grn_canonical_bytes(&l2.grn));
        assert_eq!(g1.traversed_ids(), g2.traversed_ids());
    }
}
