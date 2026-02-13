use serde_json::{Map, Value};
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

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let data1 = json!({
            "name": "test",
            "count": 42
        });

        let data2 = json!({
            "enabled": true,
            "tags": ["rust", "json"]
        });

        write!(file1, "{}", data1.to_string()).unwrap();
        write!(file2, "{}", data2.to_string()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "test",
            "count": 42,
            "enabled": true,
            "tags": ["rust", "json"]
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_missing_file() {
        let mut file = NamedTempFile::new().unwrap();
        let data = json!({"key": "value"});
        write!(file, "{}", data.to_string()).unwrap();

        let paths = vec![
            file.path().to_str().unwrap(),
            "non_existent_file.json",
        ];

        let result = merge_json_files(&paths).unwrap();
        assert_eq!(result, data);
    }
}use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let parsed: Value = serde_json::from_str(&content)?;

        match parsed {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(_) => {
                merged_array.push(parsed);
            }
            _ => {
                eprintln!("Warning: File {} does not contain a JSON object or array, skipping.", file_path);
            }
        }
    }

    let output_value = json!(merged_array);
    let output_json = serde_json::to_string_pretty(&output_value)?;
    fs::write(output_path, output_json)?;

    Ok(())
}use serde_json::{Map, Value};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, update: &Value, strategy: MergeStrategy) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    if strategy == MergeStrategy::Deep {
                        merge_json(base_value, update_value, strategy);
                    } else {
                        *base_value = update_value.clone();
                    }
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            match strategy {
                MergeStrategy::Deep => {
                    let max_len = base_arr.len().max(update_arr.len());
                    for i in 0..max_len {
                        if i < base_arr.len() && i < update_arr.len() {
                            merge_json(&mut base_arr[i], &update_arr[i], strategy);
                        } else if i >= base_arr.len() {
                            base_arr.push(update_arr[i].clone());
                        }
                    }
                }
                MergeStrategy::Shallow => {
                    base_arr.extend(update_arr.iter().cloned());
                }
                MergeStrategy::Unique => {
                    let mut seen = HashSet::new();
                    let mut new_arr = Vec::new();
                    
                    for item in base_arr.iter().chain(update_arr.iter()) {
                        let serialized = serde_json::to_string(item).unwrap_or_default();
                        if !seen.contains(&serialized) {
                            seen.insert(serialized);
                            new_arr.push(item.clone());
                        }
                    }
                    
                    *base_arr = Value::Array(new_arr);
                }
            }
        }
        (base, update) => {
            *base = update.clone();
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum MergeStrategy {
    Shallow,
    Deep,
    Unique,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shallow_merge() {
        let mut base = json!({"a": 1, "b": {"inner": 2}});
        let update = json!({"b": {"new": 3}, "c": 4});
        
        merge_json(&mut base, &update, MergeStrategy::Shallow);
        
        assert_eq!(base["b"], json!({"new": 3}));
        assert_eq!(base["c"], 4);
    }

    #[test]
    fn test_deep_merge() {
        let mut base = json!({"a": 1, "b": {"inner": 2, "keep": 5}});
        let update = json!({"b": {"inner": 3, "new": 4}});
        
        merge_json(&mut base, &update, MergeStrategy::Deep);
        
        assert_eq!(base["b"]["inner"], 3);
        assert_eq!(base["b"]["keep"], 5);
        assert_eq!(base["b"]["new"], 4);
    }

    #[test]
    fn test_unique_array_merge() {
        let mut base = json!([1, 2, 3]);
        let update = json!([3, 4, 5]);
        
        merge_json(&mut base, &update, MergeStrategy::Unique);
        
        let result_arr = base.as_array().unwrap();
        assert_eq!(result_arr.len(), 5);
        assert!(result_arr.contains(&json!(1)));
        assert!(result_arr.contains(&json!(5)));
    }
}