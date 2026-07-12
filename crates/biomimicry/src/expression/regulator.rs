//! `NetworkRegulator` — real Phase 1 brain behind the M3 `Regulator` seam.

use crate::cell::{Cell, Operation};
use crate::expression::RuleNetwork;
use crate::metabolism::{ExpressionDelta, Regulator};

/// Phase 1 regulator driven by a [`RuleNetwork`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkRegulator {
    /// Rule network evaluated in declaration order.
    pub network: RuleNetwork,
}

impl NetworkRegulator {
    /// Create from a network.
    #[must_use]
    pub fn new(network: RuleNetwork) -> Self {
        Self { network }
    }
}

impl Regulator for NetworkRegulator {
    fn regulate(&self, cell: &Cell, queued: &[Operation]) -> ExpressionDelta {
        self.network.evaluate(cell, queued)
    }
}
