//! Composable, near-pure transduction functions.
//!
//! Integer millis only — no floats on the core path.
//!
//! [`TransductionFn::call`] is fallible: type mismatches yield
//! [`crate::BiomimicryError::ValueTypeMismatch`], never `Ok(vec![])` for errors.
//! Disabled steps return `Ok(vec![])` (not an error).
//!
//! Legacy [`TransductionKernel`] chemistry is preserved for backward compatibility:
//! when `kernel != Identity`, that path runs (even if `kind` is [`TransductionKind::IdentityEcho`]).

use crate::error::{BiomimicryError, Result};
use crate::signal::{
    MetaValue, Payload, Scope, Signal, SignalKind, SignalType, Tag, Value,
};
use crate::transduction::arith::{eval_binary, eval_unary, is_unary};
use crate::transduction::fold::fold_signals;
use crate::transduction::map::apply_map;
use crate::transduction::resolve::fn_from_spec;
use crate::transduction::{ArithOp, CmpOp, MapSpec, TransductionKind};

/// Binary integer op over two payload metadata fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryMetaOp {
    /// `left + right`
    Add,
    /// `left * right`
    Mul,
    /// Compare: `-1` / `0` / `1` for less / equal / greater.
    Compare,
}

/// Unary integer op over one payload metadata field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryMetaOp {
    /// Negate.
    Negate,
    /// Reciprocal in milli-units: `1_000_000 / x` (x ≠ 0).
    ReciprocalMilli,
}

/// Chemistry body of a [`TransductionFn`] step (legacy meta-field path).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TransductionKernel {
    /// Re-emit `payload_template` (ignores input body/meta).
    #[default]
    Identity,
    /// Re-emit the inbound payload (kind/scope come from the [`TransductionFn`]).
    Forward,
    /// Binary arithmetic on i64 meta fields → write `out` meta; preserve other input meta.
    BinaryMeta {
        /// Operator.
        op: BinaryMetaOp,
        /// Left operand meta tag.
        left: Tag,
        /// Right operand meta tag.
        right: Tag,
        /// Result meta tag.
        out: Tag,
    },
    /// Unary arithmetic on an i64 meta field → write `out` meta; preserve other input meta.
    UnaryMeta {
        /// Operator.
        op: UnaryMetaOp,
        /// Input meta tag.
        input: Tag,
        /// Result meta tag.
        out: Tag,
    },
}

/// A near-pure Phase 2 transduction function.
///
/// Default “identity echo”: re-emit an operational signal with the configured
/// kind, scope, and payload template (causal stamp / source filled by emit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransductionFn {
    /// Function name / gene role.
    pub name: String,
    /// Declarative chemistry (primary path when `kernel` is [`TransductionKernel::Identity`]).
    pub kind: TransductionKind,
    /// Output signal kind.
    pub output_kind: SignalKind,
    /// Output scope.
    pub output_scope: Scope,
    /// Payload template (kept in sync with [`TransductionKind::IdentityEcho`] for builders).
    pub payload_template: Payload,
    /// When false, the step produces no outputs.
    pub enabled: bool,
    /// Legacy chemistry; used when not [`TransductionKernel::Identity`].
    pub kernel: TransductionKernel,
}

impl TransductionFn {
    /// Create a named identity-echo function (operational, `SelfCell`, empty payload).
    #[must_use]
    pub fn identity_echo(name: impl Into<String>, output_kind: impl Into<SignalKind>) -> Self {
        let payload_template = Payload::empty();
        Self {
            name: name.into(),
            kind: TransductionKind::IdentityEcho {
                payload_template: payload_template.clone(),
            },
            output_kind: output_kind.into(),
            output_scope: Scope::SelfCell,
            payload_template,
            enabled: true,
            kernel: TransductionKernel::Identity,
        }
    }

