//! Public perturbation entry point — there is no `run()` / `main()`.

use crate::cell::Operation;
use crate::error::{BiomimicryError, Result};
use crate::medium::ScheduledOp;
use crate::organism::Organism;
use crate::signal::Signal;
use crate::substrate::Store;

impl<S: Store> Organism<S> {
    /// Inject a perturbation signal into the organism.
    ///
    /// Delivers a `Receive` onto the lowest living `CellId` (boundary cell).
    ///
    /// # Errors
    ///
    /// Returns an error if no living cell can accept the signal.
    pub fn perturb(&mut self, signal: Signal) -> Result<()> {
        let cell = self
            .population
            .cells()
            .iter()
            .find(|c| c.lifecycle() != crate::cell::LifecycleState::Dead)
            .map(|c| c.id)
            .ok_or_else(|| BiomimicryError::Organism("no living cell to perturb".into()))?;
        self.scheduler.inject(ScheduledOp {
            cell,
            op: Operation::Receive(signal),
        });
        Ok(())
    }
}
