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
    let mut accumulator: HashMap<String, Vec<Value>> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                accumulator.entry(key).or_insert_with(Vec::new).push(value);
            }
        }
    }

    let mut result_map = Map::new();
    for (key, values) in accumulator {
        let merged_value = match strategy {
            MergeStrategy::Overwrite => values.last().cloned().unwrap_or(Value::Null),
            MergeStrategy::CombineArrays => {
                let combined: Vec<Value> = values
                    .into_iter()
                    .flat_map(|v| {
                        if let Value::Array(arr) = v {
                            arr
                        } else {
                            vec![v]
                        }
                    })
                    .collect();
                Value::Array(combined)
            }
            MergeStrategy::KeepFirst => values.first().cloned().unwrap_or(Value::Null),
        };
        result_map.insert(key, merged_value);
    }

    Ok(Value::Object(result_map))
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Overwrite,
    CombineArrays,
    KeepFirst,
}