//! Auto-generated ganglion templates from block import/export surfaces.

use crate::blocks::block::Block;
use crate::blocks::name::BlockName;
use crate::blocks::port_spec::PortSpec;
use crate::ganglion::{GanglionPort, PortDirection};
use crate::signal::SignalKind;

/// Template for a ganglion instantiated at organism genesis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GanglionTemplate {
    /// Block name (also the ganglion name).
    pub name: BlockName,
    /// Input ports (from imports, qualified).
    pub input_ports: Vec<GanglionPort>,
    /// Output ports (from exports, qualified).
    pub output_ports: Vec<GanglionPort>,
    /// Membership capacity hint.
    pub capacity: u32,
}

impl GanglionTemplate {
    /// Derive a template from a (pre-qualification) block; kinds are qualified here.
    #[must_use]
    pub fn from_block(block: &Block, capacity: u32) -> Self {
        let name = block.name.as_str();
        Self {
            name: block.name.clone(),
            input_ports: block
                .imports
                .iter()
                .map(|p| port_from_spec(name, p, PortDirection::In))
                .collect(),
            output_ports: block
                .exports
                .iter()
                .map(|p| port_from_spec(name, p, PortDirection::Out))
                .collect(),
            capacity,
        }
    }

    /// All ports (inputs then outputs).
    #[must_use]
    pub fn ports(&self) -> Vec<GanglionPort> {
        let mut v = self.input_ports.clone();
        v.extend(self.output_ports.iter().cloned());
        v
    }
}

fn port_from_spec(block: &str, spec: &PortSpec, direction: PortDirection) -> GanglionPort {
    let kind = SignalKind::qualified(block, spec.local_kind.as_str());
    match direction {
        PortDirection::In => GanglionPort::input(kind, spec.scope, spec.shape.clone()),
        PortDirection::Out => GanglionPort::output(kind, spec.scope, spec.shape.clone()),
    }
}

/// Build templates for every block (sorted by name for determinism).
#[must_use]
pub fn templates_from_blocks(blocks: &[Block], capacity: u32) -> Vec<GanglionTemplate> {
    let mut blocks: Vec<&Block> = blocks.iter().collect();
    blocks.sort_by(|a, b| a.name.cmp(&b.name));
    blocks
        .into_iter()
        .map(|b| GanglionTemplate::from_block(b, capacity))
        .collect()
}
