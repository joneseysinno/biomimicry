//! Signal payload body and observation metadata.
//!
//! [`Payload::body`] is the **canonical encoding** of a [`super::Value`].
//! Construct typed payloads via [`Payload::of`]; decode with [`Payload::value`].
//! [`Payload::digest`] hashes that encoding (never the in-memory enum layout).

use std::collections::BTreeMap;

use blake3::Hasher;
use smol_str::SmolStr;

use crate::error::Result;
use crate::genesis::hash::{finalize_u128, update_str, update_u32};
use crate::signal::Value;

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payload {
    /// Canonical encoding of [`Value`]; construct via [`Payload::of`].
    ///
    /// Public for the [`Value::Bytes`] escape hatch and wire compatibility.
    pub body: Vec<u8>,
    /// Optional tagged metadata (e.g. observation).
    pub metadata: BTreeMap<Tag, MetaValue>,
    /// Signal strength in milli-units (sensorium threshold gate; default 1000).
    pub strength_milli: u32,
}

impl Default for Payload {
    fn default() -> Self {
        // `Value::Unit` encoding — one hash authority for empty payloads.
        Self::of(Value::Unit)
    }
}

impl Payload {
    /// Construct a payload whose body is the canonical encoding of `value`.
    #[must_use]
    #[allow(clippy::needless_pass_by_value)] // owned Value is the natural construction API
    pub fn of(value: Value) -> Self {
        let body = value
            .encode()
            .expect("Value::of requires depth-checked values; Unit always encodes");
        Self {
            body,
            metadata: BTreeMap::new(),
            strength_milli: 1000,
        }
    }

    /// Construct from raw body bytes (prefer [`Payload::of`] for typed values).
    #[must_use]
    pub fn new(body: impl Into<Vec<u8>>) -> Self {
        Self {
            body: body.into(),
            metadata: BTreeMap::new(),
            strength_milli: 1000,
        }
    }

    /// Empty payload ([`Value::Unit`]).
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Decode [`Self::body`] as a [`Value`].
    pub fn value(&self) -> Result<Value> {
        if self.body.is_empty() {
            // Pre-0.3 raw empties still decode as Unit.
            return Ok(Value::Unit);
        }
        Value::decode(&self.body)
    }

    /// Builder: set strength milli.
    #[must_use]
    pub fn with_strength(mut self, strength_milli: u32) -> Self {
        self.strength_milli = strength_milli;
        self
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
    ///
    /// Hashes the canonical `body` encoding (plus strength and metadata) —
    /// never the in-memory [`Value`] layout — so there is one hash authority.
    #[must_use]
    pub fn digest(&self) -> u128 {
        let mut hasher = Hasher::new();
        update_u32(
            &mut hasher,
            u32::try_from(self.body.len()).expect("body length fits u32"),
        );
        hasher.update(&self.body);
        hasher.update(&self.strength_milli.to_le_bytes());
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
