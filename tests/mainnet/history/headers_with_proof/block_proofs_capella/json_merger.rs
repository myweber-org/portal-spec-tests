
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashSet::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                        if !seen_ids.insert(id.to_string()) {
                            eprintln!("Duplicate ID '{}' found in {}, skipping.", id, file_path);
                            continue;
                        }
                    }
                    merged_array.push(item);
                }
            }
            Value::Object(obj) => {
                if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                    if !seen_ids.insert(id.to_string()) {
                        eprintln!("Duplicate ID '{}' found in {}, skipping.", id, file_path);
                        continue;
                    }
                }
                merged_array.push(json!(obj));
            }
            _ => {
                eprintln!("Warning: {} does not contain a JSON array or object, skipping.", file_path);
            }
        }
    }

    let output_json = json!(merged_array);
    fs::write(output_path, serde_json::to_string_pretty(&output_json)?)?;

    println!("Successfully merged {} items into {}", merged_array.len(), output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"[{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]"#).unwrap();
        fs::write(file2.path(), r#"[{"id": "3", "name": "Charlie"}]"#).unwrap();

        let paths = [file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        let result = merge_json_files(&paths, output_file.path().to_str().unwrap());

        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use serde_json::{Map, Value};

fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: {} does not contain a JSON object, skipping.", input_path);
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(merged_map))?;

    Ok(())
}

fn main() {
    let inputs = vec!["config1.json", "config2.json", "config3.json"];
    let output = "merged_config.json";

    match merge_json_files(&inputs, output) {
        Ok(()) => println!("Successfully merged JSON files into {}", output),
        Err(e) => eprintln!("Error merging JSON files: {}", e),
    }
}
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

        if let JsonValue::Object(obj) = json_data {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let merged_value = JsonValue::Object(
        merged_map
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect()
    );

    Ok(merged_value)
}

pub fn write_merged_json(output_path: &str, json_value: &JsonValue) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(json_value)?;
    fs::write(output_path, json_string)?;
    Ok(())
}