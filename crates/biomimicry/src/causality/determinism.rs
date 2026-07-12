//! Seeded, replayable ordering for the scheduler drain path.
//!
//! Under the `determinism` feature (default), harvest/drain never depends on
//! `HashMap`/`HashSet` iteration. Order key = `(CausalStamp, SignalId)` with a
//! stable `CellId` tie-break. The seed drives an integer PRNG for genuine
//! choice points only (splitmix64-style — no floats).

use std::cmp::Ordering;

use crate::cell::CellId;
use crate::signal::{CausalStamp, Signal, SignalId};

/// Deterministic integer PRNG (splitmix64).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Prng {
    state: u64,
}

impl Prng {
    /// Seed the generator.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Next `u64` in the sequence.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// Deterministic ordering key derived from seed + stamp (legacy helper).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrderKey(pub u64);

/// Compute a deterministic delivery order key.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn order_key(seed: u64, stamp: CausalStamp) -> OrderKey {
    OrderKey(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (stamp.0 as u64))
}

/// Sort stamps into seeded delivery order.
pub fn sort_delivery(seed: u64, stamps: &mut [CausalStamp]) {
    stamps.sort_by_key(|s| order_key(seed, *s));
}

/// Total order for signals: `(stamp, id)`.
#[must_use]
pub fn by_causal_order(a: &Signal, b: &Signal) -> Ordering {
    (a.stamp, a.id).cmp(&(b.stamp, b.id))
}

/// Compare two `(stamp, id)` pairs.
#[must_use]
pub fn cmp_stamp_id(a: (CausalStamp, SignalId), b: (CausalStamp, SignalId)) -> Ordering {
    a.cmp(&b)
}

/// Stable `CellId` order for harvest (never hash iteration).
#[must_use]
pub fn cmp_cell_id(a: CellId, b: CellId) -> Ordering {
    a.cmp(&b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::{Payload, Scope, SignalType};

    #[test]
    fn prng_reproducible_and_independent() {
        let mut a = Prng::new(42);
        let mut b = Prng::new(42);
        let mut c = Prng::new(43);
        let seq_a: Vec<_> = (0..8).map(|_| a.next_u64()).collect();
        let seq_b: Vec<_> = (0..8).map(|_| b.next_u64()).collect();
        let seq_c: Vec<_> = (0..8).map(|_| c.next_u64()).collect();
        assert_eq!(seq_a, seq_b);
        assert_ne!(seq_a, seq_c);
    }

    #[test]
    fn causal_order_uses_stamp_then_id() {
        let s0 = Signal::new(
            SignalType::Operational,
            "a",
            Scope::SelfCell,
            Payload::empty(),
            CellId(1),
            CausalStamp(0),
        );
        let s1 = Signal::new(
            SignalType::Operational,
            "a",
            Scope::SelfCell,
            Payload::empty(),
            CellId(1),
            CausalStamp(1),
        );
        assert_eq!(by_causal_order(&s0, &s1), Ordering::Less);
        assert_eq!(by_causal_order(&s0, &s0), Ordering::Equal);
    }
}
