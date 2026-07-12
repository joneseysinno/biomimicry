//! Produce new signals from a cascade result.

use crate::cell::{Cell, CellId};
use crate::error::Result;
use crate::signal::{CausalStamp, Signal};

/// Stamp and scope cascade outputs for medium delivery / `Emit` ops.
///
/// Overwrites `source` and `stamp` from the emitting cell; preserves kind,
/// type, scope, and payload from the cascade function. Rebuilds content-addressed id.
///
/// # Errors
///
/// Currently infallible; reserved for future emission invariant checks.
pub fn emit_from_cascade(
    outputs: Vec<Signal>,
    source: CellId,
    stamp: CausalStamp,
) -> Result<Vec<Signal>> {
    Ok(outputs
        .into_iter()
        .map(|sig| Signal::new(sig.ty, sig.kind, sig.scope, sig.payload, source, stamp))
        .collect())
}

/// Convenience: emit cascade outputs using a live cell's identity and peek stamp.
///
/// # Errors
///
/// Propagates [`emit_from_cascade`] errors.
pub fn emit_from_cascade_cell(cell: &Cell, outputs: Vec<Signal>) -> Result<Vec<Signal>> {
    emit_from_cascade(outputs, cell.id, cell.peek_stamp())
}
