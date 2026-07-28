//! Bind receptor → run cascade over active genes.

use crate::cell::ExpressionState;
use crate::error::Result;
use crate::signal::Signal;
use crate::transduction::TransductionFn;

/// A Phase 2 cascade — ordered transduction steps for one gene.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cascade {
    /// Steps to execute in order.
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

    /// Run all steps in order; each step consumes the previous step's outputs.
    ///
    /// The first step receives `input`. Every subsequent step is called once per
    /// signal produced by the prior step. All intermediate and final outputs are
    /// concatenated (so multi-step identity-echo cascades still emit every kind).
    ///
    /// The `expression` parameter is reserved for future gene-gated steps;
    /// activity gating lives in [`crate::transduction::CascadeTransducer`].
    ///
    /// # Errors
    ///
    /// Currently infallible; reserved for future binding failures.
    pub fn run(&self, expression: &ExpressionState, input: &Signal) -> Result<Vec<Signal>> {
        let _ = expression;
        if self.steps.is_empty() {
            return Ok(Vec::new());
        }
        let mut current = vec![input.clone()];
        let mut all = Vec::new();
        for step in &self.steps {
            let mut next = Vec::new();
            for sig in &current {
                next.extend(step.call(sig));
            }
            all.extend(next.iter().cloned());
            current = next;
            if current.is_empty() {
                break;
            }
        }
        Ok(all)
    }
}
