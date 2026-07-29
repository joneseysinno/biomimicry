//! Blocks — the genome linker (compose DNA fragments before `compile`).
//!
//! Declarations and re-exports only — no types or logic live here.
//!
//! An organism is assemblable from named DNA fragments. The linker is
//! **upstream** of [`crate::genesis::compile()`]: qualify · resolve · bridge ·
//! relocate · merge · validate → one GRN. There is no `block.run()` /
//! `block.call()` / `invoke`.

pub mod block;
pub mod bridge;
pub mod debug;
pub mod deliverable;
pub mod error;
pub mod fixture;
pub mod ganglion_template;
pub mod link;
pub mod manifest;
pub mod name;
pub mod namespace;
pub mod port_spec;
#[cfg(test)]
mod props;
pub mod relocate;
pub mod requires;
pub mod resolve;
pub mod source;
pub mod validate;

pub use block::*;
pub use bridge::{BridgeFragment, BridgeInfo, synthesise_bridges};
pub use debug::to_dot;
pub use deliverable::linked_organism;
pub use error::*;
pub use fixture::{
    alt_total_block, geometry_reflex_block, missing_manifest, mistyped_blocks, mistyped_manifest,
    pipeline_blocks, pipeline_manifest, scale_block, sink_block, sum_block,
};
pub use ganglion_template::*;
pub use link::{
    Linked, compile_reached_count, link, link_and_compile, note_compile_reached,
    reset_compile_counter,
};
pub use manifest::*;
pub use name::*;
pub use port_spec::*;
pub use resolve::{ResolveResult, ResolvedWire, resolve};
pub use source::*;
pub use validate::{ValidationReport, validate_pre_merge};
