use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], deduplicate_by_key: Option<&str>) -> Result<Value, String> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashSet::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| e.to_string())?;

        let json_value: Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(key) = deduplicate_by_key {
                        if let Some(obj) = item.as_object() {
                            if let Some(key_value) = obj.get(key) {
                                let key_str = key_value.to_string();
                                if !seen_keys.contains(&key_str) {
                                    seen_keys.insert(key_str);
                                    merged_array.push(item);
                                }
                                continue;
                            }
                        }
                    }
                    merged_array.push(item);
                }
            }
            Value::Object(_) => {
                if let Some(key) = deduplicate_by_key {
                    if let Some(obj) = json_value.as_object() {
                        if let Some(key_value) = obj.get(key) {
                            let key_str = key_value.to_string();
                            if !seen_keys.contains(&key_str) {
                                seen_keys.insert(key_str);
                                merged_array.push(json_value);
                            }
                        } else {
                            merged_array.push(json_value);
                        }
                    }
                } else {
                    merged_array.push(json_value);
                }
            }
            _ => return Err("JSON root must be an array or object".to_string()),
        }
    }

    Ok(Value::Array(merged_array))
}

pub fn write_merged_json(output_path: &str, value: &Value) -> Result<(), String> {
    let file = File::create(output_path).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(file, value).map_err(|e| e.to_string())?;
    Ok(())
}