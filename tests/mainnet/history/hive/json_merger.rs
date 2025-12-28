
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(serde_json::Value::Object(
        merged_map.into_iter().collect()
    ))
}
use serde_json::{Value, Map};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use std::env;

fn merge_json_files(file_paths: &[String]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_data: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_data {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(Value::Object(merged_map))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file> <input_file1> [input_file2 ...]", args[0]);
        std::process::exit(1);
    }

    let output_path = &args[1];
    let input_files = &args[2..];

    let merged = merge_json_files(input_files)?;

    let mut output_file = File::create(output_path)?;
    let json_string = serde_json::to_string_pretty(&merged)?;
    output_file.write_all(json_string.as_bytes())?;

    println!("Successfully merged {} JSON files into {}", input_files.len(), output_path);
    Ok(())
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), String> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: JsonValue = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let JsonValue::Array(arr) = json_value {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_value);
        }
    }

    let output_json = JsonValue::Array(merged_array);
    let output_str = serde_json::to_string_pretty(&output_json)
        .map_err(|e| format!("Failed to serialize merged JSON: {}", e))?;

    fs::write(output_path, output_str).map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}

pub fn merge_json_with_deduplication<P: AsRef<Path>>(paths: &[P], output_path: P, key: &str) -> Result<(), String> {
    let mut unique_map: HashMap<String, JsonValue> = HashMap::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: JsonValue = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        match json_value {
            JsonValue::Array(arr) => {
                for item in arr {
                    if let Some(id) = item.get(key).and_then(|v| v.as_str()) {
                        unique_map.insert(id.to_string(), item);
                    }
                }
            }
            JsonValue::Object(obj) => {
                if let Some(id) = obj.get(key).and_then(|v| v.as_str()) {
                    unique_map.insert(id.to_string(), JsonValue::Object(obj));
                }
            }
            _ => return Err("JSON root must be array or object".to_string()),
        }
    }

    let unique_values: Vec<JsonValue> = unique_map.into_values().collect();
    let output_json = JsonValue::Array(unique_values);
    let output_str = serde_json::to_string_pretty(&output_json)
        .map_err(|e| format!("Failed to serialize deduplicated JSON: {}", e))?;

    fs::write(output_path, output_str).map_err(|e| format!("Failed to write output file: {}", e))?;

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
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(&file2, r#"[{"id": 3}, {"id": 4}]"#).unwrap();

        merge_json_files(&[&file1, &file2], &output).unwrap();

        let content = fs::read_to_string(output).unwrap();
        let parsed: JsonValue = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_merge_with_deduplication() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": "a", "value": 1}, {"id": "b", "value": 2}]"#).unwrap();
        fs::write(&file2, r#"[{"id": "a", "value": 3}, {"id": "c", "value": 4}]"#).unwrap();

        merge_json_with_deduplication(&[&file1, &file2], &output, "id").unwrap();

        let content = fs::read_to_string(output).unwrap();
        let parsed: JsonValue = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}