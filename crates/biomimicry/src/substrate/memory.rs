//! In-memory [`Store`] implementation — default for fast, deterministic tests.

use std::collections::BTreeMap;

use crate::causality::{CausalDag, CausalEventLog, CausalNode};
use crate::error::{BiomimicryError, Result};
use crate::genesis::{Hyperedge, PrimitiveNode, PrimitiveNodeId};
use crate::substrate::{BranchId, SnapshotId, SnapshotMeta, Store};

/// Retained snapshot payload (hypergraph + causal + optional event log).
#[derive(Debug, Clone, Default)]
pub struct SnapshotPayload {
    /// Primitive nodes at snapshot time.
    pub nodes: BTreeMap<PrimitiveNodeId, PrimitiveNode>,
    /// Hyperedges at snapshot time.
    pub hyperedges: Vec<Hyperedge>,
    /// Causal DAG at snapshot time.
    pub causal: CausalDag,
    /// Optional event log clone for organism restore.
    pub event_log: Option<CausalEventLog>,
}

/// Zero-dependency in-memory store (M0 default; M7 retains real snapshots).
#[derive(Debug, Clone, Default)]
pub struct MemoryStore {
    nodes: BTreeMap<PrimitiveNodeId, PrimitiveNode>,
    hyperedges: Vec<Hyperedge>,
    causal: CausalDag,
    /// Working event log attached before snapshot (organism path).
    event_log: Option<CausalEventLog>,
    /// Log restored from the last [`Store::restore`] call.
    restored_event_log: Option<CausalEventLog>,
    snapshots: BTreeMap<SnapshotId, SnapshotPayload>,
    /// Branch tip → snapshot id.
    branches: BTreeMap<BranchId, SnapshotId>,
    current_branch: BranchId,
    next_snapshot: u64,
    next_branch: u64,
}

impl MemoryStore {
    /// Create an empty in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Capture live state as a payload.
    #[must_use]
    pub fn capture_live(&self) -> SnapshotPayload {
        SnapshotPayload {
            nodes: self.nodes.clone(),
            hyperedges: self.hyperedges.clone(),
            causal: self.causal.clone(),
            event_log: self.event_log.clone(),
        }
    }

    /// Apply a payload to live state (does not clear snapshot map).
    pub fn apply_payload(&mut self, payload: &SnapshotPayload) {
        self.nodes.clone_from(&payload.nodes);
        self.hyperedges.clone_from(&payload.hyperedges);
        self.causal.clone_from(&payload.causal);
        self.event_log.clone_from(&payload.event_log);
    }

    /// Encode live state + snapshots for durable backends (M7 InfiniteDbStore).
    ///
    /// # Errors
    ///
    /// Returns an error if encoding fails.
    pub fn to_durable_bytes(&self) -> Result<Vec<u8>> {
        Ok(durable::encode(self))
    }

    /// Decode a store from durable bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the blob is corrupt or incomplete.
    pub fn from_durable_bytes(bytes: &[u8]) -> Result<Self> {
        durable::decode(bytes)
    }
}

impl Store for MemoryStore {
    fn clear_hypergraph(&mut self) -> Result<()> {
        self.nodes.clear();
        self.hyperedges.clear();
        Ok(())
    }

    fn put_node(&mut self, node: &PrimitiveNode) -> Result<()> {
        self.nodes.insert(node.id, node.clone());
        Ok(())
    }

    fn get_node(&self, id: PrimitiveNodeId) -> Result<Option<PrimitiveNode>> {
        Ok(self.nodes.get(&id).cloned())
    }

    fn iter_nodes(&self) -> Result<Vec<PrimitiveNode>> {
        Ok(self.nodes.values().cloned().collect())
    }

    fn put_hyperedge(&mut self, edge: &Hyperedge) -> Result<()> {
        self.hyperedges.push(edge.clone());
        Ok(())
    }

    fn iter_hyperedges(&self) -> Result<Vec<Hyperedge>> {
        Ok(self.hyperedges.clone())
    }

    fn append_causal(&mut self, node: CausalNode) -> Result<()> {
        self.causal.append(node);
        Ok(())
    }

    fn replace_causal_dag(&mut self, dag: CausalDag) -> Result<()> {
        self.causal = dag;
        Ok(())
    }

    fn load_causal_dag(&self) -> Result<CausalDag> {
        Ok(self.causal.clone())
    }

    fn prepare_snapshot_log(&mut self, log: Option<CausalEventLog>) {
        self.event_log = log;
    }

