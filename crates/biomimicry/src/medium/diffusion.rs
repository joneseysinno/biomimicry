//! Integer-milli graded diffusion (attenuation with relational distance).
//!
//! Binary in/out-of-scope is the default delivery path; this module adds an
//! optional strength that starts full at the source and decrements with hop
//! distance, dropping below a threshold. No floats.

use crate::signal::Scope;

/// Default full strength at the emission source (millis).
pub const FULL_STRENGTH_MILLI: i32 = 1_000;

/// Default drop threshold (millis) — at or below, the signal is ignored.
pub const DROP_THRESHOLD_MILLI: i32 = 0;

/// Attenuation per hop (millis).
pub const ATTENUATION_PER_HOP_MILLI: i32 = 250;

/// Attenuate strength by hop-count distance.
///
/// Returns `None` when the result is at or below [`DROP_THRESHOLD_MILLI`].
#[must_use]
pub fn attenuate_milli(strength_milli: i32, hops: u32, _scope: Scope) -> Option<i32> {
    let loss = ATTENUATION_PER_HOP_MILLI.saturating_mul(i32::try_from(hops).unwrap_or(i32::MAX));
    let next = strength_milli.saturating_sub(loss);
    if next <= DROP_THRESHOLD_MILLI {
        None
    } else {
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attenuates_and_drops() {
        assert_eq!(
            attenuate_milli(FULL_STRENGTH_MILLI, 0, Scope::Neighbors),
            Some(FULL_STRENGTH_MILLI)
        );
        assert_eq!(
            attenuate_milli(FULL_STRENGTH_MILLI, 1, Scope::Neighbors),
            Some(750)
        );
        assert_eq!(attenuate_milli(200, 1, Scope::Neighbors), None);
    }
}
