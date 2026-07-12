//! Causal logical clock; monotonic stamp for signal events.
//!
//! Data + comparison only in M2 — the causal DAG lands at M7. Stamps use
//! integer millis-style discipline (no `SystemTime` / floats).

/// Opaque, monotonically comparable causal timestamp.
///
/// Integer logical counter — replay-stable and DAG-ready for M7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CausalStamp(pub i64);

impl CausalStamp {
    /// Zero stamp.
    #[must_use]
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Successor stamp (`self + 1`), saturating at `i64::MAX`.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl std::fmt::Display for CausalStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "t{}", self.0)
    }
}

/// Logical clock that issues causal stamps.
#[derive(Debug, Clone, Default)]
pub struct CausalClock {
    next: i64,
}

impl CausalClock {
    /// Create a clock starting at zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Issue the next stamp and advance.
    pub fn tick(&mut self) -> CausalStamp {
        let stamp = CausalStamp(self.next);
        self.next = self.next.saturating_add(1);
        stamp
    }

    /// Current counter without advancing.
    #[must_use]
    pub fn peek(&self) -> CausalStamp {
        CausalStamp(self.next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamps_are_monotonic() {
        let mut clock = CausalClock::new();
        let a = clock.tick();
        let b = clock.tick();
        assert!(a < b);
        assert_eq!(a.next(), b);
    }
}
