//! Crate error and result types.

use crate::cell::LifecycleState;
use crate::genesis::{DistanceMode, EndpointPolarity, GeneId, PrimitiveNodeId};
use crate::signal::Phase;
use thiserror::Error;

/// Errors produced by the biomimicry engine.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BiomimicryError {
    /// An illegal cell lifecycle transition was requested.
    #[error("illegal lifecycle transition: {from:?} → {to:?}")]
    IllegalLifecycleTransition {
        /// Source state.
        from: LifecycleState,
        /// Requested target state.
        to: LifecycleState,
    },

    /// An operation was dispatched to a dead cell.
    #[error("dead cell dispatch")]
    DeadCellDispatch,

    /// Operation execution is not available until a later milestone.
    #[error("operation `{op}` unavailable until milestone {since_milestone}")]
    OperationUnavailable {
        /// Operation name.
        op: &'static str,
        /// Milestone that will implement execution.
        since_milestone: u32,
    },

    /// An energy budget pool is exhausted for the requested phase.
    #[error("budget exhausted for {phase:?}")]
    BudgetExhausted {
        /// Which phase budget failed.
        phase: Phase,
    },

    /// No active receptor matched the inbound signal.
    #[error("receptor mismatch")]
    ReceptorMismatch,

    /// Scope cannot be resolved until a later milestone (e.g. `Cluster` → M6).
    #[error("scope `{scope:?}` unavailable until milestone {since_milestone}")]
    ScopeUnavailable {
        /// Scope that could not be resolved.
        scope: crate::signal::Scope,
        /// Milestone that will implement resolution.
        since_milestone: u32,
    },

    /// Cadence K must be ≥ 1.
    #[error("cadence misconfigured: k={k}")]
    CadenceMisconfigured {
        /// Invalid K value.
        k: u32,
    },

    /// Hyperedge failed structural validation.
    #[error("malformed hyperedge: {reason}")]
    MalformedHyperedge {
        /// Human-readable reason.
        reason: String,
    },

    /// Endpoint references a node absent from the hypergraph.
    #[error("dangling endpoint: node {node:?}")]
    DanglingEndpoint {
        /// Missing node id.
        node: PrimitiveNodeId,
    },

    /// Exact `(node, polarity, role, scope)` tuple appears twice on one hyperedge.
    #[error("duplicate endpoint: node {node:?} polarity {polarity:?}")]
    DuplicateEndpoint {
        /// Node involved.
        node: PrimitiveNodeId,
        /// Polarity involved.
        polarity: EndpointPolarity,
    },

    /// Hyperedge `kind` was empty.
    #[error("empty hyperedge kind")]
    EmptyKind,

    /// Requested distance mode is not available yet.
    #[error("distance unavailable: {mode:?}")]
    DistanceUnavailable {
        /// Which mode was requested.
        mode: DistanceMode,
    },

    /// Genome / DNA compilation failed after validation.
    #[error("compile failed: {reason}")]
    CompileFailed {
        /// Human-readable reason.
        reason: String,
    },

    /// Genome / DNA compilation failed (legacy string wrapper).
    #[error("genesis error: {0}")]
    Genesis(String),

    /// Scheduling invariant violated.
    #[error("metabolism error: {0}")]
    Metabolism(String),

    /// Persistence / store failure.
    #[error("substrate error: {0}")]
    Substrate(String),

    /// Snapshot id is unknown to the store.
    #[error("unknown snapshot {0:?}")]
    SnapshotUnknown(crate::substrate::SnapshotId),

    /// Checkpoint blocked because the commitment gate is closed.
    #[error("commitment gate closed")]
    CommitmentGateClosed,

    /// Organism configuration or perturbation failure.
    #[error("organism error: {0}")]
    Organism(String),

    /// Generic placeholder until subsystems fill in typed variants.
    #[error("not yet implemented: {0}")]
    Unimplemented(&'static str),

    /// Phase 1 rule network evaluation failed.
    #[error("rule evaluation failed: {reason}")]
    RuleEvalFailed {
        /// Human-readable reason.
        reason: String,
    },

    /// No cascade is registered for an active gene that requested transduction.
    #[error("cascade unavailable for gene {gene:?}")]
    CascadeUnavailable {
        /// Gene that lacked a cascade body.
        gene: GeneId,
    },
}

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, BiomimicryError>;
