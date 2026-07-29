//! Crate error and result types.

use crate::cell::LifecycleState;
use crate::genesis::{DistanceMode, EndpointPolarity, GeneId, PrimitiveNodeId};
use crate::signal::Phase;
use thiserror::Error;

pub use crate::blocks::LinkError;

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

    /// Cistron failed structural validation.
    #[error("malformed cistron: {reason}")]
    MalformedCistron {
        /// Human-readable reason.
        reason: String,
    },

    /// Endpoint references a node absent from the GRN.
    #[error("dangling endpoint: node {node:?}")]
    DanglingEndpoint {
        /// Missing node id.
        node: PrimitiveNodeId,
    },

    /// Exact `(node, polarity, role, scope)` tuple appears twice on one cistron.
    #[error("duplicate endpoint: node {node:?} polarity {polarity:?}")]
    DuplicateEndpoint {
        /// Node involved.
        node: PrimitiveNodeId,
        /// Polarity involved.
        polarity: EndpointPolarity,
    },

    /// Cistron `kind` was empty.
    #[error("empty cistron kind")]
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

    /// Transduction received a value of the wrong type.
    #[error("value type mismatch in `{function}`: expected {expected}, got {got}")]
    ValueTypeMismatch {
        /// Function / op name.
        function: String,
        /// Expected shape description.
        expected: String,
        /// Actual shape description.
        got: String,
    },

    /// Nested [`crate::signal::Value`] exceeded [`crate::signal::MAX_VALUE_DEPTH`].
    #[error("value depth {depth} exceeds maximum")]
    ValueDepthExceeded {
        /// Observed depth.
        depth: u32,
    },

    /// Canonical value encoding could not be decoded.
    #[error("value decode failed: {reason}")]
    ValueDecode {
        /// Human-readable reason.
        reason: String,
    },

    /// Integer division by zero in a transduction op.
    #[error("divide by zero in `{function}`")]
    DivideByZero {
        /// Function / op name.
        function: String,
    },

    /// Ganglion input port has no matching member cells.
    #[error("port unsatisfied: kind `{kind}`")]
    PortUnsatisfied {
        /// Port signal kind.
        kind: String,
    },

    /// Value shape at a port did not match the port contract.
    #[error("port shape mismatch: kind `{kind}`: expected {expected}, got {got}")]
    PortShapeMismatch {
        /// Port signal kind.
        kind: String,
        /// Expected shape.
        expected: String,
        /// Actual shape.
        got: String,
    },

    /// `stimulate` was re-entered from inside a cascade (forbidden).
    #[error("stimulate re-entered")]
    StimulateReentered,

    /// Effector sink persistence is not available yet.
    #[error("sink persistence unavailable until milestone {since_milestone}")]
    SinkPersistenceUnavailable {
        /// Milestone that will implement durable sinks.
        since_milestone: u32,
    },

    /// Version solving is not available yet (exact pins only in M12).
    #[error("version solve unavailable until milestone {since_milestone}")]
    VersionSolveUnavailable {
        /// Milestone that will implement a solver.
        since_milestone: u32,
    },
}

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, BiomimicryError>;
