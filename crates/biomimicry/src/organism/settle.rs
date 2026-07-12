//! Drive the scheduler until convergence or timeout.

use crate::attractor::SettleStatus;
use crate::error::Result;
use crate::organism::Organism;
use crate::substrate::Store;

impl<S: Store> Organism<S> {
    /// Drive metabolism until the organism settles or the timeout elapses.
    ///
    /// # Errors
    ///
    /// Returns an error if scheduling or homeostasis fails.
    pub fn settle(&mut self, _max_ticks: u64) -> Result<SettleStatus> {
        todo!("tick scheduler until convergence or timeout")
    }
}
