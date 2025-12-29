
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let merged_json = Value::Object(merged_map);
    let json_string = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, json_string)?;

    Ok(())
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    output_path: &str,
    conflict_strategy: ConflictStrategy,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut value_map: HashMap<String, Value> = HashMap::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        value_map.insert(key, value);
                    }
                    ConflictStrategy::Skip => {
                        value_map.entry(key).or_insert(value);
                    }
                    ConflictStrategy::MergeObjects => {
                        if let Some(existing) = value_map.get_mut(&key) {
                            if let (Value::Object(existing_map), Value::Object(new_map)) = (existing, &value) {
                                let mut merged = existing_map.clone();
                                for (k, v) in new_map {
                                    merged.insert(k.clone(), v.clone());
                                }
                                *existing = Value::Object(merged);
                            } else {
                                value_map.insert(key, value);
                            }
                        } else {
                            value_map.insert(key, value);
                        }
                    }
                }
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let final_object: Map<String, Value> = value_map.into_iter().collect();
    let merged_json = Value::Object(final_object);
    let json_string = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, json_string)?;

    Ok(())
}

#[derive(Clone, Copy)]
pub enum ConflictStrategy {
    Overwrite,
    Skip,
    MergeObjects,
}