
use serde_json::{Map, Value};
use std::collections::HashSet;

pub enum ConflictResolution {
    PreferFirst,
    PreferSecond,
    MergeArrays,
    FailOnConflict,
}

pub fn merge_json(
    first: &Value,
    second: &Value,
    strategy: ConflictResolution,
) -> Result<Value, String> {
    match (first, second) {
        (Value::Object(map1), Value::Object(map2)) => {
            merge_objects(map1, map2, strategy)
        }
        (Value::Array(arr1), Value::Array(arr2)) => {
            merge_arrays(arr1, arr2, strategy)
        }
        _ => {
            if first == second {
                Ok(first.clone())
            } else {
                handle_scalar_conflict(first, second, strategy)
            }
        }
    }
}

fn merge_objects(
    map1: &Map<String, Value>,
    map2: &Map<String, Value>,
    strategy: ConflictResolution,
) -> Result<Value, String> {
    let mut result = Map::new();
    let keys1: HashSet<_> = map1.keys().collect();
    let keys2: HashSet<_> = map2.keys().collect();

    for key in keys1.union(&keys2) {
        let val1 = map1.get(*key);
        let val2 = map2.get(*key);

        match (val1, val2) {
            (Some(v1), Some(v2)) => {
                let merged = merge_json(v1, v2, strategy.clone())?;
                result.insert((*key).clone(), merged);
            }
            (Some(v), None) | (None, Some(v)) => {
                result.insert((*key).clone(), v.clone());
            }
            _ => unreachable!(),
        }
    }

    Ok(Value::Object(result))
}

fn merge_arrays(
    arr1: &[Value],
    arr2: &[Value],
    strategy: ConflictResolution,
) -> Result<Value, String> {
    match strategy {
        ConflictResolution::MergeArrays => {
            let mut merged = arr1.to_vec();
            merged.extend_from_slice(arr2);
            Ok(Value::Array(merged))
        }
        ConflictResolution::PreferFirst => Ok(Value::Array(arr1.to_vec())),
        ConflictResolution::PreferSecond => Ok(Value::Array(arr2.to_vec())),
        ConflictResolution::FailOnConflict => {
            if arr1 == arr2 {
                Ok(Value::Array(arr1.to_vec()))
            } else {
                Err("Array conflict detected".to_string())
            }
        }
    }
}

fn handle_scalar_conflict(
    first: &Value,
    second: &Value,
    strategy: ConflictResolution,
) -> Result<Value, String> {
    match strategy {
        ConflictResolution::PreferFirst => Ok(first.clone()),
        ConflictResolution::PreferSecond => Ok(second.clone()),
        ConflictResolution::MergeArrays => {
            let merged = vec![first.clone(), second.clone()];
            Ok(Value::Array(merged))
        }
        ConflictResolution::FailOnConflict => {
            Err(format!("Conflict between {:?} and {:?}", first, second))
        }
    }
}