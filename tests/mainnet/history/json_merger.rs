use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashMap::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Array(arr) = json_value {
            for item in arr {
                if let Some(obj) = item.as_object() {
                    if let Some(id_value) = obj.get("id") {
                        if let Some(id_str) = id_value.as_str() {
                            if seen_ids.contains_key(id_str) {
                                eprintln!("Duplicate ID '{}' found in {}, skipping.", id_str, file_path);
                                continue;
                            }
                            seen_ids.insert(id_str.to_string(), ());
                        }
                    }
                }
                merged_array.push(item);
            }
        } else {
            return Err("Each JSON file must contain a JSON array at its root.".into());
        }
    }

    let output_json = json!(merged_array);
    fs::write(output_path, output_json.to_string())?;
    println!("Successfully merged {} files into {}", file_paths.len(), output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = r#"[{"id": "a", "data": 1}, {"id": "b", "data": 2}]"#;
        let file2_content = r#"[{"id": "c", "data": 3}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let paths = vec![file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        let result = merge_json_files(&paths, output_file.path().to_str().unwrap());

        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let json: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                merge_value(&mut merged, key, value);
            }
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }

    Ok(Value::Object(merged))
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get(&key) {
        Some(Value::Object(existing_obj)) => {
            if let Value::Object(new_obj) = new_value {
                let mut merged_obj = existing_obj.clone();
                for (nested_key, nested_value) in new_obj {
                    merge_value(&mut merged_obj, nested_key, nested_value);
                }
                map.insert(key, Value::Object(merged_obj));
            } else {
                map.insert(key, new_value);
            }
        }
        Some(Value::Array(existing_arr)) => {
            if let Value::Array(new_arr) = new_value {
                let mut merged_arr = existing_arr.clone();
                merged_arr.extend(new_arr);
                map.insert(key, Value::Array(merged_arr));
            } else {
                map.insert(key, new_value);
            }
        }
        _ => {
            map.insert(key, new_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_objects() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#).unwrap();
        fs::write(&file2, r#"{"b": {"y": 20}, "c": 3}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": {"x": 10, "y": 20},
            "c": 3
        });

        assert_eq!(result, expected);
    }
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
            merge_objects(&mut merged, obj);
        }
    }
    
    let output_json = Value::Object(merged);
    let output_str = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_str)?;
    
    Ok(())
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, value) in source {
        if let Some(existing) = target.get_mut(&key) {
            if let (Value::Object(mut target_obj), Value::Object(source_obj)) = (existing, value) {
                merge_objects(&mut target_obj, source_obj);
                *existing = Value::Object(target_obj);
            } else {
                *existing = value;
            }
        } else {
            target.insert(key, value);
        }
    }
}