//! Bind receptor → run cascade over active genes.

use crate::cell::ExpressionState;
use crate::error::Result;
use crate::signal::Signal;
use crate::transduction::TransductionFn;

/// A Phase 2 cascade bound to a receptor match.
#[derive(Debug, Clone)]
pub struct Cascade {
    /// Steps to execute in order.
    pub steps: Vec<TransductionFn>,
}

impl Cascade {
    /// Create an empty cascade.
    #[must_use]
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Bind a receptor match and run over the active gene set.
    ///
    /// # Errors
    ///
    /// Returns an error if the cascade cannot bind or execute.
    pub fn run(&self, _expression: &ExpressionState, _input: &Signal) -> Result<Vec<Signal>> {
        todo!("bind receptor → run cascade on active genes")
    }
}

impl Default for Cascade {
    fn default() -> Self {
        Self::new()
    }
}
