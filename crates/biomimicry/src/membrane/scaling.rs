//! Breadth (more surfaces) vs depth (ganglia behind them).

/// How the membrane grows under load.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScalingStrategy {
    /// Add more boundary surfaces.
    Breadth,
    /// Deepen ganglia behind existing surfaces.
    Depth,
}

/// Choose a scaling strategy given milli load metrics.
///
/// - Depth if `depth_pressure_milli >= 500`
/// - else Breadth (including when inbound rate is high — grow surface by default)
#[must_use]
pub fn choose_scaling(inbound_rate_milli: u32, depth_pressure_milli: u32) -> ScalingStrategy {
    let _ = inbound_rate_milli;
    if depth_pressure_milli >= 500 {
        ScalingStrategy::Depth
    } else {
        // High inbound_rate_milli (≥ 500) also maps to Breadth — same arm by design.
        ScalingStrategy::Breadth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p2_choose_scaling_milli_thresholds() {
        assert_eq!(choose_scaling(0, 500), ScalingStrategy::Depth);
        assert_eq!(choose_scaling(500, 0), ScalingStrategy::Breadth);
        assert_eq!(choose_scaling(100, 100), ScalingStrategy::Breadth);
        assert_eq!(choose_scaling(999, 499), ScalingStrategy::Breadth);
    }
}
