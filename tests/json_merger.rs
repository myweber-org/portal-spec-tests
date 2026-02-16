
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let json1 = json!({
            "name": "test",
            "value": 42
        });

        let json2 = json!({
            "enabled": true,
            "tags": ["rust", "json"]
        });

        write!(file1, "{}", json1.to_string()).unwrap();
        write!(file2, "{}", json2.to_string()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "test",
            "value": 42,
            "enabled": true,
            "tags": ["rust", "json"]
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_overwrite() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let json1 = json!({"key": "first"});
        let json2 = json!({"key": "second"});

        write!(file1, "{}", json1.to_string()).unwrap();
        write!(file2, "{}", json2.to_string()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        assert_eq!(result["key"], "second");
    }
}use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let data1 = json!({
            "name": "Alice",
            "age": 30
        });

        let data2 = json!({
            "city": "London",
            "active": true
        });

        write!(file1, "{}", data1.to_string()).unwrap();
        write!(file2, "{}", data2.to_string()).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 30,
            "city": "London",
            "active": true
        });

        assert_eq!(result, expected);
    }
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashMap::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_data: JsonValue = serde_json::from_reader(reader)?;

        if let JsonValue::Array(arr) = json_data {
            for item in arr {
                if let Some(key) = item.get("id").and_then(|v| v.as_str()) {
                    if !seen_keys.contains_key(key) {
                        seen_keys.insert(key.to_string(), true);
                        merged_array.push(item);
                    }
                } else {
                    merged_array.push(item);
                }
            }
        } else {
            merged_array.push(json_data);
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &merged_array)?;
    Ok(())
}

pub fn merge_json_directories<P: AsRef<Path>>(dir_path: P, output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut json_paths = Vec::new();
    
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            json_paths.push(path);
        }
    }

    if json_paths.is_empty() {
        return Err("No JSON files found in directory".into());
    }

    merge_json_files(&json_paths, output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_merge_json_files() {
        let dir = tempdir().unwrap();
        let file1_path = dir.path().join("data1.json");
        let file2_path = dir.path().join("data2.json");
        let output_path = dir.path().join("merged.json");

        let data1 = serde_json::json!([{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]);
        let data2 = serde_json::json!([{"id": "2", "name": "Robert"}, {"id": "3", "name": "Charlie"}]);

        fs::write(&file1_path, serde_json::to_string_pretty(&data1).unwrap()).unwrap();
        fs::write(&file2_path, serde_json::to_string_pretty(&data2).unwrap()).unwrap();

        merge_json_files(&[file1_path, file2_path], &output_path).unwrap();

        let output_content = fs::read_to_string(output_path).unwrap();
        let parsed: JsonValue = serde_json::from_str(&output_content).unwrap();
        
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}