
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }
    
    Ok(Value::Object(merged))
}

fn merge_objects(base: &mut Map<String, Value>, new: Map<String, Value>) {
    for (key, new_value) in new {
        match base.get_mut(&key) {
            Some(existing_value) => {
                if let (Value::Object(mut existing_obj), Value::Object(new_obj)) = (existing_value.clone(), new_value) {
                    merge_objects(&mut existing_obj, new_obj);
                    base.insert(key, Value::Object(existing_obj));
                } else if existing_value != &new_value {
                    base.insert(key, new_value);
                }
            }
            None => {
                base.insert(key, new_value);
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
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"common": "value1", "unique1": true}"#).unwrap();
        fs::write(&file2, r#"{"common": "value2", "unique2": 42}"#).unwrap();
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        
        assert_eq!(result["common"], json!("value2"));
        assert_eq!(result["unique1"], json!(true));
        assert_eq!(result["unique2"], json!(42));
    }
}
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

use serde_json::{json, Value};

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), String> {
    let mut merged_array = Vec::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("Input file not found: {}", path_str));
        }

        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| e.to_string())?;

        let json_value: Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(_) => {
                merged_array.push(json_value);
            }
            _ => {
                return Err(format!("Unsupported JSON structure in file: {}", path_str));
            }
        }
    }

    let output_value = Value::Array(merged_array);
    let output_json = serde_json::to_string_pretty(&output_value).map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(output_json.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn deduplicate_json_array_by_key(json_array: Vec<Value>, key: &str) -> Vec<Value> {
    let mut seen = HashMap::new();
    let mut result = Vec::new();

    for item in json_array {
        if let Some(obj) = item.as_object() {
            if let Some(value) = obj.get(key) {
                let key_string = value.to_string();
                if !seen.contains_key(&key_string) {
                    seen.insert(key_string.clone(), true);
                    result.push(item);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#;
        let file2_content = r#"[{"id": 3, "name": "Charlie"}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let input_paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&input_paths, output_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);
    }

    #[test]
    fn test_deduplicate_json_array_by_key() {
        let json_array = vec![
            json!({"id": 1, "name": "Alice"}),
            json!({"id": 2, "name": "Bob"}),
            json!({"id": 1, "name": "Alice Duplicate"}),
            json!({"id": 3, "name": "Charlie"}),
        ];

        let deduplicated = deduplicate_json_array_by_key(json_array, "id");
        assert_eq!(deduplicated.len(), 3);

        let ids: Vec<i64> = deduplicated
            .iter()
            .filter_map(|v| v.get("id").and_then(|id| id.as_i64()))
            .collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(ids.contains(&3));
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    let mut key_sources = Map::new();

    for (index, path) in paths.iter().enumerate() {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing_source = key_sources.get(&key).and_then(|v| v.as_u64()).unwrap_or(0);
                    let conflict_resolution = resolve_conflict(&key, &merged[&key], &value, existing_source as usize, index);
                    
                    match conflict_resolution {
                        ConflictResolution::KeepExisting => continue,
                        ConflictResolution::UseNew(new_value) => {
                            merged.insert(key.clone(), new_value);
                            key_sources.insert(key.clone(), Value::from(index as u64));
                        }
                        ConflictResolution::MergeArrays => {
                            if let (Value::Array(mut existing), Value::Array(new)) = (&merged[&key], &value) {
                                let mut combined = existing.clone();
                                combined.extend(new.clone());
                                let unique_values: Vec<Value> = combined.into_iter().collect::<HashSet<_>>().into_iter().collect();
                                merged.insert(key.clone(), Value::from(unique_values));
                                key_sources.insert(key.clone(), Value::from(vec![existing_source, index as u64]));
                            }
                        }
                    }
                } else {
                    merged.insert(key.clone(), value);
                    key_sources.insert(key.clone(), Value::from(index as u64));
                }
            }
        }
    }

    let result = Value::Object(merged);
    let output = serde_json::to_string_pretty(&result)?;
    fs::write(output_path, output)?;

    Ok(())
}

enum ConflictResolution {
    KeepExisting,
    UseNew(Value),
    MergeArrays,
}

fn resolve_conflict(key: &str, existing: &Value, new: &Value, source1: usize, source2: usize) -> ConflictResolution {
    match (existing, new) {
        (Value::Array(_), Value::Array(_)) => ConflictResolution::MergeArrays,
        (Value::Number(a), Value::Number(b)) if a == b => ConflictResolution::KeepExisting,
        (Value::String(a), Value::String(b)) if a == b => ConflictResolution::KeepExisting,
        (Value::Bool(a), Value::Bool(b)) if a == b => ConflictResolution::KeepExisting,
        _ => {
            eprintln!("Conflict for key '{}':", key);
            eprintln!("  Source {}: {}", source1 + 1, existing);
            eprintln!("  Source {}: {}", source2 + 1, new);
            eprintln!("  Using value from source {}", source2 + 1);
            ConflictResolution::UseNew(new.clone())
        }
    }
}