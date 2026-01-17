
use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, update: &Value, strategy: MergeStrategy) -> Result<(), String> {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            merge_objects(base_map, update_map, strategy)?;
        }
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            merge_arrays(base_arr, update_arr, strategy)?;
        }
        (base_val, update_val) if base_val.is_null() => {
            *base = update_val.clone();
        }
        (base_val, update_val) if base_val != update_val => {
            return Err(format!("Type mismatch or value conflict: {:?} vs {:?}", base_val, update_val));
        }
        _ => {}
    }
    Ok(())
}

fn merge_objects(base: &mut Map<String, Value>, update: &Map<String, Value>, strategy: MergeStrategy) -> Result<(), String> {
    for (key, update_val) in update {
        match base.get_mut(key) {
            Some(base_val) => {
                merge_json(base_val, update_val, strategy)?;
            }
            None => {
                base.insert(key.clone(), update_val.clone());
            }
        }
    }
    Ok(())
}

fn merge_arrays(base: &mut Vec<Value>, update: &Vec<Value>, strategy: MergeStrategy) -> Result<(), String> {
    match strategy {
        MergeStrategy::Replace => {
            base.clear();
            base.extend(update.iter().cloned());
        }
        MergeStrategy::Append => {
            base.extend(update.iter().cloned());
        }
        MergeStrategy::MergeUnique => {
            let mut seen = HashMap::new();
            for item in base.iter() {
                let key = format!("{:?}", item);
                seen.insert(key, true);
            }
            for item in update {
                let key = format!("{:?}", item);
                if !seen.contains_key(&key) {
                    base.push(item.clone());
                    seen.insert(key, true);
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Replace,
    Append,
    MergeUnique,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let update = json!({"b": {"d": 3}, "e": 4});
        merge_json(&mut base, &update, MergeStrategy::Replace).unwrap();
        assert_eq!(base, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_merge_arrays_replace() {
        let mut base = json!([1, 2, 3]);
        let update = json!([4, 5]);
        merge_json(&mut base, &update, MergeStrategy::Replace).unwrap();
        assert_eq!(base, json!([4, 5]));
    }

    #[test]
    fn test_merge_arrays_append() {
        let mut base = json!([1, 2]);
        let update = json!([3, 4]);
        merge_json(&mut base, &update, MergeStrategy::Append).unwrap();
        assert_eq!(base, json!([1, 2, 3, 4]));
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

        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap().as_u64().unwrap(), 30);
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "Berlin");
        assert_eq!(obj.get("active").unwrap().as_bool().unwrap(), true);
        assert_eq!(obj.len(), 4);
    }

    #[test]
    fn test_duplicate_key_overwrites() {
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

        assert_eq!(obj.get("id").unwrap().as_u64().unwrap(), 200);
        assert_eq!(obj.get("tag").unwrap().as_str().unwrap(), "new");
    }
}use serde_json::{Map, Value};
use std::env;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file> <input_file1> [input_file2 ...]", args[0]);
        std::process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let mut merged_map = Map::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File '{}' not found, skipping.", input_path);
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
            eprintln!("Warning: '{}' does not contain a JSON object, skipping.", input_path);
        }
    }

    let merged_value = Value::Object(merged_map);
    let mut output_file = File::create(output_path)?;
    write!(output_file, "{}", serde_json::to_string_pretty(&merged_value)?)?;

    println!("Successfully merged {} file(s) into '{}'.", input_paths.len(), output_path);
    Ok(())
}use serde_json::{Map, Value};
use std::collections::HashMap;
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
            return Err("Top-level JSON is not an object".into());
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
            "name": "test",
            "count": 42
        });

        let data2 = json!({
            "enabled": true,
            "tags": ["a", "b"]
        });

        write!(file1, "{}", data1.to_string()).unwrap();
        write!(file2, "{}", data2.to_string()).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        let expected = json!({
            "name": "test",
            "count": 42,
            "enabled": true,
            "tags": ["a", "b"]
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_overwrite() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let data1 = json!({"key": "first"});
        let data2 = json!({"key": "second"});

        write!(file1, "{}", data1.to_string()).unwrap();
        write!(file2, "{}", data2.to_string()).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["key"], "second");
    }
}