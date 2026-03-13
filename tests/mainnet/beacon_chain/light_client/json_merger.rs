
use std::collections::HashMap;
use std::fs;
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[&str]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_data: JsonValue = serde_json::from_str(&content)?;

        if let JsonValue::Object(map) = json_data {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Duplicate key '{}' found in {}", key, path_str);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain an object at the root".into());
        }
    }

    let merged_json = JsonValue::Object(
        merged_map
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect()
    );

    Ok(merged_json)
}

pub fn write_merged_json(output_path: &str, json_value: &JsonValue) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(json_value)?;
    fs::write(output_path, json_string)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = r#"{"name": "test", "count": 42}"#;
        let file2_content = r#"{"enabled": true, "tags": ["a", "b"]}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        assert!(result.get("name").is_some());
        assert!(result.get("enabled").is_some());
        assert_eq!(result["count"], 42);
        assert_eq!(result["enabled"], true);
    }
}use serde_json::{Value, Map};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, extension: &Value, overwrite_arrays: bool) {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(ext_map)) => {
            for (key, ext_value) in ext_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, ext_value, overwrite_arrays);
                } else {
                    base_map.insert(key.clone(), ext_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(ext_arr)) => {
            if overwrite_arrays {
                *base_arr = ext_arr.clone();
            } else {
                let mut seen = HashSet::new();
                for item in base_arr.iter() {
                    if let Value::Object(map) = item {
                        if let Some(Value::String(id)) = map.get("id") {
                            seen.insert(id.clone());
                        }
                    }
                }
                
                for item in ext_arr {
                    if let Value::Object(map) = item {
                        if let Some(Value::String(id)) = map.get("id") {
                            if !seen.contains(id) {
                                base_arr.push(item.clone());
                                seen.insert(id.clone());
                            }
                        } else {
                            base_arr.push(item.clone());
                        }
                    } else {
                        base_arr.push(item.clone());
                    }
                }
            }
        }
        (base, extension) => {
            if !overwrite_arrays || !extension.is_array() {
                *base = extension.clone();
            }
        }
    }
}

pub fn merge_json_with_strategy(
    base: &Value,
    extension: &Value,
    strategy: MergeStrategy
) -> Value {
    let mut result = base.clone();
    merge_json(&mut result, extension, strategy.overwrite_arrays());
    result
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Shallow,
    Deep,
    DeepPreserveArrays,
}

impl MergeStrategy {
    fn overwrite_arrays(&self) -> bool {
        match self {
            MergeStrategy::Shallow => true,
            MergeStrategy::Deep => true,
            MergeStrategy::DeepPreserveArrays => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shallow_merge() {
        let base = json!({"a": 1, "b": {"inner": 2}});
        let ext = json!({"b": {"new": 3}, "c": 4});
        let result = merge_json_with_strategy(&base, &ext, MergeStrategy::Shallow);
        
        assert_eq!(result["b"], json!({"new": 3}));
        assert_eq!(result["c"], 4);
    }

    #[test]
    fn test_deep_merge_preserve_arrays() {
        let base = json!({
            "items": [{"id": "1", "value": "a"}],
            "config": {"timeout": 30}
        });
        let ext = json!({
            "items": [{"id": "2", "value": "b"}],
            "config": {"retries": 3}
        });
        
        let result = merge_json_with_strategy(&base, &ext, MergeStrategy::DeepPreserveArrays);
        let items = result["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(result["config"]["timeout"], 30);
        assert_eq!(result["config"]["retries"], 3);
    }
}use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::io;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> io::Result<Value> {
    let mut merged_array = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let parsed: Value = serde_json::from_str(&content)?;
        
        if let Value::Array(arr) = parsed {
            merged_array.extend(arr);
        } else {
            merged_array.push(parsed);
        }
    }

    Ok(json!(merged_array))
}

pub fn write_merged_json<P: AsRef<Path>>(paths: &[P], output_path: P) -> io::Result<()> {
    let merged = merge_json_files(paths)?;
    let json_string = serde_json::to_string_pretty(&merged)?;
    fs::write(output_path, json_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_objects() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"id": 1, "name": "test1"}"#).unwrap();
        fs::write(&file2, r#"{"id": 2, "name": "test2"}"#).unwrap();
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        
        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["id"], 1);
        assert_eq!(arr[1]["name"], "test2");
    }

    #[test]
    fn test_merge_json_arrays() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"[{"a": 1}, {"b": 2}]"#).unwrap();
        fs::write(&file2, r#"[{"c": 3}, {"d": 4}]"#).unwrap();
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let arr = result.as_array().unwrap();
        
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0]["a"], 1);
        assert_eq!(arr[3]["d"], 4);
    }
}