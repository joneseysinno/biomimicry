//! [`EffectorSink`] trait — write target outside the signal stream.

use std::collections::BTreeMap;

use crate::effector::EffectorId;
use crate::error::Result;
use crate::signal::{CausalStamp, Value};

/// Sink for effector writes (working state).
///
/// Durable persistence is typed-deferred
/// ([`crate::BiomimicryError::SinkPersistenceUnavailable`]).
pub trait EffectorSink: Send {
    /// Write `value` under `id` at `stamp` (working state).
    ///
    /// # Errors
    ///
    /// Implementation-defined; memory sink is infallible.
    fn write(&mut self, id: EffectorId, value: Value, stamp: CausalStamp) -> Result<()>;

    /// Read the current working value for `id`.
    fn read(&self, id: EffectorId) -> Option<&Value>;

    /// Snapshot of working state (deterministic `BTreeMap` iteration).
    fn snapshot(&self) -> BTreeMap<EffectorId, Value>;
}