    fn take_restored_event_log(&mut self) -> Option<CausalEventLog> {
        self.restored_event_log.take()
    }

    fn snapshot(&mut self, label: &str) -> Result<SnapshotMeta> {
        let id = SnapshotId(self.next_snapshot);
        self.next_snapshot = self.next_snapshot.saturating_add(1);
        let payload = self.capture_live();
        self.snapshots.insert(id, payload);
        Ok(SnapshotMeta {
            id,
            branch: self.current_branch,
            label: label.to_owned(),
        })
    }

    fn branch(&mut self, from: SnapshotId, _label: &str) -> Result<BranchId> {
        let payload = self
            .snapshots
            .get(&from)
            .ok_or(BiomimicryError::SnapshotUnknown(from))?
            .clone();
        let branch_id = BranchId(self.next_branch);
        self.next_branch = self.next_branch.saturating_add(1);
        let tip = SnapshotId(self.next_snapshot);
        self.next_snapshot = self.next_snapshot.saturating_add(1);
        self.snapshots.insert(tip, payload.clone());
        self.branches.insert(branch_id, tip);
        self.current_branch = branch_id;
        self.apply_payload(&payload);
        self.restored_event_log.clone_from(&payload.event_log);
        Ok(branch_id)
    }

    fn restore(&mut self, id: SnapshotId) -> Result<()> {
        let payload = self
            .snapshots
            .get(&id)
            .ok_or(BiomimicryError::SnapshotUnknown(id))?
            .clone();
        self.apply_payload(&payload);
        self.restored_event_log = payload.event_log;
        Ok(())
    }
}

mod durable {
    #![allow(clippy::cast_possible_truncation)]

    use super::*;
    use crate::causality::{CausalEdgeKind, CausalEvent, CausalStamp};
    use crate::cell::CellId;
    use crate::genesis::{
        DimensionVector, Directionality, EndpointPolarity, EndpointRef, HyperedgeKind, Primitive,
        Role,
    };
    use crate::signal::{Scope, SignalId};

    const MAGIC: &[u8; 4] = b"BM7S";
    const VERSION: u32 = 1;

    pub(super) fn encode(store: &MemoryStore) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(MAGIC);
        write_u32(&mut buf, VERSION);
        write_live(&mut buf, store);
        write_u64(&mut buf, store.snapshots.len() as u64);
        for (id, payload) in &store.snapshots {
            write_u64(&mut buf, id.0);
            write_payload(&mut buf, payload);
        }
        write_u64(&mut buf, store.branches.len() as u64);
        for (bid, sid) in &store.branches {
            write_u64(&mut buf, bid.0);
            write_u64(&mut buf, sid.0);
        }
        write_u64(&mut buf, store.current_branch.0);
        write_u64(&mut buf, store.next_snapshot);
        write_u64(&mut buf, store.next_branch);
        buf
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<MemoryStore> {
        let mut r = Reader::new(bytes);
        let magic = r.read_exact(4)?;
        if magic != MAGIC {
            return Err(BiomimicryError::Substrate("bad durable magic".into()));
        }
        let version = r.u32()?;
        if version != VERSION {
            return Err(BiomimicryError::Substrate(format!(
                "unsupported durable version {version}"
            )));
        }
        let mut store = MemoryStore::new();
        read_live(&mut r, &mut store)?;
        let n_snaps = r.u64()? as usize;
        for _ in 0..n_snaps {
            let id = SnapshotId(r.u64()?);
            let payload = read_payload(&mut r)?;
            store.snapshots.insert(id, payload);
        }
        let n_branches = r.u64()? as usize;
        for _ in 0..n_branches {
            let bid = BranchId(r.u64()?);
            let sid = SnapshotId(r.u64()?);
            store.branches.insert(bid, sid);
        }
        store.current_branch = BranchId(r.u64()?);
        store.next_snapshot = r.u64()?;
        store.next_branch = r.u64()?;
        Ok(store)
    }

    fn write_live(buf: &mut Vec<u8>, store: &MemoryStore) {
        write_u64(buf, store.nodes.len() as u64);
        for node in store.nodes.values() {
            write_node(buf, node);
        }
        write_u64(buf, store.hyperedges.len() as u64);
        for edge in &store.hyperedges {
            write_hyperedge(buf, edge);
        }
        write_dag(buf, &store.causal);
        write_opt_log(buf, store.event_log.as_ref());
    }

