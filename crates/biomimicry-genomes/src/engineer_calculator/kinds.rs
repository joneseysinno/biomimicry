//! Engineering-calculator SignalKind vocabulary.

/// Operator: addition.
pub const OP_ADD: &str = "calc.op.add";
/// Operator: multiplication.
pub const OP_MUL: &str = "calc.op.mul";
/// Operator: negation.
pub const OP_NEG: &str = "calc.op.neg";
/// Operator: reciprocal.
pub const OP_RECIP: &str = "calc.op.recip";
/// Operator: compare.
pub const OP_CMP: &str = "calc.op.cmp";

/// Value-ready kind (operand / reduced subexpression).
pub const VALUE: &str = "calc.value";
/// Final readout kind.
pub const RESULT: &str = "calc.result";

/// Immune: division by zero.
pub const ERROR_DIVZERO: &str = "calc.error.divzero";
/// Immune: type / parse error.
pub const ERROR_TYPE: &str = "calc.error.type";

/// Gate receptor roles (coincidence operands).
pub const OPERAND_A: &str = "operand.a";
/// Second gate operand role.
pub const OPERAND_B: &str = "operand.b";

/// Payload meta tag for a numeric result.
pub const META_VALUE: &str = "value";

/// Schema stamp role embedded in DNA (`genome_stamp` cistron).
pub const SCHEMA_STAMP: &str = "engineer_calculator.v1";

/// All binary operator kinds shipped in this genome.
pub const BINARY_OPS: [&str; 2] = [OP_ADD, OP_MUL];
