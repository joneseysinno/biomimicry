//! Global K-cadence (Phase 2 cycles per Phase 1 cycle).
//!
//! M3 locks one organism-global K (default 10). M6 adds thin
//! [`crate::organism::Organism::effective_k`]: when exactly one non-empty
//! ganglion exists, settle uses that ganglion's `SpaceConfig.k`; otherwise the
//! scheduler cadence K applies. Fine-grained per-op K arbitration remains future work.

use crate::error::{BiomimicryError, Result};

/// How many Phase 2 cycles run per Phase 1 cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cadence {
    /// Phase 2 iterations per Phase 1 tick (K-ratio). Must be ≥ 1.
    pub k: u32,
}

impl Default for Cadence {
    fn default() -> Self {
        Self { k: 10 }
    }
}

impl Cadence {
    /// Create a cadence with the given K.
    ///
    /// # Errors
    ///
    /// Returns [`BiomimicryError::CadenceMisconfigured`] when `k == 0`.
    pub fn try_new(k: u32) -> Result<Self> {
        if k == 0 {
            return Err(BiomimicryError::CadenceMisconfigured { k });
        }
        Ok(Self { k })
    }

    /// Create a cadence with the given K (panics forbidden — use [`Self::try_new`]).
    #[must_use]
    pub const fn new(k: u32) -> Self {
        Self { k }
    }

    /// Validate K ≥ 1.
    ///
    /// # Errors
    ///
    /// Returns `CadenceMisconfigured` when `k == 0`.
    pub fn validate(self) -> Result<()> {
        if self.k == 0 {
            Err(BiomimicryError::CadenceMisconfigured { k: self.k })
        } else {
            Ok(())
        }
    }
}

/// Per-subsystem cadence config — **seam for M6 ganglia**.
///
/// M3 uses only the organism-global [`Cadence`]. At M6, each ganglion may carry
/// its own `SpaceConfig { k }` (e.g. fast-interface K=100, differentiation K=2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpaceConfig {
    /// Subsystem K-ratio.
    pub k: u32,
}
