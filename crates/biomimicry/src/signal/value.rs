//! Typed value lattice carried by signal payloads.
//!
//! # Integer discipline (millis)
//!
//! [`Value::Int`] is the **only** numeric. By convention values are millis-scaled
//! (thousandths): `1000` means `1.000`. Genes that need a different scale record
//! it in payload metadata (`scale`); the engine never rescales silently.
//!
//! Arithmetic over millis lives in [`crate::transduction::ArithOp`] and uses:
//! - `i128` intermediates
//! - round half away from zero
//! - saturating (never wrapping)
//! - division by zero → [`crate::BiomimicryError::DivideByZero`]
//!
//! # Canonical encoding
//!
//! Tag byte, then length-prefixed fields, little-endian. [`Value::Record`] keys
//! iterate in `BTreeMap` order. Depth is capped at [`MAX_VALUE_DEPTH`].

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use smol_str::SmolStr;

use crate::error::{BiomimicryError, Result};

/// Maximum nesting depth for [`Value::List`] / [`Value::Record`].
pub const MAX_VALUE_DEPTH: u32 = 8;

const TAG_UNIT: u8 = 0;
const TAG_BOOL: u8 = 1;
const TAG_INT: u8 = 2;
const TAG_TEXT: u8 = 3;
const TAG_LIST: u8 = 4;
const TAG_RECORD: u8 = 5;
const TAG_BYTES: u8 = 6;

/// Typed payload content — the authority behind [`super::Payload::body`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Value {
    /// Absence; the identity for fold.
    Unit,
    /// Boolean.
    Bool(bool),
    /// Integer — millis-scaled by convention (see module docs).
    Int(i64),
    /// Labels, option names, escalation prose.
    Text(SmolStr),
    /// Ordered, heterogeneous list.
    List(Vec<Value>),
    /// Canonical key order via [`BTreeMap`] (no hash iteration).
    Record(BTreeMap<SmolStr, Value>),
    /// Escape hatch; opaque, never interpreted by the engine.
    Bytes(Vec<u8>),
}

impl Default for Value {
    fn default() -> Self {
        Self::Unit
    }
}

impl Value {
    /// Construct a text value.
    #[must_use]
    pub fn text(s: impl AsRef<str>) -> Self {
        Self::Text(SmolStr::new(s.as_ref()))
    }

    /// Construct a record from key/value pairs.
    pub fn record_from(
        entries: impl IntoIterator<Item = (impl AsRef<str>, Value)>,
    ) -> Result<Self> {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            map.insert(SmolStr::new(k.as_ref()), v);
        }
        Self::Record(map).checked()
    }

    /// Construct a list, enforcing depth.
    pub fn list(items: Vec<Value>) -> Result<Self> {
        Self::List(items).checked()
    }

    /// Depth of this value (leaves are 1; nesting increments).
    #[must_use]
    pub fn depth(&self) -> u32 {
        match self {
            Self::Unit | Self::Bool(_) | Self::Int(_) | Self::Text(_) | Self::Bytes(_) => 1,
            Self::List(xs) => 1 + xs.iter().map(Value::depth).max().unwrap_or(0),
            Self::Record(m) => 1 + m.values().map(Value::depth).max().unwrap_or(0),
        }
    }

    /// Reject values deeper than [`MAX_VALUE_DEPTH`].
    pub fn check_depth(&self) -> Result<()> {
        let depth = self.depth();
        if depth > MAX_VALUE_DEPTH {
            return Err(BiomimicryError::ValueDepthExceeded { depth });
        }
        Ok(())
    }

    /// Reject values deeper than [`MAX_VALUE_DEPTH`], returning `self` on success.
    pub fn checked(self) -> Result<Self> {
        self.check_depth()?;
        Ok(self)
    }

    /// Canonical encoding (tag + length-prefixed fields, LE).
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.check_depth()?;
        let mut out = Vec::new();
        encode_into(self, &mut out)?;
        Ok(out)
    }

    /// Decode a canonical encoding, checking depth.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut cur = Cursor::new(bytes);
        let value = decode_from(&mut cur, 1)?;
        if cur.position() != bytes.len() as u64 {
            return Err(BiomimicryError::ValueDecode {
                reason: "trailing bytes after value".into(),
            });
        }
        value.checked()
    }

    /// Shape of this value (structure without concrete data).
    #[must_use]
    pub fn shape(&self) -> ValueShape {
        match self {
            Self::Unit => ValueShape::Unit,
            Self::Bool(_) => ValueShape::Bool,
            Self::Int(_) => ValueShape::Int,
            Self::Text(_) => ValueShape::Text,
            Self::List(xs) => ValueShape::List(Box::new(
                xs.first().map_or(ValueShape::Unit, Value::shape),
            )),
            Self::Record(m) => ValueShape::Record(
                m.iter()
                    .map(|(k, v)| (k.clone(), v.shape()))
                    .collect(),
            ),
            Self::Bytes(_) => ValueShape::Bytes,
        }
    }

    /// Borrow as `i64` if this is [`Value::Int`].
    #[must_use]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(n) => Some(*n),
            _ => None,
        }
    }

    /// Borrow as `bool` if this is [`Value::Bool`].
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

