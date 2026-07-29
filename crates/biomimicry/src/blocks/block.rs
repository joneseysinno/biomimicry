//! [`Block`] — a named DNA fragment plus declared signal-kind surface.

use blake3::Hasher;

use crate::blocks::name::{BlockId, BlockName, BlockReq, Version};
use crate::blocks::port_spec::PortSpec;
use crate::genesis::hash::{finalize_u128, update_i32, update_str, update_u32};
use crate::genesis::{Cistron, PrimitiveNode};
use crate::signal::{Scope, ValueShape};

/// A composable DNA fragment: nodes + cistrons + typed import/export surface.
///
/// Blocks are tissue types, not libraries — there is no `run` / `call` / `invoke`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// Block name (`"sum"`, `"structural"`, …).
    pub name: BlockName,
    /// Exact semver of this fragment.
    pub version: Version,
    /// Fragment-local primitive pool (relocated at link).
    pub nodes: Vec<PrimitiveNode>,
    /// Behavioural fragment (transduction specs included).
    pub cistrons: Vec<Cistron>,
    /// Kinds this block expects to receive.
    pub imports: Vec<PortSpec>,
    /// Kinds this block promises to emit.
    pub exports: Vec<PortSpec>,
    /// Other blocks required by name + version range.
    pub requires: Vec<BlockReq>,
}

impl Block {
    /// Construct an empty block shell (no DNA yet).
    #[must_use]
    pub fn new(name: impl Into<BlockName>, version: Version) -> Self {
        Self {
            name: name.into(),
            version,
            nodes: Vec::new(),
            cistrons: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            requires: Vec::new(),
        }
    }

    /// Builder: replace nodes.
    #[must_use]
    pub fn with_nodes(mut self, nodes: Vec<PrimitiveNode>) -> Self {
        self.nodes = nodes;
        self
    }

    /// Builder: replace cistrons.
    #[must_use]
    pub fn with_cistrons(mut self, cistrons: Vec<Cistron>) -> Self {
        self.cistrons = cistrons;
        self
    }

    /// Builder: replace imports.
    #[must_use]
    pub fn with_imports(mut self, imports: Vec<PortSpec>) -> Self {
        self.imports = imports;
        self
    }

    /// Builder: replace exports.
    #[must_use]
    pub fn with_exports(mut self, exports: Vec<PortSpec>) -> Self {
        self.exports = exports;
        self
    }

    /// Builder: replace requires.
    #[must_use]
    pub fn with_requires(mut self, requires: Vec<BlockReq>) -> Self {
        self.requires = requires;
        self
    }

    /// Content-addressed identity — invariant under linkage.
    #[must_use]
    pub fn id(&self) -> BlockId {
        BlockId::from_canonical(&self.canonical_bytes())
    }

    /// Deterministic canonical byte encoding (domain-separated).
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut hasher = Hasher::new();
        update_str(&mut hasher, self.name.as_str());
        update_str(&mut hasher, &self.version.to_string());

        let mut nodes = self.nodes.clone();
        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        update_u32(
            &mut hasher,
            u32::try_from(nodes.len()).expect("node count fits u32"),
        );
        for n in &nodes {
            hasher.update(&n.id.0.to_le_bytes());
            update_u32(&mut hasher, n.primitive.type_id());
            update_u32(
                &mut hasher,
                u32::try_from(n.coord.len()).expect("coord len fits u32"),
            );
            for &c in n.coord.as_slice() {
                update_i32(&mut hasher, c);
            }
        }

        let mut cistron_ids: Vec<u128> = self.cistrons.iter().map(Cistron::content_id).collect();
        cistron_ids.sort_unstable();
        update_u32(
            &mut hasher,
            u32::try_from(cistron_ids.len()).expect("cistron count fits u32"),
        );
        for id in cistron_ids {
            hasher.update(&id.to_le_bytes());
        }

        hash_ports(&mut hasher, &self.imports);
        hash_ports(&mut hasher, &self.exports);

        let mut reqs = self.requires.clone();
        reqs.sort_by(|a, b| {
            a.name
                .cmp(&b.name)
                .then_with(|| a.range.as_str().cmp(&b.range.as_str()))
        });
        update_u32(
            &mut hasher,
            u32::try_from(reqs.len()).expect("req count fits u32"),
        );
        for r in &reqs {
            update_str(&mut hasher, r.name.as_str());
            update_str(&mut hasher, &r.range.as_str());
        }

        let digest = finalize_u128(&hasher);
        digest.to_le_bytes().to_vec()
    }
}

fn hash_ports(hasher: &mut Hasher, ports: &[PortSpec]) {
    update_u32(
        hasher,
        u32::try_from(ports.len()).expect("port count fits u32"),
    );
    for p in ports {
        update_str(hasher, p.local_kind.as_str());
        hash_shape(hasher, &p.shape);
        hasher.update(&[scope_tag(p.scope)]);
        hasher.update(&[u8::from(p.optional)]);
    }
}

fn scope_tag(scope: Scope) -> u8 {
    scope.wire_tag()
}

fn hash_shape(hasher: &mut Hasher, shape: &ValueShape) {
    match shape {
        ValueShape::Unit => {
            hasher.update(&[0u8]);
        }
        ValueShape::Bool => {
            hasher.update(&[1u8]);
        }
        ValueShape::Int => {
            hasher.update(&[2u8]);
        }
        ValueShape::Text => {
            hasher.update(&[3u8]);
        }
        ValueShape::List(inner) => {
            hasher.update(&[4u8]);
            hash_shape(hasher, inner);
        }
        ValueShape::Record(fields) => {
            hasher.update(&[5u8]);
            update_u32(
                hasher,
                u32::try_from(fields.len()).expect("field count fits u32"),
            );
            for (k, v) in fields {
                update_str(hasher, k.as_str());
                hash_shape(hasher, v);
            }
        }
        ValueShape::Bytes => {
            hasher.update(&[6u8]);
        }
        ValueShape::Any => {
            hasher.update(&[7u8]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::port_spec::PortSpec;
    use crate::signal::Scope;

    #[test]
    fn block_id_stable_across_calls() {
        let b = Block::new("sum", Version::parse("1.0.0").unwrap())
            .with_imports(vec![PortSpec::int("a")])
            .with_exports(vec![PortSpec::required(
                "total",
                crate::signal::ValueShape::Int,
                Scope::Cluster,
            )]);
        assert_eq!(b.id(), b.id());
        let again = Block::new("sum", Version::parse("1.0.0").unwrap())
            .with_imports(vec![PortSpec::int("a")])
            .with_exports(vec![PortSpec::required(
                "total",
                crate::signal::ValueShape::Int,
                Scope::Cluster,
            )]);
        assert_eq!(b.id(), again.id());
    }
}
