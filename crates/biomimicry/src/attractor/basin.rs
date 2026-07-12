//! Basin membership and fault-tolerance geometry (integer fingerprints).

/// A basin of attraction in the landscape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Basin {
    /// Basin center fingerprint.
    pub center: u128,
    /// XOR-distance tolerance (`0` ⇒ exact match only).
    pub radius: u32,
}

impl Basin {
    /// Create a basin around an attractor fingerprint.
    #[must_use]
    pub fn new(center: u128, radius: u32) -> Self {
        Self { center, radius }
    }

    /// Whether a state fingerprint is inside this basin.
    #[must_use]
    pub fn contains(&self, state_key: u128) -> bool {
        if self.radius == 0 {
            return state_key == self.center;
        }
        let dist = (state_key ^ self.center).count_ones();
        dist <= self.radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_basin() {
        let b = Basin::new(0xABC, 0);
        assert!(b.contains(0xABC));
        assert!(!b.contains(0xABD));
    }
}
