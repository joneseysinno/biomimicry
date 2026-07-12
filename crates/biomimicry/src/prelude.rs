//! Curated re-exports for downstream consumers (gardening surface).

pub use crate::attractor::{Basin, DivergenceKind, Landscape, SettleStatus};
pub use crate::causality::{
    CausalDag, CausalDiff, CausalEventLog, CausalNode, CommitmentGate, Equilibrium, diff_dags,
    log_to_dag,
};
pub use crate::cell::{BehavioralMode, Cell, CellId, LifecycleState};
pub use crate::error::{BiomimicryError, Result};
pub use crate::expression::{NetworkRegulator, RegulatoryRule, RuleNetwork};
pub use crate::ganglion::{Ganglion, GanglionHandle, GanglionHealth, GanglionView};
pub use crate::genesis::{
    EndpointPolarity, Gene, GeneId, Genome, Hypergraph, Primitive, SpatialHypergraph,
};
pub use crate::homeostasis::{
    AttractorStabilityLoop, DampingParams, HomeostaticLoop, PopulationSizeLoop, SignalFluxLoop,
};
pub use crate::medium::Medium;
pub use crate::membrane::{
    BoundaryCellTemplate, EscalationOption, EscalationPacket, MembranePolicy, ResponseMode,
    ScalingStrategy, choose_scaling, classify,
};
pub use crate::metabolism::{
    Cadence, EchoTransducer, ExplicitRegulator, Phase1Brain, Phase2Brain, Population, Regulator,
    Scheduler, SpaceConfig, Transducer,
};
pub use crate::organism::{Organism, OrganismBuilder};
pub use crate::sensorium::{
    ImmuneFlag, ReadoutCollector, SensoryGeneTemplate, SensoryPolicy, SignalSample,
};
pub use crate::signal::{Phase, Scope, Signal, SignalKind, SignalPayload, SignalScope, SignalType};
pub use crate::substrate::{BranchId, MemoryStore, SnapshotId, SnapshotMeta, Store};
pub use crate::transduction::{Cascade, CascadeTransducer, TransductionFn};
