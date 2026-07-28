//! Composable, near-pure transduction functions.
//!
//! Integer millis only — no floats on the core path.

use crate::signal::{MetaValue, Payload, Scope, Signal, SignalKind, SignalType, Tag};

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

/// Chemistry body of a [`TransductionFn`] step.
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
    /// Output signal kind.
    pub output_kind: SignalKind,
    /// Output scope.
    pub output_scope: Scope,
    /// Payload template copied onto each output (Identity) or merged as defaults.
    pub payload_template: Payload,
    /// When false, the step produces no outputs.
    pub enabled: bool,
    /// Chemistry applied to the inbound signal.
    pub kernel: TransductionKernel,
}

impl TransductionFn {
    /// Create a named identity-echo function (operational, `SelfCell`, empty payload).
    #[must_use]
    pub fn identity_echo(name: impl Into<String>, output_kind: impl Into<SignalKind>) -> Self {
        Self {
            name: name.into(),
            output_kind: output_kind.into(),
            output_scope: Scope::SelfCell,
            payload_template: Payload::empty(),
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

    /// Builder: override payload template.
    #[must_use]
    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload_template = payload;
        self
    }

    /// Builder: attach a chemistry kernel.
    #[must_use]
    pub fn with_kernel(mut self, kernel: TransductionKernel) -> Self {
        self.kernel = kernel;
        self
    }

    /// Run the function over an input signal, producing unstamped output specs.
    ///
    /// Outputs use a placeholder stamp/source; [`crate::transduction::emit_from_cascade`]
    /// rewrites them from the cell context.
    ///
    /// Kernel failures (missing meta, parse errors, div-by-zero) yield no outputs.
    #[must_use]
    pub fn call(&self, input: &Signal) -> Vec<Signal> {
        if !self.enabled {
            return Vec::new();
        }
        let Some(payload) = apply_kernel(&self.kernel, input, &self.payload_template) else {
            return Vec::new();
        };
        vec![Signal::new(
            SignalType::Operational,
            self.output_kind.clone(),
            self.output_scope,
            payload,
            input.source,
            input.stamp,
        )]
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
