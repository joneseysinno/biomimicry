//! Curated re-exports for downstream consumers (gardening surface).

pub use crate::attractor::{Basin, DivergenceKind, Landscape, SettleStatus};
pub use crate::blocks::{
    Block, BlockId, BlockName, BlockSource, LinkError, Manifest, OrganismGenotype, PortSpec, link,
};
pub use crate::causality::{
    CausalDag, CausalDiff, CausalEventLog, CausalNode, CommitmentGate, Equilibrium, diff_dags,
    log_to_dag,
};
pub use crate::cell::{BehavioralMode, Cell, CellId, LifecycleState};
pub use crate::effector::{EffectorId, EffectorSink, MemoryEffectorSink};
pub use crate::error::{BiomimicryError, Result};
pub use crate::expression::{NetworkRegulator, RegulatoryRule, RuleNetwork};
pub use crate::ganglion::{
    Ganglion, GanglionHandle, GanglionHealth, GanglionPort, GanglionResponse, GanglionView,
    stimulate,
};
pub use crate::genesis::{Cistron, EndpointPolarity, Gene, GeneId, Genome, Grn, Primitive};
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
pub use crate::signal::{
    Phase, Scope, Signal, SignalKind, SignalPayload, SignalScope, SignalType, Value, ValueShape,
};
pub use crate::substrate::{BranchId, MemoryStore, SnapshotId, SnapshotMeta, Store};
pub use crate::transduction::{
    ArithOp, Cascade, CascadeTransducer, FoldSpec, TransductionFn, TransductionSpec,
};
