
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use serde_json::{Value, Map};

pub fn merge_json_files(file_paths: &[String], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map: Map<String, Value> = Map::new();

    for file_path in file_paths {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let json_data: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_data {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain an object at the root level".into());
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(merged_map))?;

    Ok(())
}

pub fn merge_json_with_strategy(
    file_paths: &[String],
    output_path: &str,
    conflict_strategy: ConflictStrategy,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for file_path in file_paths {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let json_data: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_data {
            for (key, value) in map {
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        accumulator.insert(key, value);
                    }
                    ConflictStrategy::Skip => {
                        accumulator.entry(key).or_insert(value);
                    }
                    ConflictStrategy::MergeObjects => {
                        if let Some(existing) = accumulator.get_mut(&key) {
                            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, &value) {
                                let mut merged = existing_obj.clone();
                                for (k, v) in new_obj {
                                    merged.insert(k.clone(), v.clone());
                                }
                                *existing = Value::Object(merged);
                            } else {
                                accumulator.insert(key, value);
                            }
                        } else {
                            accumulator.insert(key, value);
                        }
                    }
                }
            }
        } else {
            return Err("Each JSON file must contain an object at the root level".into());
        }
    }

    let output_map: Map<String, Value> = accumulator.into_iter().collect();
    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(output_map))?;

    Ok(())
}

#[derive(Clone, Copy)]
pub enum ConflictStrategy {
    Overwrite,
    Skip,
    MergeObjects,
}
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

use serde_json::{json, Value};

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), String> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        merged_array.push(json_value);
    }

    let merged_json = json!(merged_array);
    let json_string = serde_json::to_string_pretty(&merged_json)
        .map_err(|e| format!("Failed to serialize merged JSON: {}", e))?;

    let mut output_file = File::create(&output_path)
        .map_err(|e| format!("Failed to create output file {}: {}", output_path.as_ref().display(), e))?;

    output_file.write_all(json_string.as_bytes())
        .map_err(|e| format!("Failed to write to output file {}: {}", output_path.as_ref().display(), e))?;

    Ok(())
}

pub fn merge_json_directories<P: AsRef<Path>>(dir_path: P, output_path: P, extension: &str) -> Result<(), String> {
    let mut json_files = Vec::new();
    
    for entry in fs::read_dir(dir_path).map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == extension) {
            json_files.push(path);
        }
    }

    if json_files.is_empty() {
        return Err("No JSON files found in directory".to_string());
    }

    merge_json_files(&json_files, output_path)
}

pub fn deduplicate_json_array_by_key<P: AsRef<Path>>(input_path: P, output_path: P, key: &str) -> Result<(), String> {
    let file = File::open(&input_path).map_err(|e| format!("Failed to open {}: {}", input_path.as_ref().display(), e))?;
    let reader = BufReader::new(file);
    let json_array: Vec<Value> = serde_json::from_reader(reader)
        .map_err(|e| format!("Invalid JSON array in {}: {}", input_path.as_ref().display(), e))?;

    let mut seen = HashMap::new();
    let mut deduplicated = Vec::new();

    for item in json_array {
        if let Some(obj) = item.as_object() {
            if let Some(key_value) = obj.get(key) {
                if let Some(key_str) = key_value.as_str() {
                    if !seen.contains_key(key_str) {
                        seen.insert(key_str.to_string(), true);
                        deduplicated.push(item);
                    }
                    continue;
                }
            }
        }
        deduplicated.push(item);
    }

    let output_json = json!(deduplicated);
    let json_string = serde_json::to_string_pretty(&output_json)
        .map_err(|e| format!("Failed to serialize deduplicated JSON: {}", e))?;

    let mut output_file = File::create(&output_path)
        .map_err(|e| format!("Failed to create output file {}: {}", output_path.as_ref().display(), e))?;

    output_file.write_all(json_string.as_bytes())
        .map_err(|e| format!("Failed to write to output file {}: {}", output_path.as_ref().display(), e))?;

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

        fs::write(&file1, r#"{"id": 1, "name": "Alice"}"#).unwrap();
        fs::write(&file2, r#"{"id": 2, "name": "Bob"}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()], output.path());
        assert!(result.is_ok());

        let content = fs::read_to_string(output.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_deduplicate_json_array() {
        let input = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        let json_content = r#"[{"id": "1", "value": "a"}, {"id": "1", "value": "b"}, {"id": "2", "value": "c"}]"#;
        fs::write(&input, json_content).unwrap();

        let result = deduplicate_json_array_by_key(input.path(), output.path(), "id");
        assert!(result.is_ok());

        let content = fs::read_to_string(output.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 2);
    }
}