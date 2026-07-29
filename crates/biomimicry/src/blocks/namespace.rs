//! Total qualification rewrite over a DNA fragment.

use crate::blocks::block::Block;
use crate::blocks::name::BlockName;
use crate::blocks::port_spec::LocalKind;
use crate::genesis::{Cistron, Primitive, Role};
use crate::signal::SignalKind;
use crate::transduction::{TransductionFnSpec, TransductionKind, TransductionSpec};

/// Roles that are mechanism verbs, not signal-kind match keys — left unqualified.
const MECHANISM_ROLES: &[&str] = &["activate", "suppress", "produce", "inhibit", "out", "in"];

/// Qualify every Receptor± / Signal± role and every [`TransductionSpec`] kind
/// in `block` under `block.name`. Returns a new block (identity unchanged —
/// qualification is a link-time rewrite of a *copy*).
#[must_use]
pub fn qualify_block(block: &Block) -> Block {
    let name = block.name.as_str();
    let mut out = block.clone();
    for cistron in &mut out.cistrons {
        qualify_cistron(cistron, name);
    }
    out
}

/// Qualify a single cistron in place.
pub fn qualify_cistron(cistron: &mut Cistron, block: &str) {
    for ep in &mut cistron.endpoints {
        if matches!(ep.primitive, Primitive::Receptor | Primitive::Signal)
            && should_qualify_role(ep.role.as_str())
        {
            let local = ep.role.as_str();
            if !local.contains("::") {
                ep.role = Role::new(SignalKind::qualified(block, local).as_str());
            }
        }
    }
    if let Some(spec) = &mut cistron.transduction {
        *spec = qualify_spec(spec, block);
    }
}

fn should_qualify_role(role: &str) -> bool {
    !MECHANISM_ROLES.contains(&role)
}

/// Qualify all output kinds in a transduction spec.
#[must_use]
pub fn qualify_spec(spec: &TransductionSpec, block: &str) -> TransductionSpec {
    TransductionSpec {
        steps: spec
            .steps
            .iter()
            .map(|s| qualify_fn_spec(s, block))
            .collect(),
    }
}

fn qualify_fn_spec(step: &TransductionFnSpec, block: &str) -> TransductionFnSpec {
    let mut out = step.clone();
    out.output_kind = qualify_kind(&step.output_kind, block);
    if let TransductionKind::Fanout(children) = &step.kind {
        out.kind = TransductionKind::Fanout(
            children
                .iter()
                .map(|c| qualify_fn_spec(c, block))
                .collect(),
        );
    }
    out
}

fn qualify_kind(kind: &SignalKind, block: &str) -> SignalKind {
    if kind.is_qualified() || MECHANISM_ROLES.contains(&kind.as_str()) || kind.as_str() == "effect"
    {
        return kind.clone();
    }
    SignalKind::qualified(block, kind.as_str())
}

/// Assert P4: no unqualified Receptor/Signal role or transduction kind remains.
#[must_use]
pub fn assert_qualification_total(cistrons: &[Cistron]) -> bool {
    for c in cistrons {
        for ep in &c.endpoints {
            if matches!(ep.primitive, Primitive::Receptor | Primitive::Signal)
                && should_qualify_role(ep.role.as_str())
                && !ep.role.as_str().contains("::")
            {
                return false;
            }
        }
        if let Some(spec) = &c.transduction {
            if !spec_kinds_qualified(spec) {
                return false;
            }
        }
    }
    true
}

fn spec_kinds_qualified(spec: &TransductionSpec) -> bool {
    spec.steps.iter().all(fn_spec_qualified)
}

fn fn_spec_qualified(step: &TransductionFnSpec) -> bool {
    let kind_ok = step.output_kind.is_qualified()
        || MECHANISM_ROLES.contains(&step.output_kind.as_str())
        || step.output_kind.as_str() == "effect";
    if !kind_ok {
        return false;
    }
    match &step.kind {
        TransductionKind::Fanout(children) => children.iter().all(fn_spec_qualified),
        _ => true,
    }
}

/// Apply manifest renames to a block's local port surface and DNA roles.
pub fn apply_renames(block: &mut Block, renames: &[(LocalKind, LocalKind)]) {
    for (from, to) in renames {
        for port in block.imports.iter_mut().chain(block.exports.iter_mut()) {
            if port.local_kind == *from {
                port.local_kind = to.clone();
            }
        }
        for cistron in &mut block.cistrons {
            for ep in &mut cistron.endpoints {
                if ep.role.as_str() == from.as_str() {
                    ep.role = Role::new(to.as_str());
                }
            }
            if let Some(spec) = &mut cistron.transduction {
                for step in &mut spec.steps {
                    rename_in_step(step, from, to);
                }
            }
        }
    }
}

fn rename_in_step(step: &mut TransductionFnSpec, from: &LocalKind, to: &LocalKind) {
    if step.output_kind.as_str() == from.as_str() {
        step.output_kind = SignalKind::new(to.as_str());
    }
    if let TransductionKind::Fanout(children) = &mut step.kind {
        for child in children {
            rename_in_step(child, from, to);
        }
    }
}

/// Collect renames for one block from a rename list.
#[must_use]
pub fn renames_for(block: &BlockName, all: &[crate::blocks::manifest::Rename]) -> Vec<(LocalKind, LocalKind)> {
    all.iter()
        .filter(|r| &r.block == block)
        .map(|r| (r.from.clone(), r.to.clone()))
        .collect()
}
