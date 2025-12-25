
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        }
    }
    
    let output_value = Value::Object(merged);
    let output_json = serde_json::to_string_pretty(&output_value)?;
    fs::write(output_path, output_json)?;
    
    Ok(())
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        if let Some(existing_value) = target.get_mut(&key) {
            match (existing_value, source_value) {
                (Value::Object(ref mut target_obj), Value::Object(source_obj)) => {
                    merge_objects(target_obj, source_obj);
                }
                (Value::Array(ref mut target_arr), Value::Array(source_arr)) => {
                    target_arr.extend(source_arr);
                    target_arr.sort();
                    target_arr.dedup();
                }
                _ => {
                    *existing_value = source_value;
                }
            }
        } else {
            target.insert(key, source_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_objects() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        let json1 = json!({
            "name": "test",
            "values": [1, 2],
            "nested": {"a": 1}
        });

        let json2 = json!({
            "version": "1.0",
            "values": [2, 3],
            "nested": {"b": 2}
        });

        fs::write(&file1, serde_json::to_string(&json1).unwrap()).unwrap();
        fs::write(&file2, serde_json::to_string(&json2).unwrap()).unwrap();

        merge_json_files(&[file1.path(), file2.path()], output.path()).unwrap();

        let result_content = fs::read_to_string(output.path()).unwrap();
        let result: Value = serde_json::from_str(&result_content).unwrap();

        assert_eq!(result["name"], "test");
        assert_eq!(result["version"], "1.0");
        assert_eq!(result["values"], json!([1, 2, 3]));
        assert_eq!(result["nested"]["a"], 1);
        assert_eq!(result["nested"]["b"], 2);
    }
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_data = HashMap::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let data: HashMap<String, serde_json::Value> = serde_json::from_str(&content)?;
        
        for (key, value) in data {
            merged_data.insert(key, value);
        }
    }

    let output_json = serde_json::to_string_pretty(&merged_data)?;
    fs::write(output_path, output_json)?;

    Ok(())
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

        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path().to_str().unwrap();

        let input_paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap()
        ];

        merge_json_files(&input_paths, output_path).unwrap();

        let merged_content = fs::read_to_string(output_path).unwrap();
        let parsed: HashMap<String, serde_json::Value> = serde_json::from_str(&merged_content).unwrap();

        assert_eq!(parsed.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(parsed.get("age").unwrap().as_u64().unwrap(), 30);
        assert_eq!(parsed.get("city").unwrap().as_str().unwrap(), "London");
        assert_eq!(parsed.get("active").unwrap().as_bool().unwrap(), true);
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merge_value(&mut merged_map, key, value);
            }
        }
    }

    let merged_json = Value::Object(merged_map);
    let json_string = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, json_string)?;

    Ok(())
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get_mut(&key) {
        Some(existing_value) => {
            if existing_value.is_object() && new_value.is_object() {
                if let (Value::Object(existing_map), Value::Object(new_map)) = (existing_value, new_value) {
                    for (nested_key, nested_value) in new_map {
                        merge_value(existing_map, nested_key, nested_value);
                    }
                }
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
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"{"a": 1, "b": {"c": 2}}"#).unwrap();
        fs::write(file2.path(), r#"{"b": {"d": 3}, "e": 4}"#).unwrap();

        merge_json_files(
            &[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()],
            output.path().to_str().unwrap()
        ).unwrap();

        let result = fs::read_to_string(output.path()).unwrap();
        assert!(result.contains("\"a\": 1"));
        assert!(result.contains("\"c\": 2"));
        assert!(result.contains("\"d\": 3"));
        assert!(result.contains("\"e\": 4"));
    }
}