    /// Builder: override output scope.
    #[must_use]
    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.output_scope = scope;
        self
    }

    /// Builder: override payload template (also updates [`TransductionKind::IdentityEcho`]).
    #[must_use]
    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload_template = payload.clone();
        if let TransductionKind::IdentityEcho {
            payload_template: tmpl,
        } = &mut self.kind
        {
            *tmpl = payload;
        }
        self
    }

    /// Builder: attach a legacy chemistry kernel.
    #[must_use]
    pub fn with_kernel(mut self, kernel: TransductionKernel) -> Self {
        self.kernel = kernel;
        self
    }

    /// Builder: set declarative kind.
    #[must_use]
    pub fn with_kind(mut self, kind: TransductionKind) -> Self {
        if let TransductionKind::IdentityEcho {
            payload_template: tmpl,
        } = &kind
        {
            self.payload_template = tmpl.clone();
        }
        self.kind = kind;
        self
    }

    /// Run the function over a single input signal.
    ///
    /// Outputs use a placeholder stamp/source; [`crate::transduction::emit_from_cascade`]
    /// rewrites them from the cell context.
    ///
    /// # Errors
    ///
    /// Typed mismatches and arithmetic failures (never silent empty for errors).
    pub fn call(&self, input: &Signal) -> Result<Vec<Signal>> {
        self.call_many(std::slice::from_ref(input))
    }

    /// Run over a vector of inbound signals.
    ///
    /// Multi-input steps ([`TransductionKind::Arith`] binary, [`TransductionKind::Fold`],
    /// [`TransductionKind::Compare`], companion [`MapSpec`]s) consume the whole vector.
    /// Single-input steps map over it. [`TransductionKind::Fanout`] runs each child on
    /// the same inputs and concatenates outputs.
    ///
    /// # Errors
    ///
    /// Propagates typed failures from the active chemistry path.
    pub fn call_many(&self, inputs: &[Signal]) -> Result<Vec<Signal>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        // Legacy kernel path — preserve Option-style silent empty on meta failures.
        if self.kernel != TransductionKernel::Identity {
            let mut out = Vec::new();
            for input in inputs {
                if let Some(sig) = self.call_legacy_kernel(input) {
                    out.push(sig);
                }
            }
            return Ok(out);
        }

        match &self.kind {
            TransductionKind::Fanout(children) => {
                let mut out = Vec::new();
                for child_spec in children {
                    let child = fn_from_spec(child_spec);
                    out.extend(child.call_many(inputs)?);
                }
                Ok(out)
            }
            TransductionKind::Effect(_id) => {
                // Effect writes are queued by [`crate::transduction::CascadeTransducer`]
                // (pending → organism sink). Here we only validate the payload
                // and emit no outbound signals.
                for input in inputs {
                    let _value = input.payload.value()?;
                }
                Ok(Vec::new())
            }
            TransductionKind::Fold(spec) => match fold_signals(spec, inputs, &self.name)? {
                Some(value) => {
                    let stamp_src = inputs
                        .iter()
                        .max_by(|a, b| a.stamp.cmp(&b.stamp).then(a.id.cmp(&b.id)));
                    let Some(src) = stamp_src else {
                        return Ok(Vec::new());
                    };
                    Ok(vec![self.emit_value(src, value)])
                }
                None => Ok(Vec::new()),
            },
            TransductionKind::Arith(op) if !is_unary(*op) => {
                self.call_arith_binary(*op, inputs)
            }
            TransductionKind::Compare(op) => self.call_compare(*op, inputs),
            TransductionKind::Map(spec) if map_needs_companion(spec) => {
                self.call_map_companion(spec, inputs)
            }
            // Single-input kinds: map over the vector.
            _ => {
                let mut out = Vec::new();
                for input in inputs {
                    out.extend(self.call_kind_single(input)?);
                }
                Ok(out)
            }
        }
    }

    fn call_kind_single(&self, input: &Signal) -> Result<Vec<Signal>> {
        match &self.kind {
            TransductionKind::IdentityEcho { payload_template } => Ok(vec![Signal::new(
                SignalType::Operational,
                self.output_kind.clone(),
                self.output_scope,
                payload_template.clone(),
                input.source,
                input.stamp,
            )]),
            TransductionKind::Forward => Ok(vec![Signal::new(
                SignalType::Operational,
                self.output_kind.clone(),
                self.output_scope,
                input.payload.clone(),
                input.source,
                input.stamp,
            )]),
            TransductionKind::Const(value) => Ok(vec![self.emit_value(input, value.clone())]),
            TransductionKind::Arith(op) if is_unary(*op) => {
                let v = input.payload.value()?;
                let a = expect_int(&v, &self.name)?;
                let n = eval_unary(*op, a)?;
                Ok(vec![self.emit_value(input, Value::Int(n))])
            }
            TransductionKind::Arith(op) => {
                // Binary op invoked with a single inbound — need two Ints.
                Err(BiomimicryError::ValueTypeMismatch {
                    function: self.name.clone(),
                    expected: format!("two Int inputs for Arith({op:?})"),
                    got: "single input".into(),
                })
            }
            TransductionKind::Map(spec) => {
                let v = input.payload.value()?;
                let out = apply_map(spec, &v, None, &self.name)?;
                Ok(vec![self.emit_value(input, out)])
            }
            TransductionKind::Compare(_)
            | TransductionKind::Fold(_)
            | TransductionKind::Effect(_)
            | TransductionKind::Fanout(_) => {
                // Handled in call_many.
                unreachable!("multi-input kind reached call_kind_single")
            }
        }
    }

    fn call_arith_binary(&self, op: ArithOp, inputs: &[Signal]) -> Result<Vec<Signal>> {
        if inputs.len() < 2 {
            return Err(BiomimicryError::ValueTypeMismatch {
                function: self.name.clone(),
                expected: format!("two Int inputs for Arith({op:?})"),
                got: format!("{} input(s)", inputs.len()),
            });
        }
        let a = expect_int(&inputs[0].payload.value()?, &self.name)?;
        let b = expect_int(&inputs[1].payload.value()?, &self.name)?;
        let n = eval_binary(op, a, b)?;
        Ok(vec![self.emit_value(&inputs[1], Value::Int(n))])
    }

    fn call_compare(&self, op: CmpOp, inputs: &[Signal]) -> Result<Vec<Signal>> {
        if inputs.len() < 2 {
            return Err(BiomimicryError::ValueTypeMismatch {
                function: self.name.clone(),
                expected: format!("two values for Compare({op:?})"),
                got: format!("{} input(s)", inputs.len()),
            });
        }
        let a = inputs[0].payload.value()?;
        let b = inputs[1].payload.value()?;
        let flag = cmp_values(op, &a, &b);
        Ok(vec![self.emit_value(&inputs[1], Value::Bool(flag))])
    }

    fn call_map_companion(&self, spec: &MapSpec, inputs: &[Signal]) -> Result<Vec<Signal>> {
        if inputs.is_empty() {
            return Err(BiomimicryError::ValueTypeMismatch {
                function: self.name.clone(),
                expected: "at least one input for Map".into(),
                got: "none".into(),
            });
        }
        let primary = inputs[0].payload.value()?;
        let companion = if inputs.len() >= 2 {
            Some(inputs[1].payload.value()?)
        } else {
            None
        };
        let out = apply_map(spec, &primary, companion.as_ref(), &self.name)?;
        Ok(vec![self.emit_value(&inputs[0], out)])
    }

    fn call_legacy_kernel(&self, input: &Signal) -> Option<Signal> {
        let payload = apply_kernel(&self.kernel, input, &self.payload_template)?;
        Some(Signal::new(
            SignalType::Operational,
            self.output_kind.clone(),
            self.output_scope,
            payload,
            input.source,
            input.stamp,
        ))
    }

    fn emit_value(&self, input: &Signal, value: Value) -> Signal {
        Signal::new(
            SignalType::Operational,
            self.output_kind.clone(),
            self.output_scope,
            Payload::of(value),
            input.source,
            input.stamp,
        )
    }
}

