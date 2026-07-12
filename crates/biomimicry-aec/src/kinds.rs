//! AEC SignalKind vocabulary (Part VIII) — no core domain types.

/// Architect moves a wall (reflex perturbation).
pub const WALL_MOVE: &str = "aec.wall.move";

/// Reflex cascade emissions.
pub const AREA_UPDATED: &str = "aec.area.updated";
/// Door/window clearance re-check.
pub const CLEARANCE_CHECKED: &str = "aec.clearance.checked";
/// Structural load path re-analysis.
pub const LOAD_ANALYZED: &str = "aec.load.analyzed";
/// Material quantities update.
pub const QTY_UPDATED: &str = "aec.qty.updated";
/// Pricing delta.
pub const PRICE_DELTA: &str = "aec.price.delta";

/// Beam overspan conflict (escalation).
pub const BEAM_OVERSPAN: &str = "aec.beam.overspan";

/// Payload meta: wall displacement in milli-meters.
pub const DISPLACE_MILLI: &str = "displace_milli";

/// All reflex recompute emission kinds in cascade order.
pub const RECOMPUTE_KINDS: [&str; 5] = [
    AREA_UPDATED,
    CLEARANCE_CHECKED,
    LOAD_ANALYZED,
    QTY_UPDATED,
    PRICE_DELTA,
];
