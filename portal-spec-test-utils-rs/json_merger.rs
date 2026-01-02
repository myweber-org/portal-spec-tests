
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "London", "active": true}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
        assert_eq!(result["city"], "London");
        assert_eq!(result["active"], true);
    }

    #[test]
    fn test_merge_with_missing_file() {
        let mut file1 = NamedTempFile::new().unwrap();
        writeln!(file1, r#"{"data": "test"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            "non_existent_file.json",
        ]).unwrap();

        assert_eq!(result["data"], "test");
        assert!(result.get("non_existent").is_none());
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merge_value(&mut merged_map, key, value);
            }
        }
    }

    let merged_json = Value::Object(merged_map);
    let serialized = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, serialized)?;

    Ok(())
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get(&key) {
        Some(existing_value) => {
            if existing_value.is_object() && new_value.is_object() {
                let mut existing_obj = existing_value.as_object().unwrap().clone();
                let new_obj = new_value.as_object().unwrap();

                for (nested_key, nested_value) in new_obj {
                    merge_value(&mut existing_obj, nested_key.clone(), nested_value.clone());
                }

                map.insert(key, Value::Object(existing_obj));
            } else if existing_value.is_array() && new_value.is_array() {
                let mut existing_arr = existing_value.as_array().unwrap().clone();
                let new_arr = new_value.as_array().unwrap();
                existing_arr.extend(new_arr.clone());
                map.insert(key, Value::Array(existing_arr));
            } else {
                map.insert(key, new_value);
            }
        }
        None => {
            map.insert(key, new_value);
        }
    }
}