//! Spatial hypergraph container — the in-engine DNA substrate view.
//!
//! Persistence is behind [`crate::substrate::Store`]. [`SpatialHypergraph::load`]
//! / [`SpatialHypergraph::persist`] exercise the hypergraph-facing Store methods
//! so `infinite-db` is a drop-in at M7.

use std::collections::BTreeMap;

use super::{Hyperedge, PrimitiveNode, PrimitiveNodeId};
use crate::error::{BiomimicryError, Result};
use crate::substrate::Store;

/// In-engine spatial hypergraph (DNA substrate view).
///
/// Nodes are keyed by content-addressed [`PrimitiveNodeId`]. Insertion order of
/// hyperedges does not affect gene identity (ids are content hashes).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpatialHypergraph {
    nodes: BTreeMap<PrimitiveNodeId, PrimitiveNode>,
    hyperedges: Vec<Hyperedge>,
    /// Reserved for M7 Hilbert / spatial index — unused in M1 (results unchanged).
    structural_index: (),
}

/// Alias preserved for the Store boundary and prelude.
pub type Hypergraph = SpatialHypergraph;

impl SpatialHypergraph {
    /// Create an empty hypergraph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a primitive node. Re-inserting an equal node is a no-op; conflicting
    /// payload for the same id is an error.
    ///
    /// # Errors
    ///
    /// Returns [`BiomimicryError::CompileFailed`] if an existing node id maps to
    /// a different payload (hash collision / programmer error).
    pub fn add_node(&mut self, node: PrimitiveNode) -> Result<()> {
        if let Some(existing) = self.nodes.get(&node.id) {
            if existing != &node {
                return Err(BiomimicryError::CompileFailed {
                    reason: format!(
                        "primitive node id collision: {0:?} already maps to a different node",
                        node.id
                    ),
                });
            }
            return Ok(());
        }
        self.nodes.insert(node.id, node);
        Ok(())
    }

    /// Append a hyperedge (not yet validated — validation is compile's job).
    pub fn add_hyperedge(&mut self, edge: Hyperedge) {
        self.hyperedges.push(edge);
    }

    /// Resolve a node id to its payload.
    #[must_use]
    pub fn resolve(&self, id: PrimitiveNodeId) -> Option<&PrimitiveNode> {
        self.nodes.get(&id)
    }

    /// Iterate all nodes in id order.
    pub fn nodes(&self) -> impl Iterator<Item = &PrimitiveNode> {
        self.nodes.values()
    }

    /// Number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Traverse all hyperedges (declaration / insertion order).
    pub fn iter_hyperedges(&self) -> impl Iterator<Item = &Hyperedge> {
        self.hyperedges.iter()
    }

    /// Alias used by older Store call sites.
    pub fn edges(&self) -> impl Iterator<Item = &Hyperedge> {
        self.iter_hyperedges()
    }

    /// Number of hyperedges.
    #[must_use]
    pub fn hyperedge_count(&self) -> usize {
        self.hyperedges.len()
    }

    /// Look up the primitive type id for a node (for canonical hashing).
    #[must_use]
    pub fn primitive_type_id(&self, id: PrimitiveNodeId) -> Option<u32> {
        self.nodes.get(&id).map(|n| n.primitive.type_id())
    }

    /// Persist this hypergraph through the Store's fine-grained hypergraph API.
    ///
    /// # Errors
    ///
    /// Propagates Store I/O errors.
    pub fn persist(&self, store: &mut dyn Store) -> Result<()> {
        store.clear_hypergraph()?;
        for node in self.nodes.values() {
            store.put_node(node)?;
        }
        for edge in &self.hyperedges {
            store.put_hyperedge(edge)?;
        }
        Ok(())
    }

    /// Load a hypergraph from a Store.
    ///
    /// # Errors
    ///
    /// Propagates Store I/O errors.
    pub fn load(store: &dyn Store) -> Result<Self> {
        let mut graph = Self::new();
        for node in store.iter_nodes()? {
            graph.add_node(node)?;
        }
        for edge in store.iter_hyperedges()? {
            graph.add_hyperedge(edge);
        }
        Ok(graph)
    }
}

/// Validate a hyperedge against §2.4 structural rules given a node table.
///
/// # Errors
///
/// Returns typed genesis errors for empty endpoints, empty kind, dangling
/// refs, or duplicate endpoint tuples.
pub fn validate_hyperedge(edge: &Hyperedge, graph: &SpatialHypergraph) -> Result<()> {
    if edge.endpoints.is_empty() {
        return Err(BiomimicryError::MalformedHyperedge {
            reason: "endpoints must be non-empty".into(),
        });
    }
    if edge.kind.is_empty() {
        return Err(BiomimicryError::EmptyKind);
    }
    let mut seen = std::collections::BTreeSet::new();
    for ep in &edge.endpoints {
        if graph.resolve(ep.node).is_none() {
            return Err(BiomimicryError::DanglingEndpoint { node: ep.node });
        }
        let key = (ep.node, ep.polarity, ep.role.clone(), ep.scope);
        if !seen.insert(key) {
            return Err(BiomimicryError::DuplicateEndpoint {
                node: ep.node,
                polarity: ep.polarity,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genesis::{DimensionVector, EndpointPolarity, Primitive, endpoint};
    use crate::signal::Scope;
    use crate::substrate::MemoryStore;

    #[test]
    fn store_round_trip_preserves_graph() {
        let mut g = SpatialHypergraph::new();
        let n = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([1, 2]));
        g.add_node(n.clone()).unwrap();
        g.add_hyperedge(Hyperedge::new(
            "spike",
            vec![endpoint(
                &n,
                EndpointPolarity::Positive,
                "emit",
                Some(Scope::Systemwide),
            )],
        ));
        let mut store = MemoryStore::new();
        g.persist(&mut store).unwrap();
        let loaded = SpatialHypergraph::load(&store).unwrap();
        assert_eq!(loaded, g);
    }
}
