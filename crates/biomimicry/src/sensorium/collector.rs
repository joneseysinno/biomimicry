//! Passive SignalSample readout collector (no global observer).

use crate::signal::Payload;

/// A passive sample captured from sensory emissions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalSample {
    /// Source cell id.
    pub source: u64,
    /// Captured payload.
    pub payload: Payload,
}

/// Passive collector of sensory readouts.
#[derive(Debug, Clone, Default)]
pub struct ReadoutCollector {
    samples: Vec<SignalSample>,
}

impl ReadoutCollector {
    /// Create an empty collector.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a sample if it carries an observation tag.
    pub fn observe(&mut self, sample: SignalSample) {
        if sample.payload.is_observation() {
            self.samples.push(sample);
        }
    }

    /// Drain collected samples.
    pub fn drain(&mut self) -> Vec<SignalSample> {
        std::mem::take(&mut self.samples)
    }

    /// Number of buffered samples.
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Whether empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Borrow buffered samples without draining.
    #[must_use]
    pub fn samples(&self) -> &[SignalSample] {
        &self.samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_ignores_non_observation() {
        let mut c = ReadoutCollector::new();
        c.observe(SignalSample {
            source: 1,
            payload: Payload::empty(),
        });
        assert!(c.is_empty());
        c.observe(SignalSample {
            source: 1,
            payload: Payload::empty().with_observation("note"),
        });
        assert_eq!(c.len(), 1);
    }
}
