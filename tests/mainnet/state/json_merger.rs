
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::Path;

use serde_json::{Value, Map};

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> io::Result<()> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(obj) => {
                merged_array.push(Value::Object(obj));
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "JSON file must contain either an array or an object",
                ));
            }
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &merged_array)?;

    Ok(())
}

pub fn merge_json_with_key_deduplication<P: AsRef<Path>>(
    paths: &[P],
    output_path: P,
    key_field: &str,
) -> io::Result<()> {
    let mut unique_items: HashMap<String, Value> = HashMap::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        let items = match json_value {
            Value::Array(arr) => arr,
            Value::Object(_) => vec![json_value],
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "JSON file must contain either an array or an object",
                ));
            }
        };

        for item in items {
            if let Value::Object(map) = item {
                if let Some(key_value) = map.get(key_field) {
                    if let Some(key_str) = key_value.as_str() {
                        unique_items.insert(key_str.to_string(), Value::Object(map));
                    }
                }
            }
        }
    }

    let deduplicated_array: Vec<Value> = unique_items.into_values().collect();
    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &deduplicated_array)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_arrays() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(&file2, r#"[{"id": 3}, {"id": 4}]"#).unwrap();

        let paths = [file1.path(), file2.path()];
        merge_json_files(&paths, output_file.path()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_merge_with_deduplication() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": "a", "value": 1}, {"id": "b", "value": 2}]"#).unwrap();
        fs::write(&file2, r#"[{"id": "a", "value": 3}, {"id": "c", "value": 4}]"#).unwrap();

        let paths = [file1.path(), file2.path()];
        merge_json_with_key_deduplication(&paths, output_file.path(), "id").unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);

        let ids: Vec<&str> = array
            .iter()
            .filter_map(|item| item.get("id").and_then(|v| v.as_str()))
            .collect();
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"));
        assert!(ids.contains(&"c"));
    }
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
            return Err("Top-level JSON must be an object".into());
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

        let data1 = json!({"name": "Alice", "age": 30});
        let data2 = json!({"city": "Berlin", "age": 31});

        write!(file1, "{}", data1).unwrap();
        write!(file2, "{}", data2).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 31,
            "city": "Berlin"
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_file_not_found() {
        let result = merge_json_files(&["nonexistent.json"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_json_structure() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "[1, 2, 3]").unwrap();

        let paths = [file.path().to_str().unwrap()];
        let result = merge_json_files(&paths);
        assert!(result.is_err());
    }
}use serde_json::{Map, Value};

pub fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (Value::Object(a_map), Value::Object(b_map)) => {
            for (key, b_value) in b_map {
                if let Some(a_value) = a_map.get_mut(key) {
                    merge_json(a_value, b_value);
                } else {
                    a_map.insert(key.clone(), b_value.clone());
                }
            }
        }
        (a, b) => *a = b.clone(),
    }
}

pub fn merge_json_vec(values: Vec<Value>) -> Option<Value> {
    let mut result = Map::new();
    for value in values {
        if let Value::Object(map) = value {
            for (key, val) in map {
                if let Some(existing) = result.get_mut(&key) {
                    merge_json(existing, &val);
                } else {
                    result.insert(key, val);
                }
            }
        }
    }
    Some(Value::Object(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let mut a = json!({"a": 1, "b": {"c": 2}});
        let b = json!({"b": {"d": 3}, "e": 4});
        
        merge_json(&mut a, &b);
        
        assert_eq!(a, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_array_merge() {
        let values = vec![
            json!({"a": 1, "b": {"c": 2}}),
            json!({"b": {"d": 3}, "e": 4}),
            json!({"f": 5}),
        ];
        
        let result = merge_json_vec(values).unwrap();
        assert_eq!(result, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4, "f": 5}));
    }
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(serde_json::Value::Object(merged_map))
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

        writeln!(file1, r#"{"a": 1, "b": "test"}"#).unwrap();
        writeln!(file2, r#"{"c": true, "d": [1,2,3]}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("a").unwrap().as_i64(), Some(1));
        assert_eq!(obj.get("b").unwrap().as_str(), Some("test"));
        assert_eq!(obj.get("c").unwrap().as_bool(), Some(true));
        assert!(obj.get("d").unwrap().is_array());
    }
}
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: JsonValue = serde_json::from_str(&contents)?;
        
        match json_value {
            JsonValue::Array(arr) => {
                merged_array.extend(arr);
            }
            _ => {
                merged_array.push(json_value);
            }
        }
    }

    Ok(JsonValue::Array(merged_array))
}

pub fn merge_json_with_key(
    file_paths: &[impl AsRef<Path>],
    key: &str,
) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path in file_paths {
        let contents = fs::read_to_string(path.as_ref())?;
        let json_value: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(map) = json_value {
            if let Some(value) = map.get(key) {
                let path_str = path.as_ref().to_string_lossy().to_string();
                merged_map.insert(path_str, value.clone());
            }
        }
    }

    Ok(JsonValue::Object(serde_json::Map::from_iter(merged_map)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_array_json() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(&file2, r#"[{"id": 3}, {"id": 4}]"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_merge_with_key() {
        let file = NamedTempFile::new().unwrap();
        fs::write(&file, r#"{"user": "alice", "age": 30}"#).unwrap();

        let result = merge_json_with_key(&[file.path()], "user").unwrap();
        assert!(result.is_object());
        assert!(result.get(file.path().to_string_lossy().as_ref()).is_some());
    }
}