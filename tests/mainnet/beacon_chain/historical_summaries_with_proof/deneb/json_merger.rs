use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj_map) = json_value {
            for (key, value) in obj_map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_with_strategy(
    file_paths: &[&str],
    conflict_strategy: ConflictStrategy,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut value_tracker: HashMap<String, Vec<Value>> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj_map) = json_value {
            for (key, value) in obj_map {
                value_tracker.entry(key).or_default().push(value);
            }
        }
    }

    let mut result_map = Map::new();
    for (key, values) in value_tracker {
        let resolved_value = match conflict_strategy {
            ConflictStrategy::First => values.first().unwrap().clone(),
            ConflictStrategy::Last => values.last().unwrap().clone(),
            ConflictStrategy::MergeObjects => merge_json_values(values),
        };
        result_map.insert(key, resolved_value);
    }

    Ok(Value::Object(result_map))
}

fn merge_json_values(values: Vec<Value>) -> Value {
    let mut merged = Map::new();
    for value in values {
        if let Value::Object(map) = value {
            for (k, v) in map {
                merged.insert(k, v);
            }
        }
    }
    Value::Object(merged)
}

#[derive(Clone, Copy)]
pub enum ConflictStrategy {
    First,
    Last,
    MergeObjects,
}