use serde_json::{Map, Value};

pub fn merge_json(base: &mut Value, update: &Value) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, update_value);
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (base, update) => *base = update.clone(),
    }
}

pub fn merge_json_with_strategy(
    base: &mut Value,
    update: &Value,
    strategy: MergeStrategy,
) -> Result<(), String> {
    match strategy {
        MergeStrategy::Deep => {
            merge_json(base, update);
            Ok(())
        }
        MergeStrategy::Shallow => {
            *base = update.clone();
            Ok(())
        }
        MergeStrategy::Custom(merge_fn) => merge_fn(base, update),
    }
}

pub enum MergeStrategy {
    Deep,
    Shallow,
    Custom(fn(&mut Value, &Value) -> Result<(), String>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deep_merge() {
        let mut base = json!({
            "a": 1,
            "b": {
                "c": 2,
                "d": 3
            }
        });

        let update = json!({
            "b": {
                "d": 4,
                "e": 5
            },
            "f": 6
        });

        merge_json(&mut base, &update);

        assert_eq!(
            base,
            json!({
                "a": 1,
                "b": {
                    "c": 2,
                    "d": 4,
                    "e": 5
                },
                "f": 6
            })
        );
    }

    #[test]
    fn test_shallow_merge() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let update = json!({"b": {"d": 3}});

        merge_json_with_strategy(&mut base, &update, MergeStrategy::Shallow)
            .unwrap();

        assert_eq!(base, json!({"b": {"d": 3}}));
    }
}
use serde_json::{Map, Value};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, update: &Value, strategy: MergeStrategy) -> Result<(), String> {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            merge_objects(base_map, update_map, strategy)
        }
        _ => Err("Both values must be JSON objects".to_string()),
    }
}

fn merge_objects(
    base: &mut Map<String, Value>,
    update: &Map<String, Value>,
    strategy: MergeStrategy,
) -> Result<(), String> {
    for (key, update_value) in update {
        match base.get_mut(key) {
            Some(base_value) => {
                if let (Value::Object(base_obj), Value::Object(update_obj)) = (base_value, update_value) {
                    merge_objects(base_obj, update_obj, strategy)?;
                } else {
                    handle_conflict(key, base_value, update_value, strategy)?;
                }
            }
            None => {
                base.insert(key.clone(), update_value.clone());
            }
        }
    }
    Ok(())
}

fn handle_conflict(
    key: &str,
    base: &mut Value,
    update: &Value,
    strategy: MergeStrategy,
) -> Result<(), String> {
    match strategy {
        MergeStrategy::PreferUpdate => *base = update.clone(),
        MergeStrategy::PreferBase => (),
        MergeStrategy::CombineArrays => {
            if let (Value::Array(base_arr), Value::Array(update_arr)) = (base, update) {
                let combined: HashSet<_> = base_arr.iter().chain(update_arr).cloned().collect();
                *base = Value::Array(combined.into_iter().collect());
            } else {
                return Err(format!("Type mismatch for key '{}': cannot combine non-array values", key));
            }
        }
        MergeStrategy::FailOnConflict => {
            return Err(format!("Conflict detected for key '{}'", key));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    PreferUpdate,
    PreferBase,
    CombineArrays,
    FailOnConflict,
}use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> io::Result<Value> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Array(arr) = json_value {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_value);
        }
    }

    Ok(json!(merged_array))
}

pub fn merge_and_write<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> io::Result<()> {
    let merged = merge_json_files(input_paths)?;
    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &merged)?;
    Ok(())
}