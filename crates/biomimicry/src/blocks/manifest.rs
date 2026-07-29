//! Organism manifest — exact pins, optional explicit wires, genotype.

use std::collections::BTreeMap;

use blake3::Hasher;
use serde::{Deserialize, Serialize};

use crate::blocks::error::LinkError;
use crate::blocks::name::{BlockId, BlockName, Version};
use crate::blocks::port_spec::LocalKind;
use crate::genesis::hash::{finalize_u128, update_str, update_u32};

/// Exact version pin for one block in a composition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pin {
    /// Block name.
    pub name: BlockName,
    /// Exact version.
    pub version: Version,
}

impl Pin {
    /// Construct a pin.
    #[must_use]
    pub fn new(name: impl Into<BlockName>, version: Version) -> Self {
        Self {
            name: name.into(),
            version,
        }
    }
}

/// Explicit wire when inference is ambiguous or the author wants to be precise.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExplicitWire {
    /// Exporting block.
    pub from_block: BlockName,
    /// Export local kind.
    pub from_kind: LocalKind,
    /// Importing block.
    pub to_block: BlockName,
    /// Import local kind.
    pub to_kind: LocalKind,
}

/// Local-kind rename applied before resolution (rarely needed).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rename {
    /// Block whose local kind is renamed.
    pub block: BlockName,
    /// Old local kind.
    pub from: LocalKind,
    /// New local kind.
    pub to: LocalKind,
}

/// Application composition: exact pins + optional wires/renames.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Manifest {
    /// Exact block pins (order irrelevant — genotype sorts by [`BlockId`]).
    pub blocks: Vec<Pin>,
    /// Explicit wires (applied after / instead of inference when present).
    pub wires: Vec<ExplicitWire>,
    /// Optional local renames.
    pub renames: Vec<Rename>,
    /// Optional per-ganglion capacity override (default 8).
    pub ganglion_capacity: Option<u32>,
}

impl Manifest {
    /// Empty manifest.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: set pins.
    #[must_use]
    pub fn with_blocks(mut self, blocks: Vec<Pin>) -> Self {
        self.blocks = blocks;
        self
    }

    /// Builder: set explicit wires.
    #[must_use]
    pub fn with_wires(mut self, wires: Vec<ExplicitWire>) -> Self {
        self.wires = wires;
        self
    }

    /// Parse from TOML.
    ///
    /// # Errors
    ///
    /// Returns [`LinkError::ManifestParse`] on invalid TOML / fields / semver.
    #[allow(clippy::result_large_err)]
    pub fn from_toml(text: &str) -> Result<Self, LinkError> {
        let dto: ManifestDto = toml::from_str(text).map_err(|e| LinkError::ManifestParse {
            reason: e.to_string(),
        })?;
        dto.into_manifest()
    }

    /// Serialise to TOML.
    ///
    /// # Errors
    ///
    /// Returns [`LinkError::ManifestParse`] when serialisation fails.
    #[allow(clippy::result_large_err)]
    pub fn to_toml(&self) -> Result<String, LinkError> {
        let dto = ManifestDto::from_manifest(self);
        toml::to_string_pretty(&dto).map_err(|e| LinkError::ManifestParse {
            reason: e.to_string(),
        })
    }

    /// Canonical bytes for [`OrganismGenotype`] (pins sorted by name, then version).
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut hasher = Hasher::new();
        update_str(&mut hasher, "manifest");
        let mut pins = self.blocks.clone();
        pins.sort();
        update_u32(
            &mut hasher,
            u32::try_from(pins.len()).expect("pin count fits u32"),
        );
        for p in &pins {
            update_str(&mut hasher, p.name.as_str());
            update_str(&mut hasher, &p.version.to_string());
        }
        update_u32(
            &mut hasher,
            u32::try_from(self.wires.len()).expect("wire count fits u32"),
        );
        let mut wires = self.wires.clone();
        wires.sort_by(|a, b| {
            (&a.from_block, &a.from_kind, &a.to_block, &a.to_kind).cmp(&(
                &b.from_block,
                &b.from_kind,
                &b.to_block,
                &b.to_kind,
            ))
        });
        for w in &wires {
            update_str(&mut hasher, w.from_block.as_str());
            update_str(&mut hasher, w.from_kind.as_str());
            update_str(&mut hasher, w.to_block.as_str());
            update_str(&mut hasher, w.to_kind.as_str());
        }
        update_u32(
            &mut hasher,
            u32::try_from(self.renames.len()).expect("rename count fits u32"),
        );
        for r in &self.renames {
            update_str(&mut hasher, r.block.as_str());
            update_str(&mut hasher, r.from.as_str());
            update_str(&mut hasher, r.to.as_str());
        }
        if let Some(cap) = self.ganglion_capacity {
            hasher.update(&[1u8]);
            update_u32(&mut hasher, cap);
        } else {
            hasher.update(&[0u8]);
        }
        finalize_u128(&hasher).to_le_bytes().to_vec()
    }
}

