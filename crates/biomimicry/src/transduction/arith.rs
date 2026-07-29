//! Millis-scaled [`ArithOp`] evaluation.
//!
//! Locked rules (M11 §0.5):
//! - `i128` intermediates
//! - round half away from zero on scaled multiply / divide
//! - saturating, never wrapping
//! - division by zero → [`crate::BiomimicryError::DivideByZero`]

use crate::error::{BiomimicryError, Result};
use crate::transduction::ArithOp;

/// Round `numer / denom` half away from zero, saturating to [`i64`].
///
/// Division uses truncating toward-zero quotients; when the absolute remainder
/// is at least half of `|denom|`, the quotient moves one unit further from zero.
///
/// # Panics
///
/// Panics if `denom == 0` (callers must guard division separately).
#[must_use]
pub fn round_half_away(numer: i128, denom: i128) -> i64 {
    assert!(denom != 0, "round_half_away: denom must be non-zero");
    let q = numer / denom;
    let r = numer % denom;
    let q = if r != 0 && r.unsigned_abs().saturating_mul(2) >= denom.unsigned_abs() {
        let away = if (numer > 0) == (denom > 0) { 1i128 } else { -1i128 };
        q + away
    } else {
        q
    };
    saturate_i64(q)
}

/// Evaluate a unary [`ArithOp`] (`Neg` / `Abs`).
///
/// # Errors
///
/// Returns [`BiomimicryError::ValueTypeMismatch`] when `op` is not unary.
pub fn eval_unary(op: ArithOp, a: i64) -> Result<i64> {
    match op {
        ArithOp::Neg => Ok(a.saturating_neg()),
        ArithOp::Abs => Ok(saturating_abs(a)),
        other => Err(BiomimicryError::ValueTypeMismatch {
            function: format!("Arith({other:?})"),
            expected: "unary op (Neg|Abs)".into(),
            got: "binary op".into(),
        }),
    }
}

/// Evaluate a binary [`ArithOp`] (`Add` / `Sub` / `Mul` / `Div` / `Min` / `Max`).
///
/// # Errors
///
/// - [`BiomimicryError::DivideByZero`] for [`ArithOp::Div`] with `b == 0`
/// - [`BiomimicryError::ValueTypeMismatch`] when `op` is unary
pub fn eval_binary(op: ArithOp, a: i64, b: i64) -> Result<i64> {
    match op {
        ArithOp::Add => Ok(a.saturating_add(b)),
        ArithOp::Sub => Ok(a.saturating_sub(b)),
        ArithOp::Mul => Ok(round_half_away(i128::from(a) * i128::from(b), 1000)),
        ArithOp::Div => {
            if b == 0 {
                return Err(BiomimicryError::DivideByZero {
                    function: "Arith(Div)".into(),
                });
            }
            Ok(round_half_away(i128::from(a) * 1000, i128::from(b)))
        }
        ArithOp::Min => Ok(a.min(b)),
        ArithOp::Max => Ok(a.max(b)),
        other => Err(BiomimicryError::ValueTypeMismatch {
            function: format!("Arith({other:?})"),
            expected: "binary op".into(),
            got: "unary op (Neg|Abs)".into(),
        }),
    }
}

/// Whether `op` is unary (`Neg` / `Abs`).
#[must_use]
pub fn is_unary(op: ArithOp) -> bool {
    matches!(op, ArithOp::Neg | ArithOp::Abs)
}

fn saturating_abs(a: i64) -> i64 {
    if a == i64::MIN {
        i64::MAX
    } else {
        a.abs()
    }
}

fn saturate_i64(n: i128) -> i64 {
    i64::try_from(n).unwrap_or(if n > 0 { i64::MAX } else { i64::MIN })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mul_millis_half_away() {
        assert_eq!(eval_binary(ArithOp::Mul, 500, 1000).unwrap(), 500);
        assert_eq!(eval_binary(ArithOp::Mul, 500, 1).unwrap(), 1); // 0.5 → 1
        assert_eq!(eval_binary(ArithOp::Mul, -500, 1).unwrap(), -1);
        assert_eq!(eval_binary(ArithOp::Mul, 3000, 2000).unwrap(), 6000);
        // Rounding table (half away from zero).
        assert_eq!(eval_binary(ArithOp::Mul, 1, 500).unwrap(), 1); // 0.5 → 1
        assert_eq!(eval_binary(ArithOp::Mul, 1, 499).unwrap(), 0); // 0.499 → 0
        assert_eq!(eval_binary(ArithOp::Mul, -1, 500).unwrap(), -1);
        assert_eq!(eval_binary(ArithOp::Mul, 7000, 2000).unwrap(), 14000); // A1
    }

    #[test]
    fn div_by_zero() {
        assert!(matches!(
            eval_binary(ArithOp::Div, 1000, 0),
            Err(BiomimicryError::DivideByZero { .. })
        ));
    }

    #[test]
    fn add_saturating() {
        assert_eq!(eval_binary(ArithOp::Add, i64::MAX, 1).unwrap(), i64::MAX);
    }
}
