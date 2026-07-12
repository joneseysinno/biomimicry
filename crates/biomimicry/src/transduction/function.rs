//! Composable, near-pure transduction functions.

use crate::signal::Signal;

/// A near-pure Phase 2 transduction function.
#[derive(Debug, Clone)]
pub struct TransductionFn {
    /// Function name / gene role.
    pub name: String,
}

impl TransductionFn {
    /// Create a named transduction function.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Run the function over an input signal, producing outputs.
    #[must_use]
    pub fn call(&self, _input: &Signal) -> Vec<Signal> {
        todo!("run near-pure transduction function")
    }
}
