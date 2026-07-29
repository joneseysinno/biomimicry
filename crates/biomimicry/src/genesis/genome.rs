//! Genome: compiled catalog of genes with lookups and complement closure.
//!
//! Built once by [`crate::genesis::compile()`], then treated as read-only.
//! Shared across cells via [`std::sync::Arc`].

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::{EndpointPolarity, Gene, GeneId, GeneOrigin, Grn, Primitive};
use crate::transduction::Cascade;

/// Compiled genome: the catalog of expressible genes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Genome {
    genes: BTreeMap<GeneId, Gene>,
    by_kind: BTreeMap<String, BTreeSet<GeneId>>,
    /// Reverse participation index: all genes where a primitive appears at a polarity.
    by_participation: BTreeMap<(Primitive, EndpointPolarity), BTreeSet<GeneId>>,
    /// Cascades resolved from cistron transduction specs (compile step 3).
    cascades: BTreeMap<GeneId, Cascade>,
}

impl Genome {
    /// Create an empty genome.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Wrap as a cheaply shareable `Arc`.
    #[must_use]
    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Whether a gene id is present.
    #[must_use]
    pub fn contains(&self, id: GeneId) -> bool {
        self.genes.contains_key(&id)
    }

    /// Look up a gene by id.
    #[must_use]
    pub fn get(&self, id: GeneId) -> Option<&Gene> {
        self.genes.get(&id)
    }

    /// Iterate all genes (id order).
    pub fn iter(&self) -> impl Iterator<Item = &Gene> {
        self.genes.values()
    }

    /// Number of registered genes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.genes.len()
    }

    /// Whether the genome has no genes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.genes.is_empty()
    }

    /// Content id of `gene`'s complement (may equal `gene.id` if self-complementary).
    #[must_use]
    pub fn complement_id(&self, gene: &Gene, graph: &Grn) -> GeneId {
        let flipped = gene.cistron.complement();
        GeneId::of_in_graph(&flipped, graph)
    }

    /// The complement gene, if registered.
    #[must_use]
    pub fn complement_of(&self, id: GeneId, graph: &Grn) -> Option<&Gene> {
        let gene = self.get(id)?;
        let cid = self.complement_id(gene, graph);
        self.get(cid)
    }

    /// Genes where `primitive` participates with `polarity`.
    pub fn genes_with(
        &self,
        primitive: Primitive,
        polarity: EndpointPolarity,
    ) -> impl Iterator<Item = GeneId> + '_ {
        self.by_participation
            .get(&(primitive, polarity))
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }

    /// Genes registered under a kind label.
    pub fn genes_of_kind(&self, kind: &str) -> impl Iterator<Item = GeneId> + '_ {
        self.by_kind
            .get(kind)
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }

    /// Traversed-only gene ids (excludes complement-derived).
    #[must_use]
    pub fn traversed_ids(&self) -> BTreeSet<GeneId> {
        self.genes
            .values()
            .filter(|g| matches!(g.origin, GeneOrigin::Traversed))
            .map(|g| g.id)
            .collect()
    }

    /// Cascades derived from gene transduction specs (compile step 3).
    #[must_use]
    pub fn cascades(&self) -> &BTreeMap<GeneId, Cascade> {
        &self.cascades
    }

    /// Register a resolved cascade for `gene` (compile step 3 / tests).
    pub(crate) fn insert_cascade(&mut self, gene: GeneId, cascade: Cascade) {
        self.cascades.insert(gene, cascade);
    }

    /// Insert a gene and maintain indices. Duplicate id is a no-op (same gene).
    pub(crate) fn insert(&mut self, gene: Gene, graph: &Grn) {
        let id = gene.id;
        if self.genes.contains_key(&id) {
            return;
        }
        self.by_kind
            .entry(gene.cistron.kind.0.clone())
            .or_default()
            .insert(id);
        for ep in &gene.cistron.endpoints {
            if let Some(node) = graph.resolve(ep.node) {
                self.by_participation
                    .entry((node.primitive, ep.polarity))
                    .or_default()
                    .insert(id);
            }
        }
        self.genes.insert(id, gene);
    }
}
