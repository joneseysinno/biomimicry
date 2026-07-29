//! Resolve declarative [`TransductionSpec`] into runtime [`Cascade`] / [`TransductionFn`].

use crate::signal::Payload;
use crate::transduction::{
    Cascade, TransductionFn, TransductionFnSpec, TransductionKernel, TransductionKind,
    TransductionSpec,
};

/// Convert a declarative function spec into a runtime [`TransductionFn`].
#[must_use]
pub fn fn_from_spec(spec: &TransductionFnSpec) -> TransductionFn {
    let payload_template = match &spec.kind {
        TransductionKind::IdentityEcho { payload_template } => payload_template.clone(),
        _ => Payload::empty(),
    };
    TransductionFn {
        name: spec.name.clone(),
        kind: spec.kind.clone(),
        output_kind: spec.output_kind.clone(),
        output_scope: spec.output_scope,
        payload_template,
        enabled: spec.enabled,
        kernel: TransductionKernel::Identity,
    }
}

/// Build a pipeline [`Cascade`] from a declarative [`TransductionSpec`].
#[must_use]
pub fn cascade_from_spec(spec: &TransductionSpec) -> Cascade {
    let mut cascade = Cascade::new();
    for step in &spec.steps {
        cascade = cascade.with_step(fn_from_spec(step));
    }
    cascade
}
