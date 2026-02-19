
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashSet::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                        if !seen_ids.insert(id.to_string()) {
                            eprintln!("Duplicate ID '{}' found in {}, skipping.", id, file_path);
                            continue;
                        }
                    }
                    merged_array.push(item);
                }
            }
            Value::Object(obj) => {
                if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                    if !seen_ids.insert(id.to_string()) {
                        eprintln!("Duplicate ID '{}' found in {}, skipping.", id, file_path);
                        continue;
                    }
                }
                merged_array.push(json!(obj));
            }
            _ => {
                eprintln!("Warning: {} does not contain a JSON array or object, skipping.", file_path);
            }
        }
    }

    let output_json = json!(merged_array);
    fs::write(output_path, serde_json::to_string_pretty(&output_json)?)?;

    println!("Successfully merged {} items into {}", merged_array.len(), output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"[{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]"#).unwrap();
        fs::write(file2.path(), r#"[{"id": "3", "name": "Charlie"}]"#).unwrap();

        let paths = [file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        let result = merge_json_files(&paths, output_file.path().to_str().unwrap());

        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use serde_json::{Map, Value};

fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", input_path);
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
            eprintln!("Warning: {} does not contain a JSON object, skipping.", input_path);
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(merged_map))?;

    Ok(())
}

fn main() {
    let inputs = vec!["config1.json", "config2.json", "config3.json"];
    let output = "merged_config.json";

    match merge_json_files(&inputs, output) {
        Ok(()) => println!("Successfully merged JSON files into {}", output),
        Err(e) => eprintln!("Error merging JSON files: {}", e),
    }
}
use std::collections::HashMap;
use std::fs;
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[&str]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_data: JsonValue = serde_json::from_str(&content)?;

        if let JsonValue::Object(obj) = json_data {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let merged_value = JsonValue::Object(
        merged_map
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect()
    );

    Ok(merged_value)
}

pub fn write_merged_json(output_path: &str, json_value: &JsonValue) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(json_value)?;
    fs::write(output_path, json_string)?;
    Ok(())
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
        Some(existing) => {
            if let (Value::Object(ref mut existing_obj), Value::Object(new_obj)) = (existing, &new_value) {
                for (nested_key, nested_value) in new_obj {
                    merge_value(existing_obj, nested_key.clone(), nested_value.clone());
                }
            } else if existing != &new_value {
                *existing = Value::Array(vec![existing.clone(), new_value]);
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

    #[test]
    fn test_merge_json() {
        let json1 = json!({
            "name": "Alice",
            "settings": {
                "theme": "dark"
            }
        });

        let json2 = json!({
            "age": 30,
            "settings": {
                "font_size": 14
            }
        });

        let merged = merge_json(&json1, &json2);
        assert_eq!(merged["name"], "Alice");
        assert_eq!(merged["age"], 30);
        assert_eq!(merged["settings"]["theme"], "dark");
        assert_eq!(merged["settings"]["font_size"], 14);
    }

    fn merge_json(a: &Value, b: &Value) -> Value {
        let mut map = Map::new();
        
        if let Value::Object(obj) = a {
            for (key, value) in obj {
                map.insert(key.clone(), value.clone());
            }
        }

        if let Value::Object(obj) = b {
            for (key, value) in obj {
                merge_value(&mut map, key.clone(), value.clone());
            }
        }

        Value::Object(map)
    }
}
use serde_json::{Map, Value};
use std::collections::HashSet;

pub enum ConflictResolution {
    PreferFirst,
    PreferSecond,
    MergeArrays,
    FailOnConflict,
}

pub fn merge_json(
    first: &Map<String, Value>,
    second: &Map<String, Value>,
    resolution: ConflictResolution,
) -> Result<Map<String, Value>, String> {
    let mut result = Map::new();
    let mut all_keys: HashSet<&String> = first.keys().chain(second.keys()).collect();

    for key in all_keys {
        let first_val = first.get(key);
        let second_val = second.get(key);

        match (first_val, second_val) {
            (Some(f), None) => {
                result.insert(key.clone(), f.clone());
            }
            (None, Some(s)) => {
                result.insert(key.clone(), s.clone());
            }
            (Some(f), Some(s)) => {
                let merged = handle_conflict(key, f, s, &resolution)?;
                result.insert(key.clone(), merged);
            }
            _ => unreachable!(),
        }
    }

    Ok(result)
}

fn handle_conflict(
    key: &str,
    first: &Value,
    second: &Value,
    resolution: &ConflictResolution,
) -> Result<Value, String> {
    match resolution {
        ConflictResolution::PreferFirst => Ok(first.clone()),
        ConflictResolution::PreferSecond => Ok(second.clone()),
        ConflictResolution::MergeArrays => {
            if let (Value::Array(a1), Value::Array(a2)) = (first, second) {
                let mut merged = a1.clone();
                merged.extend(a2.clone());
                Ok(Value::Array(merged))
            } else {
                Err(format!(
                    "Cannot merge non-array values for key '{}' with MergeArrays strategy",
                    key
                ))
            }
        }
        ConflictResolution::FailOnConflict => Err(format!("Conflict detected for key '{}'", key)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_prefer_first() {
        let mut first = Map::new();
        first.insert("a".to_string(), json!(1));
        first.insert("b".to_string(), json!("test"));

        let mut second = Map::new();
        second.insert("b".to_string(), json!("overwritten"));
        second.insert("c".to_string(), json!(true));

        let merged = merge_json(&first, &second, ConflictResolution::PreferFirst).unwrap();
        
        assert_eq!(merged.get("a"), Some(&json!(1)));
        assert_eq!(merged.get("b"), Some(&json!("test")));
        assert_eq!(merged.get("c"), Some(&json!(true)));
    }

    #[test]
    fn test_merge_arrays() {
        let mut first = Map::new();
        first.insert("items".to_string(), json!([1, 2, 3]));

        let mut second = Map::new();
        second.insert("items".to_string(), json!([4, 5]));

        let merged = merge_json(&first, &second, ConflictResolution::MergeArrays).unwrap();
        
        if let Value::Array(arr) = &merged["items"] {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr, &vec![json!(1), json!(2), json!(3), json!(4), json!(5)]);
        } else {
            panic!("Expected array");
        }
    }
}