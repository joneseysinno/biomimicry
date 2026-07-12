//! Signal payload body and observation metadata.

use std::collections::BTreeMap;

use blake3::Hasher;
use smol_str::SmolStr;

use crate::genesis::hash::{finalize_u128, update_str, update_u32};

/// Typed metadata tag. [`Tag::OBSERVATION`] marks sensorium observation signals.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tag(pub SmolStr);

impl Tag {
    /// Observation signal marker (Part VII / M6 collector contract).
    pub const OBSERVATION: &'static str = "observation";

    /// Construct a tag.
    #[must_use]
    pub fn new(label: impl AsRef<str>) -> Self {
        Self(SmolStr::new(label.as_ref()))
    }

    /// Borrow the tag label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// The canonical observation tag.
    #[must_use]
    pub fn observation() -> Self {
        Self::new(Self::OBSERVATION)
    }
}

impl From<&str> for Tag {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Opaque metadata value (stringly for M2; typed decode is gene-defined).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MetaValue(pub SmolStr);

impl MetaValue {
    /// Construct from any stringy value.
    #[must_use]
    pub fn new(value: impl AsRef<str>) -> Self {
        Self(SmolStr::new(value.as_ref()))
    }

    /// Borrow the value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for MetaValue {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Payload body + metadata map carried by a [`super::Signal`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Payload {
    /// Opaque content bytes (typed interpretation is gene-defined).
    pub body: Vec<u8>,
    /// Optional tagged metadata (e.g. observation).
    pub metadata: BTreeMap<Tag, MetaValue>,
}

impl Payload {
    /// Construct from raw body bytes.
    #[must_use]
    pub fn new(body: impl Into<Vec<u8>>) -> Self {
        Self {
            body: body.into(),
            metadata: BTreeMap::new(),
        }
    }

    /// Empty payload.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Builder: insert a metadata entry.
    #[must_use]
    pub fn with_meta(mut self, tag: impl Into<Tag>, value: impl Into<MetaValue>) -> Self {
        self.metadata.insert(tag.into(), value.into());
        self
    }

    /// Mark as an observation signal (endocrine + observation tag contract).
    #[must_use]
    pub fn with_observation(self, note: impl AsRef<str>) -> Self {
        self.with_meta(Tag::observation(), MetaValue::new(note.as_ref()))
    }

    /// Whether the observation tag is present.
    #[must_use]
    pub fn is_observation(&self) -> bool {
        self.metadata.contains_key(&Tag::observation())
    }

    /// Content digest for [`super::SignalId`] hashing.
    #[must_use]
    pub fn digest(&self) -> u128 {
        let mut hasher = Hasher::new();
        update_u32(
            &mut hasher,
            u32::try_from(self.body.len()).expect("body length fits u32"),
        );
        hasher.update(&self.body);
        update_u32(
            &mut hasher,
            u32::try_from(self.metadata.len()).expect("metadata count fits u32"),
        );
        for (tag, value) in &self.metadata {
            update_str(&mut hasher, tag.as_str());
            update_str(&mut hasher, value.as_str());
        }
        finalize_u128(&hasher)
    }
}

/// Backward-compatible alias used by later-milestone stubs.
pub type SignalPayload = Payload;
