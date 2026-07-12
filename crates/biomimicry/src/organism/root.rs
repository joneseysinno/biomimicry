//! The organism aggregate — owns population, ganglia, medium, scheduler, clock.

use std::sync::Arc;

use crate::causality::CausalClock;
use crate::cell::Cell;
use crate::ganglion::Ganglion;
use crate::genesis::Genome;
use crate::medium::Delivery;
use crate::metabolism::Scheduler;
use crate::substrate::Store;

/// The thing you instantiate and **perturb**.
///
/// There is no `run()` and no orchestrator cell inside it.
#[derive(Debug)]
pub struct Organism<S: Store> {
    /// Compiled genome (shared read-only catalog).
    pub genome: Arc<Genome>,
    /// Living cell population.
    pub cells: Vec<Cell>,
    /// Named ganglia.
    pub ganglia: Vec<Ganglion>,
    /// Signaling medium.
    pub medium: Delivery,
    /// Two-phase scheduler.
    pub scheduler: Scheduler,
    /// Causal logical clock.
    pub clock: CausalClock,
    /// Persistence backend.
    pub store: S,
}