    fn read_live(r: &mut Reader<'_>, store: &mut MemoryStore) -> Result<()> {
        let n_nodes = r.u64()? as usize;
        for _ in 0..n_nodes {
            let node = read_node(r)?;
            store.nodes.insert(node.id, node);
        }
        let n_edges = r.u64()? as usize;
        for _ in 0..n_edges {
            store.hyperedges.push(read_hyperedge(r)?);
        }
        store.causal = read_dag(r)?;
        store.event_log = read_opt_log(r)?;
        Ok(())
    }

    fn write_payload(buf: &mut Vec<u8>, payload: &SnapshotPayload) {
        write_u64(buf, payload.nodes.len() as u64);
        for node in payload.nodes.values() {
            write_node(buf, node);
        }
        write_u64(buf, payload.hyperedges.len() as u64);
        for edge in &payload.hyperedges {
            write_hyperedge(buf, edge);
        }
        write_dag(buf, &payload.causal);
        write_opt_log(buf, payload.event_log.as_ref());
    }

    fn read_payload(r: &mut Reader<'_>) -> Result<SnapshotPayload> {
        let mut nodes = BTreeMap::new();
        let n_nodes = r.u64()? as usize;
        for _ in 0..n_nodes {
            let node = read_node(r)?;
            nodes.insert(node.id, node);
        }
        let mut hyperedges = Vec::new();
        let n_edges = r.u64()? as usize;
        for _ in 0..n_edges {
            hyperedges.push(read_hyperedge(r)?);
        }
        let causal = read_dag(r)?;
        let event_log = read_opt_log(r)?;
        Ok(SnapshotPayload {
            nodes,
            hyperedges,
            causal,
            event_log,
        })
    }

    fn write_dag(buf: &mut Vec<u8>, dag: &CausalDag) {
        write_u64(buf, dag.len() as u64);
        for n in dag.nodes() {
            write_i64(buf, n.stamp.0);
            write_u64(buf, n.predecessors.len() as u64);
            for p in &n.predecessors {
                write_i64(buf, p.0);
            }
            write_u8(
                buf,
                match n.kind {
                    CausalEdgeKind::Single => 0,
                    CausalEdgeKind::Conjunction => 1,
                    CausalEdgeKind::Disjunction => 2,
                },
            );
            write_u128(buf, n.signal_id.0);
            write_str(buf, &n.tag);
        }
    }

    fn read_dag(r: &mut Reader<'_>) -> Result<CausalDag> {
        let mut dag = CausalDag::new();
        let n = r.u64()? as usize;
        for _ in 0..n {
            let stamp = CausalStamp(r.i64()?);
            let n_pred = r.u64()? as usize;
            let mut predecessors = Vec::with_capacity(n_pred);
            for _ in 0..n_pred {
                predecessors.push(CausalStamp(r.i64()?));
            }
            let kind = match r.u8()? {
                0 => CausalEdgeKind::Single,
                1 => CausalEdgeKind::Conjunction,
                2 => CausalEdgeKind::Disjunction,
                other => {
                    return Err(BiomimicryError::Substrate(format!("bad edge kind {other}")));
                }
            };
            let signal_id = SignalId(r.u128()?);
            let tag = r.str()?;
            dag.append(CausalNode {
                stamp,
                predecessors,
                kind,
                signal_id,
                tag,
            });
        }
        Ok(dag)
    }

    fn write_opt_log(buf: &mut Vec<u8>, log: Option<&CausalEventLog>) {
        match log {
            None => write_u8(buf, 0),
            Some(log) => {
                write_u8(buf, 1);
                write_u64(buf, log.len() as u64);
                for e in log.events() {
                    write_u8(buf, u8::from(e.parent.is_some()));
                    if let Some(p) = e.parent {
                        write_u128(buf, p.0);
                    }
                    write_u128(buf, e.child.0);
                    write_u64(buf, e.cell.0);
                    write_i64(buf, e.stamp.0);
                    write_str(buf, &e.tag);
                }
            }
        }
    }

    fn read_opt_log(r: &mut Reader<'_>) -> Result<Option<CausalEventLog>> {
        match r.u8()? {
            0 => Ok(None),
            1 => {
                let mut log = CausalEventLog::new();
                let n = r.u64()? as usize;
                for _ in 0..n {
                    let has_parent = r.u8()? != 0;
                    let parent = if has_parent {
                        Some(SignalId(r.u128()?))
                    } else {
                        None
                    };
                    let child = SignalId(r.u128()?);
                    let cell = CellId(r.u64()?);
                    let stamp = CausalStamp(r.i64()?);
                    let tag = r.str()?;
                    log.push(CausalEvent {
                        parent,
                        child,
                        cell,
                        stamp,
                        tag,
                    });
                }
                Ok(Some(log))
            }
            other => Err(BiomimicryError::Substrate(format!(
                "bad opt log tag {other}"
            ))),
        }
    }

