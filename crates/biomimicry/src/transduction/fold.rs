//! Fold accumulation — N inbound values → one outbound.
//!
//! Application order is the M3 total order `(CausalStamp, SignalId)`.
//! An unfilled arity barrier is incomplete (returns no value), not an error.

use crate::causality::determinism::by_causal_order;
use crate::error::{BiomimicryError, Result};
use crate::signal::{CausalStamp, Signal, Value};
use crate::transduction::arith::{eval_binary, is_unary};
use crate::transduction::{FoldBarrier, FoldSpec};

/// Per-cell, per-gene fold accumulator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoldState {
    /// Running accumulator.
    pub acc: Value,
    /// Number of inbound values folded.
    pub seen: u32,
    /// Highest stamp among folded inputs.
    pub high_stamp: CausalStamp,
}

impl FoldState {
    /// Fresh state with the given initial accumulator.
    #[must_use]
    pub fn new(init: Value) -> Self {
        Self {
            acc: init,
            seen: 0,
            high_stamp: CausalStamp(i64::MIN),
        }
    }

    /// Reset to `init` (expression change / after barrier emit).
    pub fn clear(&mut self, init: Value) {
        self.acc = init;
        self.seen = 0;
        self.high_stamp = CausalStamp(i64::MIN);
    }

    /// Whether the barrier is satisfied.
    #[must_use]
    pub fn barrier_met(&self, barrier: FoldBarrier) -> bool {
        match barrier {
            FoldBarrier::Arity(n) => self.seen >= n,
            FoldBarrier::Cadence => self.seen > 0,
        }
    }

    /// Fold one inbound `value` at `stamp` under `op`.
    ///
    /// When `acc` is [`Value::Unit`] and `seen == 0`, the first inbound becomes
    /// the accumulator without applying `op` (Unit is the fold identity).
    ///
    /// # Errors
    ///
    /// Propagates arithmetic / type errors from the op.
    pub fn apply(
        &mut self,
        op: crate::transduction::ArithOp,
        value: Value,
        stamp: CausalStamp,
        function: &str,
    ) -> Result<()> {
        if stamp > self.high_stamp {
            self.high_stamp = stamp;
        }
        if self.seen == 0 && self.acc == Value::Unit {
            self.acc = value;
            self.seen = 1;
            return Ok(());
        }
        if is_unary(op) {
            return Err(BiomimicryError::ValueTypeMismatch {
                function: function.into(),
                expected: "binary fold op".into(),
                got: format!("unary {op:?}"),
            });
        }
        let a = expect_int(&self.acc, function)?;
        let b = expect_int(&value, function)?;
        let next = eval_binary(op, a, b)?;
        self.acc = Value::Int(next);
        self.seen = self.seen.saturating_add(1);
        Ok(())
    }
}

/// Clear helper for expression-change invalidation.
pub fn clear_on_expression_change(state: &mut FoldState, init: Value) {
    state.clear(init);
}

/// Fold `signals` under `spec` in `(stamp, SignalId)` order.
///
/// Returns `Ok(None)` when the barrier is not yet met (incomplete, not an error).
///
/// # Errors
///
/// Propagates type / arithmetic failures from [`FoldState::apply`].
pub fn fold_signals(spec: &FoldSpec, signals: &[Signal], function: &str) -> Result<Option<Value>> {
    let mut ordered: Vec<&Signal> = signals.iter().collect();
    ordered.sort_by(|a, b| by_causal_order(a, b));

    let mut state = FoldState::new(spec.init.clone());
    for sig in ordered {
        let value = sig.payload.value()?;
        state.apply(spec.op, value, sig.stamp, function)?;
        if matches!(spec.barrier, FoldBarrier::Arity(n) if state.seen >= n) {
            break;
        }
    }
    if state.barrier_met(spec.barrier) {
        Ok(Some(state.acc))
    } else {
        Ok(None)
    }
}

fn expect_int(value: &Value, function: &str) -> Result<i64> {
    match value {
        Value::Int(n) => Ok(*n),
        other => Err(BiomimicryError::ValueTypeMismatch {
            function: function.into(),
            expected: "Int".into(),
            got: match other {
                Value::Unit => "Unit",
                Value::Bool(_) => "Bool",
                Value::Text(_) => "Text",
                Value::List(_) => "List",
                Value::Record(_) => "Record",
                Value::Bytes(_) => "Bytes",
                Value::Int(_) => unreachable!(),
            }
            .into(),
        }),
    }
}
