//! Declarative, content-hashable transduction specifications.
//!
//! A [`TransductionSpec`] is what a gene *codes for* — the enzyme body carried
//! in the DNA ([`crate::genesis::Cistron`]). It must be `PartialEq` / serialisable
//! / hashable; no closures, no trait objects.

use blake3::Hasher;
use smol_str::SmolStr;

use crate::effector::EffectorId;
use crate::genesis::hash::{finalize_u128, update_str, update_u32};
use crate::signal::{Payload, Scope, SignalKind, Value};

/// Barrier that ends a fold accumulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FoldBarrier {
    /// Emit when `seen == n` inbound signals have been folded (default; replay-stable).
    Arity(u32),
    /// Emit at the end of the inner cycle (aggregations of unknown size).
    Cadence,
}

impl Default for FoldBarrier {
    fn default() -> Self {
        Self::Arity(2)
    }
}

/// Integer arithmetic op — single authority for millis discipline.
///
/// Evaluation rules (locked M11 §0.5 / §2.3):
/// - `i128` intermediates
/// - round half away from zero on scaled multiply/divide
/// - saturating, never wrapping
/// - division by zero → [`crate::BiomimicryError::DivideByZero`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ArithOp {
    /// `a + b`
    Add = 0,
    /// `a - b`
    Sub = 1,
    /// Millis multiply: `(a * b) / 1000` with round-half-away-from-zero.
    Mul = 2,
    /// Millis divide: `(a * 1000) / b` with round-half-away-from-zero.
    Div = 3,
    /// `min(a, b)`
    Min = 4,
    /// `max(a, b)`
    Max = 5,
    /// `-a`
    Neg = 6,
    /// `|a|`
    Abs = 7,
}

/// Comparison op → [`Value::Bool`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CmpOp {
    /// `a == b`
    Eq = 0,
    /// `a != b`
    Ne = 1,
    /// `a < b`
    Lt = 2,
    /// `a <= b`
    Le = 3,
    /// `a > b`
    Gt = 4,
    /// `a >= b`
    Ge = 5,
}

/// Record / list structural transform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapSpec {
    /// Get a record field by key; input must be a [`Value::Record`].
    Get {
        /// Field name.
        key: SmolStr,
    },
    /// Set a record field; input must be a [`Value::Record`].
    Set {
        /// Field name.
        key: SmolStr,
        /// Value to write (or take from inbound if `None` — uses companion signal).
        value: Option<Value>,
    },
    /// Rename a record field.
    Rename {
        /// Old key.
        from: SmolStr,
        /// New key.
        to: SmolStr,
    },
    /// Project a record onto a subset of keys (canonical order preserved).
    Project {
        /// Keys to keep.
        keys: Vec<SmolStr>,
    },
    /// Index into a list.
    Index {
        /// Zero-based index.
        index: u32,
    },
    /// Append one value to a list (value from companion or literal).
    Append {
        /// Literal to append; `None` means use the second inbound value.
        value: Option<Value>,
    },
}

/// Fold: accumulate N inbound values into one outbound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoldSpec {
    /// Operator applied in `(stamp, SignalId)` order.
    pub op: ArithOp,
    /// Completion rule.
    pub barrier: FoldBarrier,
    /// Initial accumulator (`Unit` means “first inbound becomes acc”).
    pub init: Value,
}

impl FoldSpec {
    /// Fold with arity barrier (the M11 default).
    #[must_use]
    pub fn arity(op: ArithOp, n: u32) -> Self {
        Self {
            op,
            barrier: FoldBarrier::Arity(n),
            init: Value::Unit,
        }
    }
}

/// Chemistry body of one cascade step — declarative, hashable, no closures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransductionKind {
    /// Re-emit a fixed payload template (M4 identity-echo, preserved).
    IdentityEcho {
        /// Template payload.
        payload_template: Payload,
    },
    /// Re-emit the inbound payload with this step's kind/scope (bridge identity).
    Forward,
    /// Arithmetic over inbound [`Value::Int`]s.
    Arith(ArithOp),
    /// Fold N inbound → 1 outbound.
    Fold(FoldSpec),
    /// Record / list structural map.
    Map(MapSpec),
    /// Compare → [`Value::Bool`].
    Compare(CmpOp),
    /// Inject a literal.
    Const(Value),
    /// Phase 2 write leaving the signal stream.
    Effect(EffectorId),
    /// Explicit fan-out: every child sees the same inputs; outputs concatenated.
    Fanout(Vec<TransductionFnSpec>),
}

