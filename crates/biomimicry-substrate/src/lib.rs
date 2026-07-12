//! `infinite-db`-backed [`biomimicry::substrate::Store`] implementation.
//!
//! Scaffold only (M0). The real wiring lands at Milestone 7.

use biomimicry::error::{BiomimicryError, Result};

#[cfg(feature = "infinite-db")]
use infinite_db as _;

/// Store backed by `infinite-db` v0.4.0.
///
/// Enabled when this crate is built with the `infinite-db` feature (default).
#[derive(Debug, Default)]
pub struct InfiniteDbStore {
    _private: (),
}

impl InfiniteDbStore {
    /// Open (or create) an infinite-db-backed store at `path`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened.
    pub fn open(_path: &str) -> Result<Self> {
        Err(BiomimicryError::Unimplemented(
            "InfiniteDbStore::open — lands at Milestone 7",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_is_stubbed() {
        let err = InfiniteDbStore::open("unused").expect_err("stub");
        assert!(matches!(err, BiomimicryError::Unimplemented(_)));
    }
}
