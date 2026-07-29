//! Compile a DNA GRN into a genome by traversal + complement closure + enzyme resolution.
//!
//! Three documented steps:
//! 1. **Traversal** — one [`GeneOrigin::Traversed`] gene per valid cistron.
//! 2. **Closure** — for each traversed gene, ensure its complement is registered;
//!    if absent, register a [`GeneOrigin::Complement`] gene (genome-level derived,
//!    not backed by its own DNA cistron).
//! 3. **Enzyme resolution** — for every gene carrying a transduction spec, build its
//!    [`crate::transduction::Cascade`] and store it on the genome.

use std::sync::Arc;

use super::distance::{spread, weight_from_spread};
use super::gene::{Gene, GeneId, GeneOrigin};
use super::genome::Genome;
use super::grn::{Grn, validate_cistron};
use crate::error::{BiomimicryError, Result};
use crate::transduction::cascade_from_spec;

/// Traverse `grn`, register every valid gene cistron, then close under
/// complement. Returns a shareable [`Arc<Genome>`].
///
/// Invalid cistrons cause the entire compile to fail without producing a
/// genome (P6: dangling endpoint ⇒ genome unchanged from the caller's view).
///
/// # Errors
///
/// Returns typed genesis errors when any cistron fails validation.
pub fn compile(grn: &Grn) -> Result<Arc<Genome>> {
    // Validate all edges first — fail closed before mutating a genome.
    for edge in grn.iter_cistrons() {
        validate_cistron(edge, grn)?;
    }

    let mut genome = Genome::new();

    // Step 1: traversal
    for edge in grn.iter_cistrons() {
        let mut edge = edge.clone();
        if edge.weight_milli.is_none() {
            let s = spread(&edge, grn)?;
            edge.weight_milli = Some(weight_from_spread(s));
        }
        let gene = Gene::traversed(edge, grn);
        genome.insert(gene, grn);
    }

    // Step 2: complement closure
    let traversed: Vec<GeneId> = genome.traversed_ids().into_iter().collect();
    for id in traversed {
        let gene = genome.get(id).expect("traversed id just collected").clone();
        if gene.is_self_complement(grn) {
            // Fixpoint: already registered once; closure is a no-op.
            continue;
        }
        let complement = gene.complement(grn);
        let cid = complement.id;
        if !genome.contains(cid) {
            let mut edge = complement.cistron;
            if edge.weight_milli.is_none() {
                let s = spread(&edge, grn)?;
                edge.weight_milli = Some(weight_from_spread(s));
            }
            let derived = Gene {
                id: cid,
                cistron: edge,
                origin: GeneOrigin::Complement(id),
            };
            genome.insert(derived, grn);
        }
    }

    // Invariant check — closure must hold for every gene.
    for gene in genome.iter() {
        let cid = genome.complement_id(gene, grn);
        if !genome.contains(cid) {
            return Err(BiomimicryError::CompileFailed {
                reason: format!("complement closure failed for gene {id}", id = gene.id),
            });
        }
    }

    // Step 3: enzyme resolution — cascade per spec-carrying gene.
    // Complement-derived genes already carry inhibitory specs via `Cistron::complement`.
    let gene_ids: Vec<GeneId> = genome.iter().map(|g| g.id).collect();
    for id in gene_ids {
        let Some(gene) = genome.get(id) else {
            continue;
        };
        if let Some(spec) = &gene.cistron.transduction {
            genome.insert_cascade(id, cascade_from_spec(spec));
        }
    }

    Ok(genome.into_arc())
}
