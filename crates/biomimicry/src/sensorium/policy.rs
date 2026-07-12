//! Mechanical sensory acceptance policy (milli-integer; not a cell type).

use crate::causality::CausalStamp;

/// Per-cell sensory gate: threshold + refractory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SensoryPolicy {
    /// Minimum payload `strength_milli` to accept (0 = off).
    pub threshold_milli: u32,
    /// Minimum stamp delta between accepts (0 = off).
    pub refractory_millis: u32,
    /// Stamp of last accepted signal.
    pub last_accept_stamp: Option<CausalStamp>,
}

impl SensoryPolicy {
    /// Create a policy with threshold and refractory (both milli).
    #[must_use]
    pub fn new(threshold_milli: u32, refractory_millis: u32) -> Self {
        Self {
            threshold_milli,
            refractory_millis,
            last_accept_stamp: None,
        }
    }

    /// Whether an inbound signal should be accepted.
    #[must_use]
    pub fn accepts(&self, strength_milli: u32, stamp: CausalStamp) -> bool {
        if self.threshold_milli > 0 && strength_milli < self.threshold_milli {
            return false;
        }
        if self.refractory_millis > 0 {
            if let Some(last) = self.last_accept_stamp {
                let delta = stamp.0.saturating_sub(last.0);
                if let Ok(d) = u32::try_from(delta) {
                    if d < self.refractory_millis {
                        return false;
                    }
                } else if delta < 0 {
                    return false;
                }
            }
        }
        true
    }

    /// Record a successful accept.
    pub fn record_accept(&mut self, stamp: CausalStamp) {
        self.last_accept_stamp = Some(stamp);
    }
}
