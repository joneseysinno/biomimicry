//! Basin membership and fault-tolerance geometry.

/// A basin of attraction in the landscape.
#[derive(Debug, Clone)]
pub struct Basin {
    /// Stable attractor identity.
    pub id: u64,
    /// Radius / tolerance in state space.
    pub radius: f64,
}

impl Basin {
    /// Create a basin around an attractor.
    #[must_use]
    pub fn new(id: u64, radius: f64) -> Self {
        Self { id, radius }
    }

    /// Whether a state key is inside this basin.
    #[must_use]
    pub fn contains(&self, _state_key: u64) -> bool {
        todo!("test basin membership")
    }
}
