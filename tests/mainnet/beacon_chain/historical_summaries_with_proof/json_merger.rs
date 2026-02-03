use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), String> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).map_err(|e| e.to_string())?;

        let json_value: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;

        match json_value {
            serde_json::Value::Array(arr) => {
                merged_array.extend(arr);
            }
            serde_json::Value::Object(obj) => {
                merged_array.push(serde_json::Value::Object(obj));
            }
            _ => {
                return Err(format!("Unsupported JSON structure in file: {}", file_path));
            }
        }
    }

    let output_json = serde_json::Value::Array(merged_array);
    let json_string = serde_json::to_string_pretty(&output_json).map_err(|e| e.to_string())?;

    fs::write(output_path, json_string).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn merge_json_with_deduplication(
    file_paths: &[&str],
    output_path: &str,
    unique_key: &str,
) -> Result<(), String> {
    let mut unique_map: HashMap<String, serde_json::Value> = HashMap::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).map_err(|e| e.to_string())?;

        let json_value: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;

        match json_value {
            serde_json::Value::Array(arr) => {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        if let Some(key_value) = obj.get(unique_key) {
                            if let Some(key_str) = key_value.as_str() {
                                unique_map.insert(key_str.to_string(), item);
                            }
                        }
                    }
                }
            }
            serde_json::Value::Object(obj) => {
                if let Some(key_value) = obj.get(unique_key) {
                    if let Some(key_str) = key_value.as_str() {
                        unique_map.insert(key_str.to_string(), serde_json::Value::Object(obj));
                    }
                }
            }
            _ => {
                return Err(format!("Unsupported JSON structure in file: {}", file_path));
            }
        }
    }

    let deduplicated_array: Vec<serde_json::Value> = unique_map.into_values().collect();
    let output_json = serde_json::Value::Array(deduplicated_array);
    let json_string = serde_json::to_string_pretty(&output_json).map_err(|e| e.to_string())?;

    fs::write(output_path, json_string).map_err(|e| e.to_string())?;

    Ok(())
}