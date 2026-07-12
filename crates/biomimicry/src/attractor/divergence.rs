//! Divergence and limit-cycle detector over discrete fingerprints.

/// Pathological dynamics detected on a trajectory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DivergenceKind {
    /// Unbounded drift (unique fingerprint count exceeds a cap).
    Diverging,
    /// Sustained oscillation (limit cycle / ringing).
    LimitCycle,
}

/// Detect divergence or limit-cycle behavior on a fingerprint trajectory.
///
/// Limit cycle: a repeating cycle of length ≥ 2 appears in the trailing window
/// (period-2 through period-`max_period`).
/// Diverging: unique fingerprint count in the whole trajectory exceeds
/// `unique_cap` (when provided via [`detect_divergence_with_cap`]).
#[must_use]
pub fn detect_divergence(trajectory: &[u128]) -> Option<DivergenceKind> {
    detect_divergence_with_cap(trajectory, None)
}

/// Like [`detect_divergence`], with an optional unique-fingerprint cap.
#[must_use]
pub fn detect_divergence_with_cap(
    trajectory: &[u128],
    unique_cap: Option<usize>,
) -> Option<DivergenceKind> {
    if let Some(cap) = unique_cap {
        let mut uniq = trajectory.to_vec();
        uniq.sort_unstable();
        uniq.dedup();
        if uniq.len() > cap {
            return Some(DivergenceKind::Diverging);
        }
    }

    if let Some(cycle) = find_limit_cycle(trajectory) {
        let _ = cycle;
        return Some(DivergenceKind::LimitCycle);
    }
    None
}

/// Trailing period-2..8 cycle detection (needs at least 2 full periods).
fn find_limit_cycle(trajectory: &[u128]) -> Option<usize> {
    if trajectory.len() < 4 {
        return None;
    }
    let max_period = 8.min(trajectory.len() / 2);
    for period in 2..=max_period {
        if has_trailing_cycle(trajectory, period) {
            // Reject period that is actually a fixed point (all equal).
            let slice = &trajectory[trajectory.len() - period..];
            if slice.iter().any(|&x| x != slice[0]) {
                return Some(period);
            }
        }
    }
    None
}

fn has_trailing_cycle(trajectory: &[u128], period: usize) -> bool {
    let need = period * 2;
    if trajectory.len() < need {
        return false;
    }
    let end = trajectory.len();
    let a = &trajectory[end - need..end - period];
    let b = &trajectory[end - period..end];
    a == b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_period_2_limit_cycle() {
        let t = [1u128, 2, 1, 2, 1, 2];
        assert_eq!(detect_divergence(&t), Some(DivergenceKind::LimitCycle));
    }

    #[test]
    fn fixed_point_is_not_cycle() {
        let t = [7u128, 7, 7, 7];
        assert_eq!(detect_divergence(&t), None);
    }
}
