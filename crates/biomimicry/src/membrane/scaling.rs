//! Breadth (more surfaces) vs depth (ganglia behind them).

/// How the membrane grows under load.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScalingStrategy {
    /// Add more boundary surfaces.
    Breadth,
    /// Deepen ganglia behind existing surfaces.
    Depth,
}

/// Choose a scaling strategy given load metrics.
#[must_use]
pub fn choose_scaling(_inbound_rate: f64, _depth_pressure: f64) -> ScalingStrategy {
    todo!("choose breadth vs depth scaling")
}
