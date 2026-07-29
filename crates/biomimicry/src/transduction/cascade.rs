//! Bind receptor → run cascade over active genes.
//!
//! # 0.3.0 semantic break — pipeline, not fan-out
//!
//! Prior to 0.3.0, every step received the same original `input` and all step
//! outputs were **concatenated**. That made multi-step computation impossible
//! (step *i+1* never saw step *i*'s result).
//!
//! From 0.3.0, [`Cascade::run`] **chains**: step *i*'s outputs are step *i+1*'s
//! inputs; **only the final step's outputs** are returned. Explicit fan-out is
//! [`crate::transduction::TransductionKind::Fanout`]. A single-step cascade is
//! bit-identical to the pre-0.3.0 behaviour for identity-echo steps.

use crate::cell::ExpressionState;
use crate::error::Result;
use crate::signal::Signal;
use crate::transduction::TransductionFn;

/// A Phase 2 cascade — ordered transduction steps for one gene.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cascade {
    /// Steps to execute in order (pipeline).
    pub steps: Vec<TransductionFn>,
}

impl Cascade {
    /// Create an empty cascade.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: append a step.
    #[must_use]
    pub fn with_step(mut self, step: TransductionFn) -> Self {
        self.steps.push(step);
        self
    }

    /// Run the cascade as a pipeline; return only the final step's outputs.
    ///
    /// The first step receives `input` as a singleton vector. Each subsequent
    /// step receives the previous step's entire output vector via
    /// [`TransductionFn::call_many`] (so multi-input arith / fold see all
    /// siblings). Single-input steps map over that vector.
    ///
    /// The `expression` parameter is reserved for future gene-gated steps;
    /// activity gating lives in [`crate::transduction::CascadeTransducer`].
    ///
    /// # Errors
    ///
    /// Propagates typed failures from any step (`ValueTypeMismatch`,
    /// `DivideByZero`, decode errors, …).
    pub fn run(&self, expression: &ExpressionState, input: &Signal) -> Result<Vec<Signal>> {
        let _ = expression;
        if self.steps.is_empty() {
            return Ok(Vec::new());
        }
        let mut current = vec![input.clone()];
        for step in &self.steps {
            current = step.call_many(&current)?;
            if current.is_empty() {
                break;
            }
        }
        Ok(current)
    }
}
