
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map: HashMap<String, Value> = HashMap::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", input_path);
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
            eprintln!("Warning: File {} does not contain a JSON object, skipping.", input_path);
        }
    }

    let mut sorted_map = Map::new();
    let mut keys: Vec<&String> = merged_map.keys().collect();
    keys.sort();

    for key in keys {
        sorted_map.insert(key.clone(), merged_map[key].clone());
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(sorted_map))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"{"name": "Alice", "age": 30}"#;
        let json2 = r#"{"city": "London", "country": "UK"}"#;
        let json3 = r#"{"name": "Bob", "active": true}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let file3 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        std::fs::write(file1.path(), json1).unwrap();
        std::fs::write(file2.path(), json2).unwrap();
        std::fs::write(file3.path(), json3).unwrap();

        let inputs = &[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
            file3.path().to_str().unwrap(),
        ];

        merge_json_files(inputs, output_file.path().to_str().unwrap()).unwrap();

        let mut content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["name"], "Bob");
        assert_eq!(parsed["age"], 30);
        assert_eq!(parsed["city"], "London");
        assert_eq!(parsed["country"], "UK");
        assert_eq!(parsed["active"], true);
    }

    #[test]
    fn test_merge_with_missing_file() {
        let json1 = r#"{"data": "test"}"#;
        let file1 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        std::fs::write(file1.path(), json1).unwrap();

        let inputs = &[
            file1.path().to_str().unwrap(),
            "non_existent_file.json",
        ];

        let result = merge_json_files(inputs, output_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let mut content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["data"], "test");
    }
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashMap::new();

    for path in paths {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let json_data: JsonValue = serde_json::from_str(&content)?;

        match json_data {
            JsonValue::Array(arr) => {
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
            }
            JsonValue::Object(obj) => {
                if let Some(key) = obj.get("id").and_then(|v| v.as_str()) {
                    if !seen_keys.contains_key(key) {
                        seen_keys.insert(key.to_string(), true);
                        merged_array.push(JsonValue::Object(obj));
                    }
                } else {
                    merged_array.push(JsonValue::Object(obj));
                }
            }
            _ => {
                merged_array.push(json_data);
            }
        }
    }

    let output_json = JsonValue::Array(merged_array);
    let serialized = serde_json::to_string_pretty(&output_json)?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(serialized.as_bytes())?;

    Ok(())
}

pub fn merge_json_directory<P: AsRef<Path>>(dir_path: P, output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut json_files = Vec::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            json_files.push(path);
        }
    }

    if json_files.is_empty() {
        return Err("No JSON files found in directory".into());
    }

    merge_json_files(&json_files, output_path)
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    let mut conflict_log = Vec::new();
    let mut processed_keys = HashSet::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if processed_keys.contains(&key) {
                    conflict_log.push(format!("Conflict detected for key '{}' in file {:?}", key, path.as_ref()));
                    continue;
                }
                merged.insert(key.clone(), value);
                processed_keys.insert(key);
            }
        }
    }

    let result = Value::Object(merged);
    let output = serde_json::to_string_pretty(&result)?;
    fs::write(output_path, output)?;

    if !conflict_log.is_empty() {
        eprintln!("Merged with conflicts:");
        for log in conflict_log {
            eprintln!("  {}", log);
        }
    }

    Ok(())
}
use serde_json::{Map, Value};
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
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_merge_json_files() {
        let dir = tempdir().unwrap();
        
        let file1_path = dir.path().join("a.json");
        let mut file1 = File::create(&file1_path).unwrap();
        writeln!(file1, r#"{"name": "test", "count": 42}"#).unwrap();

        let file2_path = dir.path().join("b.json");
        let mut file2 = File::create(&file2_path).unwrap();
        writeln!(file2, r#"{"active": true, "tags": ["rust", "json"]}"#).unwrap();

        let paths = vec![
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap(), "test");
        assert_eq!(obj.get("count").unwrap(), 42);
        assert_eq!(obj.get("active").unwrap(), true);
        assert!(obj.contains_key("tags"));
    }
}