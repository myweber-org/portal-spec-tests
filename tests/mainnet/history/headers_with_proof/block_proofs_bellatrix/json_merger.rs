use serde_json::{Value, json};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;
        merged_array.push(json_value);
    }

    let output_json = json!(merged_array);
    let output_str = serde_json::to_string_pretty(&output_json)?;

    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, output_str)?;

    Ok(())
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[&str]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let json_data: JsonValue = serde_json::from_str(&content)?;
        
        if let JsonValue::Array(arr) = json_data {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_data);
        }
    }

    Ok(JsonValue::Array(merged_array))
}

pub fn merge_json_with_key(file_paths: &[&str], key: &str) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let content = fs::read_to_string(path)?;
        let json_data: JsonValue = serde_json::from_str(&content)?;

        if let JsonValue::Object(obj) = json_data {
            if let Some(value) = obj.get(key) {
                merged_map.insert(path_str.to_string(), value.clone());
            }
        }
    }

    Ok(JsonValue::Object(serde_json::Map::from_iter(
        merged_map.into_iter()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(&file2, r#"{"id": 3}"#).unwrap();

        let paths = &[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        let result = merge_json_files(paths).unwrap();

        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_merge_json_with_key() {
        let file = NamedTempFile::new().unwrap();
        fs::write(&file, r#"{"user": "alice", "age": 30}"#).unwrap();

        let paths = &[file.path().to_str().unwrap()];
        let result = merge_json_with_key(paths, "user").unwrap();

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key(file.path().to_str().unwrap()));
    }
}