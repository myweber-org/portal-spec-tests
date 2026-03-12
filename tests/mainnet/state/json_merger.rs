use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(serde_json::Value::Object(merged_map.into_iter().collect()))
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(serde_json::Value::Object(merged_map.into_iter().collect()))
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = merged.get(&key).unwrap();
                    if existing != &value {
                        let resolved = resolve_conflict(&key, existing, &value);
                        merged.insert(key, resolved);
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        }
    }

    let output_json = Value::Object(merged);
    let output_str = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_str)?;

    Ok(())
}

fn resolve_conflict(key: &str, v1: &Value, v2: &Value) -> Value {
    match (v1, v2) {
        (Value::Array(a1), Value::Array(a2)) => {
            let mut combined = a1.clone();
            combined.extend(a2.clone());
            Value::Array(combined)
        },
        (Value::Number(n1), Value::Number(n2)) => {
            if n1.as_f64().unwrap_or(0.0) > n2.as_f64().unwrap_or(0.0) {
                v1.clone()
            } else {
                v2.clone()
            }
        },
        _ => v2.clone()
    }
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashMap::new();

    for path in paths {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;

        match json_data {
            JsonValue::Array(arr) => {
                for item in arr {
                    if let Some(key) = item.get("id").and_then(|v| v.as_str()) {
                        if !seen_keys.contains_key(key) {
                            seen_keys.insert(key.to_string(), true);
                            merged_array.push(item);
                        }
                    } else {
                        merged_array.push(item);
                    }
                }
            }
            JsonValue::Object(obj) => {
                if let Some(key) = obj.get("id").and_then(|v| v.as_str()) {
                    if !seen_keys.contains_key(key) {
                        seen_keys.insert(key.to_string(), true);
                        merged_array.push(JsonValue::Object(obj));
                    }
                } else {
                    merged_array.push(JsonValue::Object(obj));
                }
            }
            _ => {
                merged_array.push(json_data);
            }
        }
    }

    let output_json = JsonValue::Array(merged_array);
    let output_str = serde_json::to_string_pretty(&output_json)?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(output_str.as_bytes())?;

    Ok(())
}

pub fn merge_json_directory<P: AsRef<Path>>(dir_path: P, output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut json_files = Vec::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            json_files.push(path);
        }
    }

    if json_files.is_empty() {
        return Err("No JSON files found in directory".into());
    }

    merge_json_files(&json_files, output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_merge_json_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("data1.json");
        let file2_path = temp_dir.path().join("data2.json");
        let output_path = temp_dir.path().join("merged.json");

        fs::write(&file1_path, r#"[{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]"#).unwrap();
        fs::write(&file2_path, r#"[{"id": "2", "name": "Robert"}, {"id": "3", "name": "Charlie"}]"#).unwrap();

        merge_json_files(&[file1_path, file2_path], &output_path).unwrap();

        let content = fs::read_to_string(output_path).unwrap();
        let parsed: JsonValue = serde_json::from_str(&content).unwrap();

        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        let json: Value = serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }
    
    Ok(Value::Object(merged))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        match target.get_mut(&key) {
            Some(target_value) => {
                if let (Value::Object(target_obj), Value::Object(source_obj)) = (target_value, &source_value) {
                    let mut target_obj = target_obj.clone();
                    merge_objects(&mut target_obj, source_obj.clone());
                    target.insert(key, Value::Object(target_obj));
                } else {
                    target.insert(key, source_value);
                }
            }
            None => {
                target.insert(key, source_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"a": 1, "nested": {"x": 10}}"#).unwrap();
        fs::write(&file2, r#"{"b": 2, "nested": {"y": 20}}"#).unwrap();
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], 2);
        assert_eq!(result["nested"]["x"], 10);
        assert_eq!(result["nested"]["y"], 20);
    }
}