/// Structural type of a [`Value`], used for port contracts (M12 link-time).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ValueShape {
    /// [`Value::Unit`].
    Unit,
    /// [`Value::Bool`].
    Bool,
    /// [`Value::Int`].
    Int,
    /// [`Value::Text`].
    Text,
    /// Homogeneous element shape for lists (element shape may be `Unit` when empty).
    List(Box<ValueShape>),
    /// Record field shapes in canonical key order.
    Record(BTreeMap<SmolStr, ValueShape>),
    /// [`Value::Bytes`].
    Bytes,
    /// Accept any value (stimulation / port wildcards).
    Any,
}

impl ValueShape {
    /// Whether `value` conforms to this shape.
    #[must_use]
    pub fn matches(&self, value: &Value) -> bool {
        match (self, value) {
            (Self::Any, _)
            | (Self::Unit, Value::Unit)
            | (Self::Bool, Value::Bool(_))
            | (Self::Int, Value::Int(_))
            | (Self::Text, Value::Text(_))
            | (Self::Bytes, Value::Bytes(_)) => true,
            (Self::List(elem), Value::List(xs)) => xs.iter().all(|v| elem.matches(v)),
            (Self::Record(expect), Value::Record(got)) => {
                expect.len() == got.len()
                    && expect.iter().all(|(k, shape)| {
                        got.get(k).is_some_and(|v| shape.matches(v))
                    })
            }
            _ => false,
        }
    }

    /// Whether an export shape can satisfy an import shape at link time.
    ///
    /// [`ValueShape::Any`] is compatible with everything (either side).
    /// Otherwise shapes must be structurally equal.
    #[must_use]
    pub fn compatible(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Any, _) | (_, Self::Any) => true,
            (Self::List(a), Self::List(b)) => a.compatible(b),
            (Self::Record(a), Self::Record(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .all(|(k, sa)| b.get(k).is_some_and(|sb| sa.compatible(sb)))
            }
            _ => self == other,
        }
    }
}

fn encode_into(value: &Value, out: &mut Vec<u8>) -> Result<()> {
    match value {
        Value::Unit => out.push(TAG_UNIT),
        Value::Bool(b) => {
            out.push(TAG_BOOL);
            out.push(u8::from(*b));
        }
        Value::Int(n) => {
            out.push(TAG_INT);
            out.extend_from_slice(&n.to_le_bytes());
        }
        Value::Text(s) => {
            out.push(TAG_TEXT);
            write_str(out, s.as_str())?;
        }
        Value::List(xs) => {
            out.push(TAG_LIST);
            write_u32(out, u32::try_from(xs.len()).expect("list len fits u32"));
            for x in xs {
                encode_into(x, out)?;
            }
        }
        Value::Record(m) => {
            out.push(TAG_RECORD);
            write_u32(out, u32::try_from(m.len()).expect("record len fits u32"));
            for (k, v) in m {
                write_str(out, k.as_str())?;
                encode_into(v, out)?;
            }
        }
        Value::Bytes(b) => {
            out.push(TAG_BYTES);
            write_u32(out, u32::try_from(b.len()).expect("bytes len fits u32"));
            out.extend_from_slice(b);
        }
    }
    Ok(())
}

