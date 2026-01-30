
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                merge_value(&mut merged, key, value);
            }
        }
    }

    Ok(Value::Object(merged))
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get_mut(&key) {
        Some(existing) => {
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, &new_value) {
                for (nested_key, nested_value) in new_obj {
                    merge_value(existing_obj, nested_key.clone(), nested_value.clone());
                }
            } else if let (Value::Array(existing_arr), Value::Array(new_arr)) = (existing, &new_value) {
                existing_arr.extend(new_arr.clone());
                existing_arr.sort();
                existing_arr.dedup();
            } else {
                map.insert(key, new_value);
            }
        }
        None => {
            map.insert(key, new_value);
        }
    }
}
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

use serde_json::{Value, json};

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), String> {
    let mut merged_array = Vec::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            return Err(format!("Input file not found: {}", input_path));
        }

        let file = File::open(path)
            .map_err(|e| format!("Failed to open {}: {}", input_path, e))?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read {}: {}", input_path, e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid JSON in {}: {}", input_path, e))?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(obj) => {
                merged_array.push(Value::Object(obj));
            }
            _ => {
                return Err(format!("JSON in {} must be an array or object", input_path));
            }
        }
    }

    let output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    serde_json::to_writer_pretty(output_file, &json!(merged_array))
        .map_err(|e| format!("Failed to write merged JSON: {}", e))?;

    Ok(())
}

pub fn merge_json_with_deduplication(
    input_paths: &[&str],
    output_path: &str,
    dedup_key: &str
) -> Result<(), String> {
    let mut seen_keys = HashMap::new();
    let mut deduplicated_array = Vec::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            return Err(format!("Input file not found: {}", input_path));
        }

        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", input_path, e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid JSON in {}: {}", input_path, e))?;

        let items = match json_value {
            Value::Array(arr) => arr,
            _ => return Err(format!("JSON in {} must be an array", input_path)),
        };

        for item in items {
            if let Some(obj) = item.as_object() {
                if let Some(key_value) = obj.get(dedup_key) {
                    let key_str = key_value.to_string();
                    if !seen_keys.contains_key(&key_str) {
                        seen_keys.insert(key_str.clone(), true);
                        deduplicated_array.push(item.clone());
                    }
                }
            }
        }
    }

    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    write!(output_file, "{}", serde_json::to_string_pretty(&deduplicated_array)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?)
        .map_err(|e| format!("Failed to write output: {}", e))?;

    Ok(())
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

        let result = merge_json_files(
            &[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()],
            output_file.path().to_str().unwrap()
        );

        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_merge_with_deduplication() {
        let file1_content = r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#;
        let file2_content = r#"[{"id": 1, "name": "Alice"}, {"id": 3, "name": "Charlie"}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let result = merge_json_with_deduplication(
            &[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()],
            output_file.path().to_str().unwrap(),
            "id"
        );

        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}