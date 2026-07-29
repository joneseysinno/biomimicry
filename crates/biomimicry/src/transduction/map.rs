//! Apply [`MapSpec`] structural transforms to [`Value`].

use std::collections::BTreeMap;

use smol_str::SmolStr;

use crate::error::{BiomimicryError, Result};
use crate::signal::Value;
use crate::transduction::MapSpec;

/// Apply `spec` to `value`.
///
/// For [`MapSpec::Set`] / [`MapSpec::Append`] with `value: None`, `companion`
/// supplies the written / appended value.
///
/// # Errors
///
/// Returns [`BiomimicryError::ValueTypeMismatch`] on wrong shapes or missing keys /
/// indices / companions.
pub fn apply_map(
    spec: &MapSpec,
    value: &Value,
    companion: Option<&Value>,
    function: &str,
) -> Result<Value> {
    match spec {
        MapSpec::Get { key } => {
            let record = expect_record(value, function)?;
            record.get(key).cloned().ok_or_else(|| BiomimicryError::ValueTypeMismatch {
                function: function.into(),
                expected: format!("Record with key `{key}`"),
                got: "missing key".into(),
            })
        }
        MapSpec::Set {
            key,
            value: literal,
        } => {
            let mut record = expect_record(value, function)?.clone();
            let written = match literal {
                Some(v) => v.clone(),
                None => companion.cloned().ok_or_else(|| BiomimicryError::ValueTypeMismatch {
                    function: function.into(),
                    expected: "companion value for Set".into(),
                    got: "none".into(),
                })?,
            };
            record.insert(key.clone(), written);
            Value::Record(record).checked()
        }
        MapSpec::Rename { from, to } => {
            let mut record = expect_record(value, function)?.clone();
            let v = record.remove(from).ok_or_else(|| BiomimicryError::ValueTypeMismatch {
                function: function.into(),
                expected: format!("Record with key `{from}`"),
                got: "missing key".into(),
            })?;
            record.insert(to.clone(), v);
            Value::Record(record).checked()
        }
        MapSpec::Project { keys } => {
            let record = expect_record(value, function)?;
            let mut out = BTreeMap::new();
            for key in keys {
                if let Some(v) = record.get(key) {
                    out.insert(key.clone(), v.clone());
                }
            }
            Value::Record(out).checked()
        }
        MapSpec::Index { index } => {
            let list = expect_list(value, function)?;
            let i = *index as usize;
            list.get(i).cloned().ok_or_else(|| BiomimicryError::ValueTypeMismatch {
                function: function.into(),
                expected: format!("List index {index}"),
                got: format!("len {}", list.len()),
            })
        }
        MapSpec::Append { value: literal } => {
            let mut items = expect_list(value, function)?.clone();
            let item = match literal {
                Some(v) => v.clone(),
                None => companion.cloned().ok_or_else(|| BiomimicryError::ValueTypeMismatch {
                    function: function.into(),
                    expected: "companion value for Append".into(),
                    got: "none".into(),
                })?,
            };
            items.push(item);
            Value::list(items)
        }
    }
}

fn expect_record<'a>(
    value: &'a Value,
    function: &str,
) -> Result<&'a BTreeMap<SmolStr, Value>> {
    match value {
        Value::Record(m) => Ok(m),
        other => Err(BiomimicryError::ValueTypeMismatch {
            function: function.into(),
            expected: "Record".into(),
            got: value_ty(other).into(),
        }),
    }
}

fn expect_list<'a>(value: &'a Value, function: &str) -> Result<&'a Vec<Value>> {
    match value {
        Value::List(xs) => Ok(xs),
        other => Err(BiomimicryError::ValueTypeMismatch {
            function: function.into(),
            expected: "List".into(),
            got: value_ty(other).into(),
        }),
    }
}

fn value_ty(v: &Value) -> &'static str {
    match v {
        Value::Unit => "Unit",
        Value::Bool(_) => "Bool",
        Value::Int(_) => "Int",
        Value::Text(_) => "Text",
        Value::List(_) => "List",
        Value::Record(_) => "Record",
        Value::Bytes(_) => "Bytes",
    }
}
