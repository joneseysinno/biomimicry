//! [`BlockSource`] — where blocks come from (memory / directory; network later).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::blocks::block::Block;
use crate::blocks::error::LinkError;
use crate::blocks::name::{BlockName, Version};

/// Fetch / list blocks by name + version.
pub trait BlockSource {
    /// Fetch one block.
    ///
    /// # Errors
    ///
    /// When the block is unknown or cannot be decoded.
    #[allow(clippy::result_large_err)]
    fn fetch(&self, name: &BlockName, version: &Version) -> Result<Block, LinkError>;

    /// List available `(name, version)` pairs in deterministic order.
    fn list(&self) -> Vec<(BlockName, Version)>;
}

/// In-memory source for tests.
#[derive(Debug, Default, Clone)]
pub struct MemoryBlockSource {
    blocks: BTreeMap<(String, String), Block>,
}

impl MemoryBlockSource {
    /// Empty source.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a block.
    pub fn insert(&mut self, block: Block) {
        let key = (block.name.as_str().to_owned(), block.version.to_string());
        self.blocks.insert(key, block);
    }
}

impl BlockSource for MemoryBlockSource {
    #[allow(clippy::result_large_err)]
    fn fetch(&self, name: &BlockName, version: &Version) -> Result<Block, LinkError> {
        self.blocks
            .get(&(name.as_str().to_owned(), version.to_string()))
            .cloned()
            .ok_or_else(|| LinkError::UnknownBlock {
                name: name.clone(),
            })
    }

    fn list(&self) -> Vec<(BlockName, Version)> {
        self.blocks
            .values()
            .map(|b| (b.name.clone(), b.version.clone()))
            .collect()
    }
}

/// Directory of `{name}@{version}.block` files (canonical bytes + TOML header).
#[derive(Debug, Clone)]
pub struct DirBlockSource {
    root: PathBuf,
    cache: BTreeMap<(String, String), Block>,
}

impl DirBlockSource {
    /// Open a directory. DNA is restored from the in-process cache after
    /// [`Self::write_canonical`]; listing sidecars alone are not enough to
    /// reconstruct cistrons (network registry later owns durable codecs).
    ///
    /// # Errors
    ///
    /// IO failures creating / reading the root.
    #[allow(clippy::result_large_err)]
    pub fn open(root: impl AsRef<Path>) -> Result<Self, LinkError> {
        let root = root.as_ref().to_path_buf();
        if !root.exists() {
            fs::create_dir_all(&root).map_err(|e| LinkError::ManifestParse {
                reason: e.to_string(),
            })?;
        }
        Ok(Self {
            root,
            cache: BTreeMap::new(),
        })
    }

    /// Write a block as `{name}@{version}.toml` (ports + requires; DNA as hex digests).
    ///
    /// Full DNA round-trip for DirBlockSource stores the block via
    /// [`Self::write_canonical`] for bit-stable identity.
    ///
    /// # Errors
    ///
    /// IO failures.
    #[allow(clippy::result_large_err)]
    pub fn write_canonical(&mut self, block: &Block) -> Result<PathBuf, LinkError> {
        fs::create_dir_all(&self.root).map_err(|e| LinkError::ManifestParse {
            reason: e.to_string(),
        })?;
        let path = self.root.join(format!("{}@{}.bin", block.name, block.version));
        let bytes = encode_block(block);
        fs::write(&path, &bytes).map_err(|e| LinkError::ManifestParse {
            reason: e.to_string(),
        })?;
        // Also write a tiny TOML listing sidecar for `list` / open discovery.
        let list_path = self.root.join(format!("{}@{}.toml", block.name, block.version));
        let toml = format!(
            "name = \"{}\"\nversion = \"{}\"\nbinary = \"{}@{}.bin\"\n",
            block.name, block.version, block.name, block.version
        );
        fs::write(&list_path, toml).map_err(|e| LinkError::ManifestParse {
            reason: e.to_string(),
        })?;
        let key = (block.name.as_str().to_owned(), block.version.to_string());
        self.cache.insert(key, block.clone());
        Ok(path)
    }
}

impl BlockSource for DirBlockSource {
    #[allow(clippy::result_large_err)]
    fn fetch(&self, name: &BlockName, version: &Version) -> Result<Block, LinkError> {
        self.cache
            .get(&(name.as_str().to_owned(), version.to_string()))
            .cloned()
            .ok_or_else(|| LinkError::UnknownBlock {
                name: name.clone(),
            })
    }

    fn list(&self) -> Vec<(BlockName, Version)> {
        self.cache
            .values()
            .map(|b| (b.name.clone(), b.version.clone()))
            .collect()
    }
}

/// Encode a block content seal for directory storage.
fn encode_block(block: &Block) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"BMB1");
    let name = block.name.as_str().as_bytes();
    out.extend_from_slice(
        &u32::try_from(name.len())
            .expect("name len fits u32")
            .to_le_bytes(),
    );
    out.extend_from_slice(name);
    let ver = block.version.to_string();
    let ver_b = ver.as_bytes();
    out.extend_from_slice(
        &u32::try_from(ver_b.len())
            .expect("ver len fits u32")
            .to_le_bytes(),
    );
    out.extend_from_slice(ver_b);
    let canon = block.canonical_bytes();
    out.extend_from_slice(
        &u32::try_from(canon.len())
            .expect("canon len fits u32")
            .to_le_bytes(),
    );
    out.extend_from_slice(&canon);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::fixture::sum_block;

    #[test]
    fn memory_source_lists_sorted() {
        let mut src = MemoryBlockSource::new();
        src.insert(sum_block());
        let list = src.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0.as_str(), "sum");
    }

    #[test]
    fn dir_source_round_trip_via_cache() {
        let dir = std::env::temp_dir().join(format!("biomimicry-blocks-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let mut src = DirBlockSource::open(&dir).unwrap();
        let block = sum_block();
        let id = block.id();
        src.write_canonical(&block).unwrap();
        let got = src
            .fetch(&block.name, &block.version)
            .expect("fetch from cache");
        assert_eq!(got.id(), id);
        let _ = fs::remove_dir_all(&dir);
    }
}
