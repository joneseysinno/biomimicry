//! Collective lifecycle of a ganglion.

/// Aggregate health of a bounded cell population.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GanglionHealth {
    /// Population within homeostatic bounds.
    #[default]
    Healthy,
    /// Degraded but recoverable.
    Degraded,
    /// Collective failure / dissolution.
    Dead,
}
