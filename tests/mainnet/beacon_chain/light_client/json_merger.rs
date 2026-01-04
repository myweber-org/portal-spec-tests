use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], deduplicate: bool) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_hashes = HashSet::new();

    for path in paths {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;

        if let Value::Array(arr) = json_value {
            for item in arr {
                if deduplicate {
                    let item_hash = format!("{:?}", item);
                    if seen_hashes.insert(item_hash) {
                        merged_array.push(item);
                    }
                } else {
                    merged_array.push(item);
                }
            }
        } else {
            merged_array.push(json_value);
        }
    }

    Ok(Value::Array(merged_array))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output_path)?;
    serde_json::to_writer_pretty(file, value)?;
    Ok(())
}use serde_json::{Map, Value};
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
            return Err("Top-level JSON value is not an object".into());
        }
    }

    Ok(Value::Object(merged_map))
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
        writeln!(file2, r#"{"city": "Berlin", "active": true}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap(), 30);
        assert_eq!(obj.get("city").unwrap(), "Berlin");
        assert_eq!(obj.get("active").unwrap(), true);
    }

    #[test]
    fn test_overwrite_keys() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"id": 100, "tag": "old"}"#).unwrap();
        writeln!(file2, r#"{"id": 200, "tag": "new"}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("id").unwrap(), 200);
        assert_eq!(obj.get("tag").unwrap(), "new");
    }
}