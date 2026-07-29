//! Effector write path helpers (budget → sink → causal event).

use blake3::Hasher;

use crate::causality::CausalEvent;
use crate::cell::CellId;
use crate::effector::{EffectorId, EffectorSink};
use crate::error::Result;
use crate::genesis::hash::finalize_u128;
use crate::signal::{CausalStamp, SignalId, Value};

/// Perform an effector write and record a causal `effect` event.
///
/// Caller is responsible for Phase 2 budget gating (`can_transduce` / spend)
/// before invoking this — same discipline as any other transduction outcome.
///
/// # Errors
///
/// Propagates sink write failures.
pub fn write_effect(
    sink: &mut dyn EffectorSink,
    id: EffectorId,
    value: Value,
    stamp: CausalStamp,
    parent: Option<SignalId>,
    cell: CellId,
    events: &mut Vec<CausalEvent>,
) -> Result<()> {
    sink.write(id, value, stamp)?;
    let mut hasher = Hasher::new();
    hasher.update(b"effect");
    hasher.update(&id.0.to_le_bytes());
    hasher.update(&stamp.0.to_le_bytes());
    hasher.update(&cell.0.to_le_bytes());
    let child = SignalId(finalize_u128(&hasher));
    events.push(CausalEvent {
        parent,
        child,
        cell,
        stamp,
        tag: "effect".into(),
    });
    Ok(())
}
