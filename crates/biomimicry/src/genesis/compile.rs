//! Compile a DNA hypergraph into a genome by traversal + complement closure.
//!
//! Two documented steps:
//! 1. **Traversal** — one [`GeneOrigin::Traversed`] gene per valid hyperedge.
//! 2. **Closure** — for each traversed gene, ensure its complement is registered;
//!    if absent, register a [`GeneOrigin::Complement`] gene (genome-level derived,
//!    not backed by its own DNA hyperedge).

use std::sync::Arc;

use super::distance::{spread, weight_from_spread};
use super::gene::{Gene, GeneId, GeneOrigin};
use super::genome::Genome;
use super::hypergraph::{SpatialHypergraph, validate_hyperedge};
use crate::error::{BiomimicryError, Result};

/// Traverse `hypergraph`, register every valid gene hyperedge, then close under
/// complement. Returns a shareable [`Arc<Genome>`].
///
/// Invalid hyperedges cause the entire compile to fail without producing a
/// genome (P6: dangling endpoint ⇒ genome unchanged from the caller's view).
///
/// # Errors
///
/// Returns typed genesis errors when any hyperedge fails validation.
pub fn compile(hypergraph: &SpatialHypergraph) -> Result<Arc<Genome>> {
    // Validate all edges first — fail closed before mutating a genome.
    for edge in hypergraph.iter_hyperedges() {
        validate_hyperedge(edge, hypergraph)?;
    }

    let mut genome = Genome::new();

    // Step 1: traversal
    for edge in hypergraph.iter_hyperedges() {
        let mut edge = edge.clone();
        if edge.weight_milli.is_none() {
            let s = spread(&edge, hypergraph)?;
            edge.weight_milli = Some(weight_from_spread(s));
        }
        let gene = Gene::traversed(edge, hypergraph);
        genome.insert(gene, hypergraph);
    }

    // Step 2: complement closure
    let traversed: Vec<GeneId> = genome.traversed_ids().into_iter().collect();
    for id in traversed {
        let gene = genome.get(id).expect("traversed id just collected").clone();
        if gene.is_self_complement(hypergraph) {
            // Fixpoint: already registered once; closure is a no-op.
            continue;
        }
        let complement = gene.complement(hypergraph);
        let cid = complement.id;
        if !genome.contains(cid) {
            let mut edge = complement.hyperedge;
            if edge.weight_milli.is_none() {
                let s = spread(&edge, hypergraph)?;
                edge.weight_milli = Some(weight_from_spread(s));
            }
            let derived = Gene {
                id: cid,
                hyperedge: edge,
                origin: GeneOrigin::Complement(id),
            };
            genome.insert(derived, hypergraph);
        }
    }

    // Invariant check — closure must hold for every gene.
    for gene in genome.iter() {
        let cid = genome.complement_id(gene, hypergraph);
        if !genome.contains(cid) {
            return Err(BiomimicryError::CompileFailed {
                reason: format!("complement closure failed for gene {id}", id = gene.id),
            });
        }
    }

    Ok(genome.into_arc())
}
