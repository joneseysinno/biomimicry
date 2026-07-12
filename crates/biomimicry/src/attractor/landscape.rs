//! State-space terrain model for computation-as-relaxation.

/// Abstract energy / fitness landscape over organism state.
#[derive(Debug, Clone, Default)]
pub struct Landscape {
    /// Sampled height field placeholder (filled by later milestones).
    pub samples: Vec<f64>,
}

impl Landscape {
    /// Create an empty landscape.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Estimate potential energy of a state key.
    #[must_use]
    pub fn potential(&self, _state_key: u64) -> f64 {
        todo!("evaluate landscape potential at state")
    }
}
