use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    return Err(format!("Duplicate key found: {}", key).into());
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("JSON root is not an object".into());
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

        writeln!(file1, r#"{"name": "test", "value": 42}"#).unwrap();
        writeln!(file2, r#"{"enabled": true, "tags": ["a", "b"]}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]);

        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged["name"], "test");
        assert_eq!(merged["value"], 42);
        assert_eq!(merged["enabled"], true);
        assert!(merged["tags"].is_array());
    }

    #[test]
    fn test_duplicate_key_error() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"key": "first"}"#).unwrap();
        writeln!(file2, r#"{"key": "second"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate key"));
    }
}