    fn write_node(buf: &mut Vec<u8>, node: &PrimitiveNode) {
        write_u128(buf, node.id.0);
        write_u32(buf, node.primitive.type_id());
        write_u64(buf, node.coord.len() as u64);
        for &c in node.coord.as_slice() {
            write_i32(buf, c);
        }
    }

    fn read_node(r: &mut Reader<'_>) -> Result<PrimitiveNode> {
        let id = PrimitiveNodeId(r.u128()?);
        let primitive = Primitive::from_type_id(r.u32()?)
            .ok_or_else(|| BiomimicryError::Substrate("bad primitive type id".into()))?;
        let n = r.u64()? as usize;
        let mut comps = Vec::with_capacity(n);
        for _ in 0..n {
            comps.push(r.i32()?);
        }
        Ok(PrimitiveNode {
            id,
            primitive,
            coord: DimensionVector::new(comps),
        })
    }

    fn write_hyperedge(buf: &mut Vec<u8>, edge: &Hyperedge) {
        write_str(buf, edge.kind.as_str());
        write_u64(buf, edge.endpoints.len() as u64);
        for ep in &edge.endpoints {
            write_endpoint(buf, ep);
        }
        match edge.weight_milli {
            None => write_u8(buf, 0),
            Some(w) => {
                write_u8(buf, 1);
                write_i32(buf, w);
            }
        }
        write_u8(
            buf,
            match edge.directionality {
                Directionality::Directed => 0,
                Directionality::Undirected => 1,
            },
        );
    }

    fn read_hyperedge(r: &mut Reader<'_>) -> Result<Hyperedge> {
        let kind = HyperedgeKind::new(r.str()?);
        let n = r.u64()? as usize;
        let mut endpoints = Vec::with_capacity(n);
        for _ in 0..n {
            endpoints.push(read_endpoint(r)?);
        }
        let weight_milli = match r.u8()? {
            0 => None,
            1 => Some(r.i32()?),
            other => {
                return Err(BiomimicryError::Substrate(format!(
                    "bad weight tag {other}"
                )));
            }
        };
        let directionality = match r.u8()? {
            0 => Directionality::Directed,
            1 => Directionality::Undirected,
            other => {
                return Err(BiomimicryError::Substrate(format!(
                    "bad directionality {other}"
                )));
            }
        };
        Ok(Hyperedge {
            kind,
            endpoints,
            weight_milli,
            directionality,
        })
    }

    fn write_endpoint(buf: &mut Vec<u8>, ep: &EndpointRef) {
        write_u128(buf, ep.node.0);
        write_u32(buf, ep.primitive.type_id());
        write_u8(buf, ep.polarity as u8);
        write_str(buf, ep.role.as_str());
        match ep.scope {
            None => write_u8(buf, 0),
            Some(s) => {
                write_u8(buf, 1);
                write_u8(buf, s.wire_tag());
            }
        }
    }

    fn read_endpoint(r: &mut Reader<'_>) -> Result<EndpointRef> {
        let node = PrimitiveNodeId(r.u128()?);
        let primitive = Primitive::from_type_id(r.u32()?)
            .ok_or_else(|| BiomimicryError::Substrate("bad endpoint primitive".into()))?;
        let polarity = match r.u8()? {
            0 => EndpointPolarity::Positive,
            1 => EndpointPolarity::Negative,
            other => {
                return Err(BiomimicryError::Substrate(format!("bad polarity {other}")));
            }
        };
        let role = Role::new(r.str()?);
        let scope = match r.u8()? {
            0 => None,
            1 => Some(
                Scope::from_wire_tag(r.u8()?)
                    .ok_or_else(|| BiomimicryError::Substrate("bad scope wire tag".into()))?,
            ),
            other => {
                return Err(BiomimicryError::Substrate(format!("bad scope tag {other}")));
            }
        };
        Ok(EndpointRef::new(node, primitive, polarity, role, scope))
    }