fn map_needs_companion(spec: &MapSpec) -> bool {
    matches!(
        spec,
        MapSpec::Set { value: None, .. } | MapSpec::Append { value: None }
    )
}

fn expect_int(value: &Value, function: &str) -> Result<i64> {
    match value {
        Value::Int(n) => Ok(*n),
        other => Err(BiomimicryError::ValueTypeMismatch {
            function: function.into(),
            expected: "Int".into(),
            got: value_ty(other).into(),
        }),
    }
}

fn cmp_values(op: CmpOp, a: &Value, b: &Value) -> bool {
    // Structural Ord on Value covers all variants deterministically.
    let ord = a.cmp(b);
    match op {
        CmpOp::Eq => ord.is_eq(),
        CmpOp::Ne => ord.is_ne(),
        CmpOp::Lt => ord.is_lt(),
        CmpOp::Le => ord.is_le(),
        CmpOp::Gt => ord.is_gt(),
        CmpOp::Ge => ord.is_ge(),
    }
}

fn value_ty(v: &Value) -> &'static str {
    match v {
        Value::Unit => "Unit",
        Value::Bool(_) => "Bool",
        Value::Int(_) => "Int",
        Value::Text(_) => "Text",
        Value::List(_) => "List",
        Value::Record(_) => "Record",
        Value::Bytes(_) => "Bytes",
    }
}

fn apply_kernel(kernel: &TransductionKernel, input: &Signal, template: &Payload) -> Option<Payload> {
    match kernel {
        TransductionKernel::Identity => Some(template.clone()),
        TransductionKernel::Forward => {
            let mut payload = input.payload.clone();
            for (k, v) in &template.metadata {
                payload.metadata.entry(k.clone()).or_insert_with(|| v.clone());
            }
            Some(payload)
        }
        TransductionKernel::BinaryMeta {
            op,
            left,
            right,
            out,
        } => {
            let a = parse_i64_meta(&input.payload, left)?;
            let b = parse_i64_meta(&input.payload, right)?;
            let value = match op {
                BinaryMetaOp::Add => a.checked_add(b)?,
                BinaryMetaOp::Mul => a.checked_mul(b)?,
                BinaryMetaOp::Compare => match a.cmp(&b) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                },
            };
            Some(merge_result_payload(input, template, out, value))
        }
        TransductionKernel::UnaryMeta { op, input: tag, out } => {
            let x = parse_i64_meta(&input.payload, tag)?;
            let value = match op {
                UnaryMetaOp::Negate => x.checked_neg()?,
                UnaryMetaOp::ReciprocalMilli => {
                    if x == 0 {
                        return None;
                    }
                    1_000_000i64.checked_div(x)?
                }
            };
            Some(merge_result_payload(input, template, out, value))
        }
    }
}

fn parse_i64_meta(payload: &Payload, tag: &Tag) -> Option<i64> {
    payload.metadata.get(tag)?.as_str().parse().ok()
}

fn merge_result_payload(input: &Signal, template: &Payload, out: &Tag, value: i64) -> Payload {
    let mut payload = input.payload.clone();
    for (k, v) in &template.metadata {
        payload.metadata.entry(k.clone()).or_insert_with(|| v.clone());
    }
    if template.strength_milli != 1000 {
        payload.strength_milli = template.strength_milli;
    }
    payload
        .metadata
        .insert(out.clone(), MetaValue::new(value.to_string()));
    payload
}
