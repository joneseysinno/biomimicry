//! State-space terrain model for computation-as-relaxation (integer depths).

use std::collections::BTreeMap;

/// Abstract energy / fitness landscape over organism fingerprints.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Landscape {
    /// Fingerprint → milli-potential depth (higher = less settled).
    pub samples: BTreeMap<u128, u32>,
    /// Default potential when a fingerprint is unregistered.
    pub default_potential: u32,
}

impl Landscape {
    /// Create an empty landscape (default potential 1000).
    #[must_use]
    pub fn new() -> Self {
        Self {
            samples: BTreeMap::new(),
            default_potential: 1000,
        }
    }

    /// Register a sample depth for a fingerprint.
    pub fn insert(&mut self, state_key: u128, milli_potential: u32) {
        self.samples.insert(state_key, milli_potential);
    }

    /// Estimate potential energy of a state fingerprint (milli-units).
    #[must_use]
    pub fn potential(&self, state_key: u128) -> u32 {
        self.samples
            .get(&state_key)
            .copied()
            .unwrap_or(self.default_potential)
    }
}