/// One declarative cascade step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransductionFnSpec {
    /// Step name.
    pub name: String,
    /// Chemistry.
    pub kind: TransductionKind,
    /// Output signal kind.
    pub output_kind: SignalKind,
    /// Output scope.
    pub output_scope: Scope,
    /// When false, the step produces no outputs (inhibitory form).
    pub enabled: bool,
}

impl TransductionFnSpec {
    /// Identity-echo step.
    #[must_use]
    pub fn identity_echo(name: impl Into<String>, output_kind: impl Into<SignalKind>) -> Self {
        Self {
            name: name.into(),
            kind: TransductionKind::IdentityEcho {
                payload_template: Payload::empty(),
            },
            output_kind: output_kind.into(),
            output_scope: Scope::SelfCell,
            enabled: true,
        }
    }

    /// Forward the inbound payload under a new kind/scope (bridge cistrons).
    #[must_use]
    pub fn forward(name: impl Into<String>, output_kind: impl Into<SignalKind>) -> Self {
        Self {
            name: name.into(),
            kind: TransductionKind::Forward,
            output_kind: output_kind.into(),
            output_scope: Scope::SelfCell,
            enabled: true,
        }
    }

    /// Arithmetic step.
    #[must_use]
    pub fn arith(
        name: impl Into<String>,
        op: ArithOp,
        output_kind: impl Into<SignalKind>,
    ) -> Self {
        Self {
            name: name.into(),
            kind: TransductionKind::Arith(op),
            output_kind: output_kind.into(),
            output_scope: Scope::SelfCell,
            enabled: true,
        }
    }

    /// Fold step.
    #[must_use]
    pub fn fold(
        name: impl Into<String>,
        fold: FoldSpec,
        output_kind: impl Into<SignalKind>,
    ) -> Self {
        Self {
            name: name.into(),
            kind: TransductionKind::Fold(fold),
            output_kind: output_kind.into(),
            output_scope: Scope::SelfCell,
            enabled: true,
        }
    }

    /// Effector write step (no outbound signal; write is the effect).
    #[must_use]
    pub fn effect(name: impl Into<String>, effector: EffectorId) -> Self {
        Self {
            name: name.into(),
            kind: TransductionKind::Effect(effector),
            output_kind: SignalKind::new("effect"),
            output_scope: Scope::SelfCell,
            enabled: true,
        }
    }

    /// Const inject.
    #[must_use]
    pub fn const_value(
        name: impl Into<String>,
        value: Value,
        output_kind: impl Into<SignalKind>,
    ) -> Self {
        Self {
            name: name.into(),
            kind: TransductionKind::Const(value),
            output_kind: output_kind.into(),
            output_scope: Scope::SelfCell,
            enabled: true,
        }
    }

    /// Builder: override scope.
    #[must_use]
    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.output_scope = scope;
        self
    }

    /// Toggle enabled (complement polarity for a step).
    #[must_use]
    pub fn inhibited(mut self) -> Self {
        self.enabled = !self.enabled;
        self
    }
}

/// Full enzyme body carried by a cistron — ordered pipeline of steps.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TransductionSpec {
    /// Pipeline steps (chained; see [`crate::transduction::Cascade::run`]).
    pub steps: Vec<TransductionFnSpec>,
}

impl TransductionSpec {
    /// Empty spec (no enzyme).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Single-step spec.
    #[must_use]
    pub fn single(step: TransductionFnSpec) -> Self {
        Self { steps: vec![step] }
    }

    /// Builder: append a step.
    #[must_use]
    pub fn with_step(mut self, step: TransductionFnSpec) -> Self {
        self.steps.push(step);
        self
    }

    /// Inhibitory form for complement-derived genes: toggle each step's `enabled`.
    ///
    /// Toggle (not force-false) so complement∘complement restores the original
    /// spec and `GeneId` involution holds.
    #[must_use]
    pub fn inhibitory(&self) -> Self {
        Self {
            steps: self.steps.iter().cloned().map(TransductionFnSpec::inhibited).collect(),
        }
    }