fn decode_from(cur: &mut Cursor<&[u8]>, depth: u32) -> Result<Value> {
    if depth > MAX_VALUE_DEPTH {
        return Err(BiomimicryError::ValueDepthExceeded { depth });
    }
    let tag = read_u8(cur)?;
    match tag {
        TAG_UNIT => Ok(Value::Unit),
        TAG_BOOL => {
            let b = read_u8(cur)?;
            match b {
                0 => Ok(Value::Bool(false)),
                1 => Ok(Value::Bool(true)),
                _ => Err(BiomimicryError::ValueDecode {
                    reason: format!("invalid bool tag {b}"),
                }),
            }
        }
        TAG_INT => {
            let mut buf = [0u8; 8];
            read_exact(cur, &mut buf)?;
            Ok(Value::Int(i64::from_le_bytes(buf)))
        }
        TAG_TEXT => Ok(Value::Text(SmolStr::new(read_str(cur)?))),
        TAG_LIST => {
            let n = read_u32(cur)? as usize;
            let mut xs = Vec::with_capacity(n);
            for _ in 0..n {
                xs.push(decode_from(cur, depth + 1)?);
            }
            Ok(Value::List(xs))
        }
        TAG_RECORD => {
            let n = read_u32(cur)? as usize;
            let mut m = BTreeMap::new();
            for _ in 0..n {
                let k = SmolStr::new(read_str(cur)?);
                let v = decode_from(cur, depth + 1)?;
                if m.insert(k.clone(), v).is_some() {
                    return Err(BiomimicryError::ValueDecode {
                        reason: format!("duplicate record key {k}"),
                    });
                }
            }
            Ok(Value::Record(m))
        }
        TAG_BYTES => {
            let n = read_u32(cur)? as usize;
            let mut buf = vec![0u8; n];
            read_exact(cur, &mut buf)?;
            Ok(Value::Bytes(buf))
        }
        other => Err(BiomimicryError::ValueDecode {
            reason: format!("unknown value tag {other}"),
        }),
    }
}

fn write_u32(out: &mut Vec<u8>, n: u32) {
    out.extend_from_slice(&n.to_le_bytes());
}

fn write_str(out: &mut Vec<u8>, s: &str) -> Result<()> {
    write_u32(
        out,
        u32::try_from(s.len()).map_err(|_| BiomimicryError::ValueDecode {
            reason: "string too long".into(),
        })?,
    );
    out.extend_from_slice(s.as_bytes());
    Ok(())
}

fn read_u8(cur: &mut Cursor<&[u8]>) -> Result<u8> {
    let mut buf = [0u8; 1];
    read_exact(cur, &mut buf)?;
    Ok(buf[0])
}

fn read_u32(cur: &mut Cursor<&[u8]>) -> Result<u32> {
    let mut buf = [0u8; 4];
    read_exact(cur, &mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_str(cur: &mut Cursor<&[u8]>) -> Result<String> {
    let n = read_u32(cur)? as usize;
    let mut buf = vec![0u8; n];
    read_exact(cur, &mut buf)?;
    String::from_utf8(buf).map_err(|e| BiomimicryError::ValueDecode {
        reason: format!("invalid utf-8: {e}"),
    })
}

fn read_exact(cur: &mut Cursor<&[u8]>, buf: &mut [u8]) -> Result<()> {
    cur.read_exact(buf).map_err(|_| BiomimicryError::ValueDecode {
        reason: "unexpected end of input".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_round_trip() {
        let v = Value::Unit;
        assert_eq!(Value::decode(&v.encode().unwrap()).unwrap(), v);
    }

    #[test]
    fn millis_int_round_trip() {
        let v = Value::Int(14_000);
        assert_eq!(Value::decode(&v.encode().unwrap()).unwrap(), v);
    }

    #[test]
    fn record_key_order_stable() {
        let mut a = BTreeMap::new();
        a.insert(SmolStr::new("b"), Value::Int(1));
        a.insert(SmolStr::new("a"), Value::Int(2));
        let mut b = BTreeMap::new();
        b.insert(SmolStr::new("a"), Value::Int(2));
        b.insert(SmolStr::new("b"), Value::Int(1));
        assert_eq!(
            Value::Record(a).encode().unwrap(),
            Value::Record(b).encode().unwrap()
        );
    }

    #[test]
    fn depth_exceeded_at_construct() {
        let mut v = Value::Int(1);
        for _ in 0..MAX_VALUE_DEPTH {
            v = Value::List(vec![v]);
        }
        assert!(matches!(
            v.checked(),
            Err(BiomimicryError::ValueDepthExceeded { .. })
        ));
    }

    #[test]
    fn depth_exceeded_at_decode() {
        // Build an over-deep encoding by hand: nested lists beyond MAX.
        let mut bytes = Vec::new();
        for _ in 0..=MAX_VALUE_DEPTH {
            bytes.push(TAG_LIST);
            bytes.extend_from_slice(&1u32.to_le_bytes());
        }
        bytes.push(TAG_UNIT);
        assert!(matches!(
            Value::decode(&bytes),
            Err(BiomimicryError::ValueDepthExceeded { .. })
        ));
    }

    #[test]
    fn shape_matches_int() {
        assert!(ValueShape::Int.matches(&Value::Int(3)));
        assert!(!ValueShape::Int.matches(&Value::Bool(true)));
        assert!(ValueShape::Any.matches(&Value::Unit));
    }
}
