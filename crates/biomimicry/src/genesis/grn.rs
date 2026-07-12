//! Grn — the Gene Regulatory Network: the complete graph of regulatory
//! interactions from which the genome is compiled by traversal.
//!
//! Nodes are primitives (content-addressed [`PrimitiveNodeId`]); edges are
//! [`Cistron`]s. "Spatial" lives here as a property, not a name: primitive
//! proximity (an infinite-db curve index, wired at M7) makes nearby
//! combinations cheap and distant ones rare. Persistence is behind
//! [`crate::substrate::Store`]; `infinite-db` is a drop-in backend.

use std::collections::BTreeMap;

use super::{Cistron, PrimitiveNode, PrimitiveNodeId};
use crate::error::{BiomimicryError, Result};
use crate::substrate::Store;

/// In-engine gene regulatory network (DNA substrate view).
///
/// Nodes are keyed by content-addressed [`PrimitiveNodeId`]. Insertion order of
/// cistrons does not affect gene identity (ids are content hashes).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Grn {
    nodes: BTreeMap<PrimitiveNodeId, PrimitiveNode>,
    cistrons: Vec<Cistron>,
    /// Reserved for M7 Hilbert / spatial index — unused in M1 (results unchanged).
    structural_index: (),
}

impl Grn {
    /// Create an empty GRN.
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

    /// Append a cistron (not yet validated — validation is compile's job).
    pub fn add_cistron(&mut self, edge: Cistron) {
        self.cistrons.push(edge);
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

    /// Traverse all cistrons (declaration / insertion order).
    pub fn iter_cistrons(&self) -> impl Iterator<Item = &Cistron> {
        self.cistrons.iter()
    }

    /// Alias used by older Store call sites.
    pub fn edges(&self) -> impl Iterator<Item = &Cistron> {
        self.iter_cistrons()
    }

    /// Number of cistrons.
    #[must_use]
    pub fn cistron_count(&self) -> usize {
        self.cistrons.len()
    }

    /// Look up the primitive type id for a node (for canonical hashing).
    #[must_use]
    pub fn primitive_type_id(&self, id: PrimitiveNodeId) -> Option<u32> {
        self.nodes.get(&id).map(|n| n.primitive.type_id())
    }

    /// Persist this grn through the Store's fine-grained grn API.
    ///
    /// # Errors
    ///
    /// Propagates Store I/O errors.
    pub fn persist(&self, store: &mut dyn Store) -> Result<()> {
        store.clear_grn()?;
        for node in self.nodes.values() {
            store.put_node(node)?;
        }
        for edge in &self.cistrons {
            store.put_cistron(edge)?;
        }
        Ok(())
    }

    /// Load a grn from a Store.
    ///
    /// # Errors
    ///
    /// Propagates Store I/O errors.
    pub fn load(store: &dyn Store) -> Result<Self> {
        let mut graph = Self::new();
        for node in store.iter_nodes()? {
            graph.add_node(node)?;
        }
        for edge in store.iter_cistrons()? {
            graph.add_cistron(edge);
        }
        Ok(graph)
    }
}

/// Validate a cistron against §2.4 structural rules given a node table.
///
/// # Errors
///
/// Returns typed genesis errors for empty endpoints, empty kind, dangling
/// refs, or duplicate endpoint tuples.
pub fn validate_cistron(edge: &Cistron, graph: &Grn) -> Result<()> {
    if edge.endpoints.is_empty() {
        return Err(BiomimicryError::MalformedCistron {
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
        let mut g = Grn::new();
        let n = PrimitiveNode::new(Primitive::Signal, DimensionVector::new([1, 2]));
        g.add_node(n.clone()).unwrap();
        g.add_cistron(Cistron::new(
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
        let loaded = Grn::load(&store).unwrap();
        assert_eq!(loaded, g);
    }
}
