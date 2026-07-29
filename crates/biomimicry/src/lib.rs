//! # biomimicry
//!
//! Biological computing engine: computation as *settling* in a living gene
//! regulatory network (GRN).
//!
//! There is no `main()` and no orchestrator — you build an [`organism::Organism`]
//! and [`organism::Organism::perturb`] it, then wait for it to
//! [`organism::Organism::settle`].
//!
//! Engine DNA types use biological names ([`genesis::Cistron`], [`genesis::Grn`]);
//! infinite-db's `Hyperedge` / `Space` vocabulary is translated only in
//! `biomimicry-substrate`.
//!
//! ## Module map
//!
//! | Module | Role |
//! |--------|------|
//! | [`genesis`] | DNA — gene regulatory network (GRN) + genome |
//! | [`cell`] | Relational automaton |
//! | [`ganglion`] | Bounded cell population |
//! | [`signal`] | Regulatory / operational signals |
//! | [`metabolism`] | Two-phase scheduler |
//! | [`medium`] | Signaling delivery / diffusion |
//! | [`expression`] | Phase 1 rule network |
//! | [`transduction`] | Phase 2 cascades |
//! | [`effector`] | Phase 2 writes leaving the signal stream |
//! | [`blocks`] | Genome linker — compose DNA fragments |
//! | [`homeostasis`] | Negative-feedback loops |
//! | [`attractor`] | Landscape / convergence |
//! | [`membrane`] | Boundary / interface model |
//! | [`causality`] | Clocks, DAG, determinism |
//! | [`substrate`] | `Store` trait + in-memory impl |
//! | [`sensorium`] | Sensory templates + readout |
//! | [`organism`] | Aggregate root you perturb |
//!
//! ## Convention
//!
//! Every `<module>.rs` holds only docs, `mod` declarations, and `pub use` re-exports.
//! Types and functions live in leaf files named for a single concern. No `mod.rs`.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod attractor;
pub mod blocks;
pub mod causality;
pub mod cell;
pub mod effector;
pub mod error;
pub mod expression;
pub mod ganglion;
pub mod genesis;
pub mod homeostasis;
pub mod medium;
pub mod membrane;
pub mod metabolism;
pub mod organism;
pub mod prelude;
pub mod sensorium;
pub mod signal;
pub mod substrate;
pub mod transduction;

#[cfg(test)]
mod props_m10;
#[cfg(test)]
mod smoke {
    use crate::substrate::{MemoryStore, Store};

    #[test]
    fn scaffold_memory_store_constructs() {
        let store = MemoryStore::new();
        let hg = store.load_grn().expect("load");
        assert!(hg.edges().next().is_none());
    }
}
