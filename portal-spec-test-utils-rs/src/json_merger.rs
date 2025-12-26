
use serde_json::{Value, Map};
use std::collections::HashSet;

pub enum MergeStrategy {
    PreferFirst,
    PreferSecond,
    CombineArrays,
    DeepMerge,
}

pub fn merge_json(a: &Value, b: &Value, strategy: &MergeStrategy) -> Value {
    match (a, b) {
        (Value::Object(map_a), Value::Object(map_b)) => {
            let mut result = Map::new();
            let keys_a: HashSet<_> = map_a.keys().collect();
            let keys_b: HashSet<_> = map_b.keys().collect();
            
            for key in keys_a.union(&keys_b) {
                let key_str = key.to_string();
                match (map_a.get(key), map_b.get(key)) {
                    (Some(val_a), Some(val_b)) => {
                        result.insert(key_str, merge_json(val_a, val_b, strategy));
                    }
                    (Some(val), None) => {
                        result.insert(key_str, val.clone());
                    }
                    (None, Some(val)) => {
                        result.insert(key_str, val.clone());
                    }
                    _ => unreachable!(),
                }
            }
            Value::Object(result)
        }
        (Value::Array(arr_a), Value::Array(arr_b)) => {
            match strategy {
                MergeStrategy::CombineArrays => {
                    let mut combined = arr_a.clone();
                    combined.extend(arr_b.clone());
                    Value::Array(combined)
                }
                MergeStrategy::DeepMerge => {
                    let mut result = Vec::new();
                    for (i, item_a) in arr_a.iter().enumerate() {
                        if let Some(item_b) = arr_b.get(i) {
                            result.push(merge_json(item_a, item_b, strategy));
                        } else {
                            result.push(item_a.clone());
                        }
                    }
                    for item_b in arr_b.iter().skip(arr_a.len()) {
                        result.push(item_b.clone());
                    }
                    Value::Array(result)
                }
                _ => {
                    match strategy {
                        MergeStrategy::PreferFirst => a.clone(),
                        MergeStrategy::PreferSecond => b.clone(),
                        _ => a.clone(),
                    }
                }
            }
        }
        _ => {
            match strategy {
                MergeStrategy::PreferFirst => a.clone(),
                MergeStrategy::PreferSecond => b.clone(),
                _ => a.clone(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let a = json!({"name": "Alice", "age": 30});
        let b = json!({"age": 31, "city": "London"});
        
        let merged = merge_json(&a, &b, &MergeStrategy::PreferSecond);
        assert_eq!(merged["age"], 31);
        assert_eq!(merged["city"], "London");
        assert_eq!(merged["name"], "Alice");
    }

    #[test]
    fn test_array_combination() {
        let a = json!([1, 2, 3]);
        let b = json!([4, 5, 6]);
        
        let merged = merge_json(&a, &b, &MergeStrategy::CombineArrays);
        assert_eq!(merged.as_array().unwrap().len(), 6);
    }
}use serde_json::{Value, json};
use std::fs;
use std::io;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file_content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&file_content)?;
        
        if let Value::Array(arr) = json_value {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_value);
        }
    }

    Ok(json!(merged_array))
}

pub fn merge_and_write<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> io::Result<()> {
    let merged = merge_json_files(input_paths)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    
    let json_string = serde_json::to_string_pretty(&merged)?;
    fs::write(output_path, json_string)?;
    
    Ok(())
}
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting with value from {}", key, path_str);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Top-level JSON must be an object".into());
        }
    }

    Ok(serde_json::Value::Object(merged_map))
}

pub fn write_merged_json(output_path: &str, json_value: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(json_value)?;
    fs::write(output_path, json_string)?;
    Ok(())
}