use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashSet::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        if let Some(id_val) = obj.get("id") {
                            if let Some(id_str) = id_val.as_str() {
                                if !seen_ids.contains(id_str) {
                                    seen_ids.insert(id_str.to_string());
                                    merged_array.push(item);
                                }
                                continue;
                            }
                        }
                    }
                    merged_array.push(item);
                }
            }
            Value::Object(_) => merged_array.push(json_value),
            _ => return Err("Input JSON must be an object or array".into()),
        }
    }

    let output_json = json!(merged_array);
    fs::write(output_path, output_json.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_arrays() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": "a", "value": 1}, {"value": 2}]"#).unwrap();
        fs::write(&file2, r#"[{"id": "a", "value": 3}, {"id": "b", "value": 4}]"#).unwrap();

        merge_json_files(&[file1.path(), file2.path()], output.path()).unwrap();

        let content = fs::read_to_string(output.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Array(arr) = json_value {
            for item in arr {
                merged_array.push(item);
            }
        } else {
            merged_array.push(json_value);
        }
    }

    let output_file = File::create(output_path)?;
    let merged_json = json!(merged_array);
    serde_json::to_writer_pretty(output_file, &merged_json)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"{"id": 1, "name": "Alice"}"#;
        let json2 = r#"[{"id": 2, "name": "Bob"}, {"id": 3, "name": "Charlie"}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), json1).unwrap();
        fs::write(file2.path(), json2).unwrap();

        let inputs = &[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        merge_json_files(inputs, output_file.path().to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert!(parsed.is_array());
        let arr = parsed.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["name"], "Alice");
        assert_eq!(arr[1]["name"], "Bob");
        assert_eq!(arr[2]["name"], "Charlie");
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                merge_value(&mut merged, key, value);
            }
        }
    }

    Ok(Value::Object(merged))
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get_mut(&key) {
        Some(existing_value) => {
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing_value, &new_value) {
                let mut existing_obj = existing_obj.clone();
                for (nested_key, nested_value) in new_obj {
                    merge_value(&mut existing_obj, nested_key.clone(), nested_value.clone());
                }
                map.insert(key, Value::Object(existing_obj));
            } else if existing_value.is_array() && new_value.is_array() {
                if let (Value::Array(existing_arr), Value::Array(new_arr)) = (existing_value, new_value) {
                    existing_arr.extend(new_arr);
                }
            } else {
                map.insert(key, new_value);
            }
        }
        None => {
            map.insert(key, new_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_basic_objects() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "d": 4}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": 2,
            "c": 3,
            "d": 4
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_nested_conflicts() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"config": {"timeout": 30, "retries": 3}}"#).unwrap();
        fs::write(&file2, r#"{"config": {"timeout": 60, "debug": true}}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "config": {
                "timeout": 60,
                "retries": 3,
                "debug": true
            }
        });

        assert_eq!(result, expected);
    }
}use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
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

    Ok(Value::Object(merged_map))
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get_mut(&key) {
        Some(existing_value) => {
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing_value, &new_value) {
                let mut existing_obj = existing_obj.clone();
                for (nested_key, nested_value) in new_obj {
                    merge_value(&mut existing_obj, nested_key.clone(), nested_value.clone());
                }
                map.insert(key, Value::Object(existing_obj));
            } else if existing_value != &new_value {
                let conflict_array = match existing_value {
                    Value::Array(arr) => {
                        let mut arr = arr.clone();
                        arr.push(new_value.clone());
                        arr
                    }
                    _ => vec![existing_value.clone(), new_value.clone()],
                };
                map.insert(key, Value::Array(conflict_array));
            }
        }
        None => {
            map.insert(key, new_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "d": 4}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": 2,
            "c": 3,
            "d": 4
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_conflict_resolution() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#).unwrap();
        fs::write(&file2, r#"{"a": 2, "b": {"y": 20}}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": [1, 2],
            "b": {"x": 10, "y": 20}
        });

        assert_eq!(result, expected);
    }
}