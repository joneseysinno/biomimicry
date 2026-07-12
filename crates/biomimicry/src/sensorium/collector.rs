//! Passive SignalSample readout collector (no global observer).

use crate::signal::Payload;

/// A passive sample captured from sensory emissions.
#[derive(Debug, Clone)]
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
    pub fn observe(&mut self, _sample: SignalSample) {
        todo!("record observation-tagged sensory sample")
    }

    /// Drain collected samples.
    pub fn drain(&mut self) -> Vec<SignalSample> {
        std::mem::take(&mut self.samples)
    }
}