    struct Reader<'a> {
        buf: &'a [u8],
        pos: usize,
    }

    impl<'a> Reader<'a> {
        fn new(buf: &'a [u8]) -> Self {
            Self { buf, pos: 0 }
        }

        fn read_exact(&mut self, n: usize) -> Result<&'a [u8]> {
            if self.pos + n > self.buf.len() {
                return Err(BiomimicryError::Substrate("durable truncated".into()));
            }
            let slice = &self.buf[self.pos..self.pos + n];
            self.pos += n;
            Ok(slice)
        }

        fn u8(&mut self) -> Result<u8> {
            Ok(self.read_exact(1)?[0])
        }

        fn u32(&mut self) -> Result<u32> {
            let b = self.read_exact(4)?;
            Ok(u32::from_le_bytes(b.try_into().expect("4")))
        }

        fn u64(&mut self) -> Result<u64> {
            let b = self.read_exact(8)?;
            Ok(u64::from_le_bytes(b.try_into().expect("8")))
        }

        fn i64(&mut self) -> Result<i64> {
            let b = self.read_exact(8)?;
            Ok(i64::from_le_bytes(b.try_into().expect("8")))
        }

        fn i32(&mut self) -> Result<i32> {
            let b = self.read_exact(4)?;
            Ok(i32::from_le_bytes(b.try_into().expect("4")))
        }

        fn u128(&mut self) -> Result<u128> {
            let b = self.read_exact(16)?;
            Ok(u128::from_le_bytes(b.try_into().expect("16")))
        }

        fn str(&mut self) -> Result<String> {
            let len = self.u64()? as usize;
            let b = self.read_exact(len)?;
            String::from_utf8(b.to_vec())
                .map_err(|e| BiomimicryError::Substrate(format!("utf8: {e}")))
        }
    }

    fn write_u8(buf: &mut Vec<u8>, v: u8) {
        buf.push(v);
    }
    fn write_u32(buf: &mut Vec<u8>, v: u32) {
        buf.extend_from_slice(&v.to_le_bytes());
    }
    fn write_u64(buf: &mut Vec<u8>, v: u64) {
        buf.extend_from_slice(&v.to_le_bytes());
    }
    fn write_i64(buf: &mut Vec<u8>, v: i64) {
        buf.extend_from_slice(&v.to_le_bytes());
    }
    fn write_i32(buf: &mut Vec<u8>, v: i32) {
        buf.extend_from_slice(&v.to_le_bytes());
    }
    fn write_u128(buf: &mut Vec<u8>, v: u128) {
        buf.extend_from_slice(&v.to_le_bytes());
    }
    fn write_str(buf: &mut Vec<u8>, s: &str) {
        write_u64(buf, s.len() as u64);
        buf.extend_from_slice(s.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causality::{CausalEdgeKind, CausalStamp};
    use crate::genesis::Hypergraph;
    use crate::signal::SignalId;

    #[test]
    fn memory_store_round_trips_empty_hypergraph() {
        let mut store = MemoryStore::new();
        let hg = Hypergraph::new();
        store.save_hypergraph(&hg).expect("save");
        let loaded = store.load_hypergraph().expect("load");
        assert!(loaded.edges().next().is_none());
    }

    #[test]
    fn memory_store_issues_snapshot_ids() {
        let mut store = MemoryStore::new();
        let a = store.snapshot("a").expect("snap a");
        let b = store.snapshot("b").expect("snap b");
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn memory_store_appends_causal_nodes() {
        let mut store = MemoryStore::new();
        store
            .append_causal(CausalNode::leaf(CausalStamp(1), SignalId(1), "t"))
            .expect("append");
        let dag = store.load_causal_dag().expect("load dag");
        assert_eq!(dag.len(), 1);
    }

    #[test]
    fn p2_restore_unknown_id_errors() {
        let mut store = MemoryStore::new();
        let err = store.restore(SnapshotId(99)).expect_err("unknown");
        assert!(matches!(err, BiomimicryError::SnapshotUnknown(_)));
    }

    #[test]
    fn snapshot_restore_retains_causal() {
        let mut store = MemoryStore::new();
        store
            .append_causal(CausalNode {
                stamp: CausalStamp(7),
                predecessors: Vec::new(),
                kind: CausalEdgeKind::Single,
                signal_id: SignalId(7),
                tag: "x".into(),
            })
            .unwrap();
        let meta = store.snapshot("s").unwrap();
        store.causal = CausalDag::new();
        store.restore(meta.id).unwrap();
        assert_eq!(store.load_causal_dag().unwrap().len(), 1);
    }

    #[test]
    fn durable_round_trip() {
        let mut store = MemoryStore::new();
        store
            .append_causal(CausalNode::leaf(CausalStamp(3), SignalId(9), "emit"))
            .unwrap();
        let bytes = store.to_durable_bytes().unwrap();
        let loaded = MemoryStore::from_durable_bytes(&bytes).unwrap();
        assert_eq!(loaded.load_causal_dag().unwrap().len(), 1);
    }
}