/// Content-addressed organism identity: `BLAKE3₁₂₈(sorted BlockIds ‖ canonical manifest)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OrganismGenotype(pub u128);

impl OrganismGenotype {
    /// Compute from linked block ids and the manifest.
    #[must_use]
    pub fn of(block_ids: &[BlockId], manifest: &Manifest) -> Self {
        let mut ids = block_ids.to_vec();
        ids.sort();
        let mut hasher = Hasher::new();
        update_str(&mut hasher, "organism_genotype");
        update_u32(
            &mut hasher,
            u32::try_from(ids.len()).expect("id count fits u32"),
        );
        for id in ids {
            hasher.update(&id.0.to_le_bytes());
        }
        hasher.update(&manifest.canonical_bytes());
        Self(finalize_u128(&hasher))
    }
}

impl std::fmt::Display for OrganismGenotype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ManifestDto {
    #[serde(default)]
    blocks: Vec<PinDto>,
    #[serde(default)]
    wires: Vec<WireDto>,
    #[serde(default)]
    renames: Vec<RenameDto>,
    #[serde(default)]
    ganglion_capacity: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PinDto {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct WireDto {
    from_block: String,
    from_kind: String,
    to_block: String,
    to_kind: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RenameDto {
    block: String,
    from: String,
    to: String,
}

impl ManifestDto {
    #[allow(clippy::result_large_err)]
    fn into_manifest(self) -> Result<Manifest, LinkError> {
        let mut blocks = Vec::with_capacity(self.blocks.len());
        for p in self.blocks {
            let version = Version::parse(&p.version).map_err(|e| LinkError::ManifestParse {
                reason: format!("bad semver for block {}: {e}", p.name),
            })?;
            blocks.push(Pin::new(p.name.as_str(), version));
        }
        let wires = self
            .wires
            .into_iter()
            .map(|w| ExplicitWire {
                from_block: BlockName::new(w.from_block),
                from_kind: LocalKind::new(w.from_kind),
                to_block: BlockName::new(w.to_block),
                to_kind: LocalKind::new(w.to_kind),
            })
            .collect();
        let renames = self
            .renames
            .into_iter()
            .map(|r| Rename {
                block: BlockName::new(r.block),
                from: LocalKind::new(r.from),
                to: LocalKind::new(r.to),
            })
            .collect();
        Ok(Manifest {
            blocks,
            wires,
            renames,
            ganglion_capacity: self.ganglion_capacity,
        })
    }

    fn from_manifest(m: &Manifest) -> Self {
        Self {
            blocks: m
                .blocks
                .iter()
                .map(|p| PinDto {
                    name: p.name.as_str().into(),
                    version: p.version.to_string(),
                })
                .collect(),
            wires: m
                .wires
                .iter()
                .map(|w| WireDto {
                    from_block: w.from_block.as_str().into(),
                    from_kind: w.from_kind.as_str().into(),
                    to_block: w.to_block.as_str().into(),
                    to_kind: w.to_kind.as_str().into(),
                })
                .collect(),
            renames: m
                .renames
                .iter()
                .map(|r| RenameDto {
                    block: r.block.as_str().into(),
                    from: r.from.as_str().into(),
                    to: r.to.as_str().into(),
                })
                .collect(),
            ganglion_capacity: m.ganglion_capacity,
        }
    }
}

/// Look up a pin by name.
#[must_use]
pub fn pin_map(manifest: &Manifest) -> BTreeMap<BlockName, Version> {
    manifest
        .blocks
        .iter()
        .map(|p| (p.name.clone(), p.version.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_round_trip() {
        let m = Manifest::new().with_blocks(vec![
            Pin::new("sum", Version::parse("1.0.0").unwrap()),
            Pin::new("scale", Version::parse("1.0.0").unwrap()),
        ]);
        let text = m.to_toml().unwrap();
        let back = Manifest::from_toml(&text).unwrap();
        assert_eq!(m.blocks, back.blocks);
    }
}
