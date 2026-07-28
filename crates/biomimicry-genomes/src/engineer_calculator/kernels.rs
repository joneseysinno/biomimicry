//! Arithmetic chemistry — the tiny fixed set of [`TransductionFn`] kernels.

use biomimicry::signal::Scope;
use biomimicry::transduction::{
    BinaryMetaOp, TransductionFn, TransductionKernel, UnaryMetaOp,
};

use crate::engineer_calculator::kinds::{META_VALUE, OPERAND_A, OPERAND_B, VALUE};

/// Add kernel: `operand.a + operand.b` → meta `value`, emit [`VALUE`] to Neighbors.
#[must_use]
pub fn kernel_add() -> TransductionFn {
    binary_kernel("calc.add", BinaryMetaOp::Add, VALUE).with_scope(Scope::Neighbors)
}

/// Mul kernel: `operand.a * operand.b` → meta `value`, emit [`VALUE`] to Neighbors.
#[must_use]
pub fn kernel_mul() -> TransductionFn {
    binary_kernel("calc.mul", BinaryMetaOp::Mul, VALUE).with_scope(Scope::Neighbors)
}

/// Negate kernel.
#[must_use]
pub fn kernel_negate() -> TransductionFn {
    unary_kernel("calc.neg", UnaryMetaOp::Negate, VALUE).with_scope(Scope::Neighbors)
}

/// Reciprocal (milli) kernel.
#[must_use]
pub fn kernel_reciprocal() -> TransductionFn {
    unary_kernel("calc.recip", UnaryMetaOp::ReciprocalMilli, VALUE).with_scope(Scope::Neighbors)
}

/// Compare kernel: −1 / 0 / 1.
#[must_use]
pub fn kernel_compare() -> TransductionFn {
    binary_kernel("calc.cmp", BinaryMetaOp::Compare, VALUE).with_scope(Scope::Neighbors)
}

fn binary_kernel(name: &str, op: BinaryMetaOp, out_kind: &str) -> TransductionFn {
    TransductionFn::identity_echo(name, out_kind).with_kernel(TransductionKernel::BinaryMeta {
        op,
        left: OPERAND_A.into(),
        right: OPERAND_B.into(),
        out: META_VALUE.into(),
    })
}

fn unary_kernel(name: &str, op: UnaryMetaOp, out_kind: &str) -> TransductionFn {
    TransductionFn::identity_echo(name, out_kind).with_kernel(TransductionKernel::UnaryMeta {
        op,
        input: OPERAND_A.into(),
        out: META_VALUE.into(),
    })
}
