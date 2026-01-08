
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    conflict_strategy: fn(&str, &Value, &Value) -> Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match merged_map.get(&key) {
                    Some(existing_value) => {
                        let resolved_value = conflict_strategy(&key, existing_value, &value);
                        merged_map.insert(key, resolved_value);
                    }
                    None => {
                        merged_map.insert(key, value);
                    }
                }
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let final_map: Map<String, Value> = merged_map.into_iter().collect();
    Ok(Value::Object(final_map))
}

pub fn default_conflict_strategy(_key: &str, existing: &Value, new: &Value) -> Value {
    if existing.is_array() && new.is_array() {
        let mut combined = existing.as_array().unwrap().clone();
        combined.extend(new.as_array().unwrap().clone());
        Value::Array(combined)
    } else {
        new.clone()
    }
}
use serde_json::{Map, Value};

pub fn merge_json(base: &mut Value, update: &Value, resolve_conflict: bool) -> Result<(), String> {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_val) in update_map {
                if let Some(base_val) = base_map.get_mut(key) {
                    if base_val.is_object() && update_val.is_object() {
                        merge_json(base_val, update_val, resolve_conflict)?;
                    } else if resolve_conflict {
                        *base_val = update_val.clone();
                    } else {
                        return Err(format!("Conflict detected for key: {}", key));
                    }
                } else {
                    base_map.insert(key.clone(), update_val.clone());
                }
            }
            Ok(())
        }
        _ => Err("Both values must be JSON objects".to_string()),
    }
}

pub fn merge_json_with_strategy(
    base: &mut Value,
    update: &Value,
    strategy: MergeStrategy,
) -> Result<(), String> {
    match strategy {
        MergeStrategy::PreferBase => Ok(()),
        MergeStrategy::PreferUpdate => {
            *base = update.clone();
            Ok(())
        }
        MergeStrategy::Recursive(resolve) => merge_json(base, update, resolve),
        MergeStrategy::Custom(func) => func(base, update),
    }
}

pub enum MergeStrategy {
    PreferBase,
    PreferUpdate,
    Recursive(bool),
    Custom(fn(&mut Value, &Value) -> Result<(), String>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_without_conflict() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let update = json!({"b": {"d": 3}, "e": 4});
        
        merge_json(&mut base, &update, false).unwrap();
        assert_eq!(base, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_merge_with_conflict_resolution() {
        let mut base = json!({"a": 1, "b": 2});
        let update = json!({"b": 3, "c": 4});
        
        merge_json(&mut base, &update, true).unwrap();
        assert_eq!(base, json!({"a": 1, "b": 3, "c": 4}));
    }

    #[test]
    fn test_merge_conflict_error() {
        let mut base = json!({"a": 1});
        let update = json!({"a": 2});
        
        let result = merge_json(&mut base, &update, false);
        assert!(result.is_err());
    }
}