    /// Canonical content digest (included in [`crate::genesis::Cistron::content_id`]).
    #[must_use]
    pub fn content_digest(&self) -> u128 {
        let mut hasher = Hasher::new();
        update_u32(
            &mut hasher,
            u32::try_from(self.steps.len()).expect("step count fits u32"),
        );
        for step in &self.steps {
            hash_step(&mut hasher, step);
        }
        finalize_u128(&hasher)
    }

    /// Absorb into an existing hasher (cistron canonical form).
    pub(crate) fn hash_into(&self, hasher: &mut Hasher) {
        hasher.update(&self.content_digest().to_le_bytes());
    }
}

fn hash_step(hasher: &mut Hasher, step: &TransductionFnSpec) {
    update_str(hasher, &step.name);
    hasher.update(&[u8::from(step.enabled)]);
    update_str(hasher, step.output_kind.as_str());
    hasher.update(&[step.output_scope.wire_tag()]);
    hash_kind(hasher, &step.kind);
}

fn hash_kind(hasher: &mut Hasher, kind: &TransductionKind) {
    match kind {
        TransductionKind::IdentityEcho { payload_template } => {
            hasher.update(&[0u8]);
            hasher.update(&payload_template.digest().to_le_bytes());
        }
        TransductionKind::Forward => {
            hasher.update(&[8u8]);
        }
        TransductionKind::Arith(op) => {
            hasher.update(&[1u8, *op as u8]);
        }
        TransductionKind::Fold(fold) => {
            hasher.update(&[2u8, fold.op as u8]);
            match fold.barrier {
                FoldBarrier::Arity(n) => {
                    hasher.update(&[0u8]);
                    update_u32(hasher, n);
                }
                FoldBarrier::Cadence => {
                    hasher.update(&[1u8]);
                }
            }
            let enc = fold.init.encode().unwrap_or_default();
            update_u32(hasher, u32::try_from(enc.len()).unwrap_or(0));
            hasher.update(&enc);
        }
        TransductionKind::Map(map) => {
            hasher.update(&[3u8]);
            hash_map(hasher, map);
        }
        TransductionKind::Compare(op) => {
            hasher.update(&[4u8, *op as u8]);
        }
        TransductionKind::Const(v) => {
            hasher.update(&[5u8]);
            let enc = v.encode().unwrap_or_default();
            update_u32(hasher, u32::try_from(enc.len()).unwrap_or(0));
            hasher.update(&enc);
        }
        TransductionKind::Effect(id) => {
            hasher.update(&[6u8]);
            hasher.update(&id.0.to_le_bytes());
        }
        TransductionKind::Fanout(children) => {
            hasher.update(&[7u8]);
            update_u32(
                hasher,
                u32::try_from(children.len()).expect("fanout len fits u32"),
            );
            for child in children {
                hash_step(hasher, child);
            }
        }
    }
}

fn hash_map(hasher: &mut Hasher, map: &MapSpec) {
    match map {
        MapSpec::Get { key } => {
            hasher.update(&[0u8]);
            update_str(hasher, key.as_str());
        }
        MapSpec::Set { key, value } => {
            hasher.update(&[1u8]);
            update_str(hasher, key.as_str());
            hash_opt_value(hasher, value.as_ref());
        }
        MapSpec::Rename { from, to } => {
            hasher.update(&[2u8]);
            update_str(hasher, from.as_str());
            update_str(hasher, to.as_str());
        }
        MapSpec::Project { keys } => {
            hasher.update(&[3u8]);
            update_u32(hasher, u32::try_from(keys.len()).expect("keys fit u32"));
            for k in keys {
                update_str(hasher, k.as_str());
            }
        }
        MapSpec::Index { index } => {
            hasher.update(&[4u8]);
            update_u32(hasher, *index);
        }
        MapSpec::Append { value } => {
            hasher.update(&[5u8]);
            hash_opt_value(hasher, value.as_ref());
        }
    }
}

fn hash_opt_value(hasher: &mut Hasher, value: Option<&Value>) {
    match value {
        None => {
            hasher.update(&[0u8]);
        }
        Some(v) => {
            hasher.update(&[1u8]);
            let enc = v.encode().unwrap_or_default();
            update_u32(hasher, u32::try_from(enc.len()).unwrap_or(0));
            hasher.update(&enc);
        }
    }
}
