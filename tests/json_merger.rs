
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_object = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_object.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a top-level object".into());
        }
    }

    let merged_json = Value::Object(merged_object);
    let json_string = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, json_string)?;

    Ok(())
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged_map = Map::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| format!("Failed to read file: {}", e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                if merged_map.contains_key(&key) {
                    return Err(format!("Duplicate key '{}' found in JSON files", key));
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Top-level JSON value must be an object".to_string());
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_directories<P: AsRef<Path>>(dir_paths: &[P]) -> Result<HashMap<String, Value>, String> {
    let mut result_map = HashMap::new();

    for dir_path in dir_paths {
        let entries = fs::read_dir(dir_path).map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let file_name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| "Invalid file name".to_string())?
                    .to_string();

                let file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
                let mut reader = BufReader::new(file);
                let mut contents = String::new();
                reader.read_to_string(&mut contents).map_err(|e| format!("Failed to read file: {}", e))?;

                let json_value: Value = serde_json::from_str(&contents)
                    .map_err(|e| format!("Failed to parse JSON: {}", e))?;

                result_map.insert(file_name, json_value);
            }
        }
    }

    Ok(result_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "London", "country": "UK"}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap().as_u64().unwrap(), 30);
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "London");
        assert_eq!(obj.get("country").unwrap().as_str().unwrap(), "UK");
    }

    #[test]
    fn test_merge_json_files_duplicate_key() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice"}"#).unwrap();
        writeln!(file2, r#"{"name": "Bob"}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate key"));
    }
}