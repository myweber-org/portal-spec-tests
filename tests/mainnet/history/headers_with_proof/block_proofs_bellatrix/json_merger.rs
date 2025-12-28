use serde_json::{Value, json};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;
        merged_array.push(json_value);
    }

    let output_json = json!(merged_array);
    let output_str = serde_json::to_string_pretty(&output_json)?;

    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, output_str)?;

    Ok(())
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[&str]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let json_data: JsonValue = serde_json::from_str(&content)?;
        
        if let JsonValue::Array(arr) = json_data {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_data);
        }
    }

    Ok(JsonValue::Array(merged_array))
}

pub fn merge_json_with_key(file_paths: &[&str], key: &str) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let content = fs::read_to_string(path)?;
        let json_data: JsonValue = serde_json::from_str(&content)?;

        if let JsonValue::Object(obj) = json_data {
            if let Some(value) = obj.get(key) {
                merged_map.insert(path_str.to_string(), value.clone());
            }
        }
    }

    Ok(JsonValue::Object(serde_json::Map::from_iter(
        merged_map.into_iter()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(&file2, r#"{"id": 3}"#).unwrap();

        let paths = &[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        let result = merge_json_files(paths).unwrap();

        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_merge_json_with_key() {
        let file = NamedTempFile::new().unwrap();
        fs::write(&file, r#"{"user": "alice", "age": 30}"#).unwrap();

        let paths = &[file.path().to_str().unwrap()];
        let result = merge_json_with_key(paths, "user").unwrap();

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key(file.path().to_str().unwrap()));
    }
}use std::collections::HashMap;
use serde_json::{Value, Map};

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
        (base, update) => {
            *base = update.clone();
        }
    }
}

pub fn merge_json_with_strategy(base: &mut Value, update: &Value, strategy: MergeStrategy) {
    match strategy {
        MergeStrategy::Deep => merge_json(base, update),
        MergeStrategy::Shallow => {
            if let (Value::Object(base_map), Value::Object(update_map)) = (base, update) {
                for (key, value) in update_map {
                    base_map.insert(key.clone(), value.clone());
                }
            } else {
                *base = update.clone();
            }
        }
        MergeStrategy::AppendArrays => {
            if let (Value::Array(base_arr), Value::Array(update_arr)) = (base, update) {
                base_arr.extend_from_slice(update_arr);
            } else {
                merge_json(base, update);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    Deep,
    Shallow,
    AppendArrays,
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
                "d": 99,
                "e": 100
            },
            "f": 42
        });
        
        merge_json(&mut base, &update);
        
        assert_eq!(base["a"], 1);
        assert_eq!(base["b"]["c"], 2);
        assert_eq!(base["b"]["d"], 99);
        assert_eq!(base["b"]["e"], 100);
        assert_eq!(base["f"], 42);
    }

    #[test]
    fn test_shallow_merge() {
        let mut base = json!({
            "a": 1,
            "b": {
                "c": 2
            }
        });
        
        let update = json!({
            "b": {
                "d": 99
            }
        });
        
        merge_json_with_strategy(&mut base, &update, MergeStrategy::Shallow);
        
        assert_eq!(base["b"], json!({"d": 99}));
    }

    #[test]
    fn test_array_append() {
        let mut base = json!({
            "items": [1, 2, 3]
        });
        
        let update = json!({
            "items": [4, 5]
        });
        
        merge_json_with_strategy(&mut base, &update, MergeStrategy::AppendArrays);
        
        assert_eq!(base["items"], json!([1, 2, 3, 4, 5]));
    }
}