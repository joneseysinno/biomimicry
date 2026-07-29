//! Declared import/export surface of a block.

use smol_str::SmolStr;

use crate::signal::{Scope, SignalKind, ValueShape};

/// Block-local kind name (`"cost"`, `"total"`) — unqualified.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalKind(pub SmolStr);

impl LocalKind {
    /// Construct from any stringy value.
    #[must_use]
    pub fn new(kind: impl AsRef<str>) -> Self {
        Self(SmolStr::new(kind.as_ref()))
    }

    /// Borrow the local kind.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for LocalKind {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl AsRef<str> for LocalKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Fully qualified kind after linking (`"structural::cost"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct QualifiedKind(pub SignalKind);

impl QualifiedKind {
    /// Qualify a local kind under a block name.
    #[must_use]
    pub fn new(block: &str, local: &LocalKind) -> Self {
        Self(SignalKind::qualified(block, local.as_str()))
    }

    /// Wrap an already-qualified [`SignalKind`].
    #[must_use]
    pub fn from_kind(kind: SignalKind) -> Self {
        Self(kind)
    }

    /// Borrow as [`SignalKind`].
    #[must_use]
    pub fn as_kind(&self) -> &SignalKind {
        &self.0
    }

    /// Borrow the label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// One import or export port on a block's signal-kind surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortSpec {
    /// Local (unqualified) kind name.
    pub local_kind: LocalKind,
    /// Value shape contract.
    pub shape: ValueShape,
    /// Delivery scope for the port.
    pub scope: Scope,
    /// When true, an unsatisfied import is a warning rather than an error.
    pub optional: bool,
}

impl PortSpec {
    /// Required import/export with the given shape and scope.
    #[must_use]
    pub fn required(local: impl Into<LocalKind>, shape: ValueShape, scope: Scope) -> Self {
        Self {
            local_kind: local.into(),
            shape,
            scope,
            optional: false,
        }
    }

    /// Optional import (graceful degradation).
    #[must_use]
    pub fn optional(local: impl Into<LocalKind>, shape: ValueShape, scope: Scope) -> Self {
        Self {
            local_kind: local.into(),
            shape,
            scope,
            optional: true,
        }
    }

    /// Convenience: required `Int` at `Cluster` scope.
    #[must_use]
    pub fn int(local: impl Into<LocalKind>) -> Self {
        Self::required(local, ValueShape::Int, Scope::Cluster)
    }
}
