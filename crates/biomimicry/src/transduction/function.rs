//! Composable, near-pure transduction functions.
//!
//! Integer millis only — no floats on the core path.

use crate::signal::{Payload, Scope, Signal, SignalKind, SignalType};

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
    /// Payload template copied onto each output.
    pub payload_template: Payload,
    /// When false, the step produces no outputs.
    pub enabled: bool,
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

    /// Run the function over an input signal, producing unstamped output specs.
    ///
    /// Outputs use a placeholder stamp/source; [`crate::transduction::emit_from_cascade`]
    /// rewrites them from the cell context.
    #[must_use]
    pub fn call(&self, input: &Signal) -> Vec<Signal> {
        if !self.enabled {
            return Vec::new();
        }
        let _ = input;
        vec![Signal::new(
            SignalType::Operational,
            self.output_kind.clone(),
            self.output_scope,
            self.payload_template.clone(),
            input.source,
            input.stamp,
        )]
    }
}
