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

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    strategy: MergeStrategy,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                strategy.apply(&mut accumulator, key, value);
            }
        }
    }

    let final_map: Map<String, Value> = accumulator.into_iter().collect();
    Ok(Value::Object(final_map))
}

pub enum MergeStrategy {
    Overwrite,
    KeepFirst,
    MergeArrays,
}

impl MergeStrategy {
    fn apply(&self, acc: &mut HashMap<String, Value>, key: String, new_value: Value) {
        match self {
            MergeStrategy::Overwrite => {
                acc.insert(key, new_value);
            }
            MergeStrategy::KeepFirst => {
                acc.entry(key).or_insert(new_value);
            }
            MergeStrategy::MergeArrays => {
                let entry = acc.entry(key).or_insert_with(|| Value::Array(Vec::new()));
                if let (Value::Array(existing), Value::Array(new_arr)) = (entry, &new_value) {
                    let mut combined = existing.clone();
                    combined.extend(new_arr.clone());
                    *entry = Value::Array(combined);
                } else {
                    *entry = new_value;
                }
            }
        }
    }
}
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let json: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = merged.get(&key).unwrap();
                    if existing != &value {
                        return Err(format!("Conflict detected for key '{}'", key));
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }

    Ok(Value::Object(merged))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, value: &Value) -> Result<(), String> {
    let json_string = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(output_path, json_string).map_err(|e| e.to_string())?;
    Ok(())
}