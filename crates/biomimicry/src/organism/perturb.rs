//! Public perturbation entry point — there is no `run()` / `main()`.

use crate::error::Result;
use crate::organism::Organism;
use crate::signal::Signal;
use crate::substrate::Store;

impl<S: Store> Organism<S> {
    /// Inject a perturbation signal into the organism.
    ///
    /// This is the entire public API surface for "running" the engine.
    ///
    /// # Errors
    ///
    /// Returns an error if the signal cannot be accepted.
    pub fn perturb(&mut self, _signal: Signal) -> Result<()> {
        todo!("inject perturbation into medium / boundary cells")
    }
}
