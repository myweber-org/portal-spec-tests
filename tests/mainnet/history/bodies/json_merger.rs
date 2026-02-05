use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;
type JsonObject = serde_json::Map<String, JsonValue>;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged = JsonObject::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(obj) = json_data {
            merge_objects(&mut merged, obj);
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(JsonValue::Object(merged))
}

fn merge_objects(target: &mut JsonObject, source: JsonObject) {
    for (key, value) in source {
        match (target.get_mut(&key), value) {
            (Some(JsonValue::Object(existing_obj)), JsonValue::Object(new_obj)) => {
                merge_objects(existing_obj.as_object_mut().unwrap(), new_obj);
            }
            (Some(JsonValue::Array(existing_arr)), JsonValue::Array(new_arr)) => {
                existing_arr.extend(new_arr);
            }
            _ => {
                target.insert(key, value);
            }
        }
    }
}

pub fn merge_json_strings(json_strings: &[&str]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged = JsonObject::new();

    for json_str in json_strings {
        let json_data: JsonValue = serde_json::from_str(json_str)?;

        if let JsonValue::Object(obj) = json_data {
            merge_objects(&mut merged, obj);
        } else {
            return Err("Each JSON string must represent a JSON object".into());
        }
    }

    Ok(JsonValue::Object(merged))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects() {
        let json1 = json!({
            "name": "Alice",
            "details": {
                "age": 30,
                "city": "London"
            }
        });

        let json2 = json!({
            "details": {
                "country": "UK",
                "age": 31
            },
            "active": true
        });

        let merged = merge_json_strings(&[
            &json1.to_string(),
            &json2.to_string()
        ]).unwrap();

        let expected = json!({
            "name": "Alice",
            "details": {
                "age": 31,
                "city": "London",
                "country": "UK"
            },
            "active": true
        });

        assert_eq!(merged, expected);
    }

    #[test]
    fn test_merge_arrays() {
        let json1 = json!({
            "tags": ["rust", "json"],
            "data": [1, 2]
        });

        let json2 = json!({
            "tags": ["merge"],
            "data": [3, 4]
        });

        let merged = merge_json_strings(&[
            &json1.to_string(),
            &json2.to_string()
        ]).unwrap();

        let expected = json!({
            "tags": ["rust", "json", "merge"],
            "data": [1, 2, 3, 4]
        });

        assert_eq!(merged, expected);
    }
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(JsonValue::Object(serde_json::Map::from_iter(merged_map)))
}

pub fn merge_and_write(
    input_paths: &[impl AsRef<Path>],
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let merged = merge_json_files(input_paths)?;
    let json_string = serde_json::to_string_pretty(&merged)?;
    fs::write(output_path, json_string)?;
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
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_with_strategy(
    file_paths: &[&str],
    conflict_strategy: ConflictStrategy,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        accumulator.insert(key, value);
                    }
                    ConflictStrategy::Skip => {
                        accumulator.entry(key).or_insert(value);
                    }
                    ConflictStrategy::MergeObjects => {
                        if let Some(existing) = accumulator.get_mut(&key) {
                            if let (Value::Object(existing_obj), Value::Object(new_obj)) =
                                (existing, &value)
                            {
                                let mut merged_obj = existing_obj.clone();
                                for (k, v) in new_obj {
                                    merged_obj.insert(k.clone(), v.clone());
                                }
                                accumulator.insert(key, Value::Object(merged_obj));
                            } else {
                                accumulator.insert(key, value);
                            }
                        } else {
                            accumulator.insert(key, value);
                        }
                    }
                }
            }
        }
    }

    let mut final_map = Map::new();
    for (key, value) in accumulator {
        final_map.insert(key, value);
    }

    Ok(Value::Object(final_map))
}

#[derive(Clone, Copy)]
pub enum ConflictStrategy {
    Overwrite,
    Skip,
    MergeObjects,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_json(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_basic_merge() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"c": 3, "d": 4}"#);

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .unwrap();

        let expected = json!({
            "a": 1,
            "b": 2,
            "c": 3,
            "d": 4
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_overwrite_strategy() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"a": 99, "c": 3}"#);

        let result = merge_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            ConflictStrategy::Overwrite,
        )
        .unwrap();

        let expected = json!({
            "a": 99,
            "b": 2,
            "c": 3
        });

        assert_eq!(result, expected);
    }
}use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufReader, Result};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Array(arr) = json_value {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_value);
        }
    }

    Ok(json!(merged_array))
}

pub fn merge_and_write<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> Result<()> {
    let merged = merge_json_files(input_paths)?;
    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &merged)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = json!([{"id": 1}, {"id": 2}]);
        let json2 = json!({"id": 3});

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        serde_json::to_writer(&file1, &json1).unwrap();
        serde_json::to_writer(&file2, &json2).unwrap();

        let paths = [file1.path(), file2.path()];
        let result = merge_json_files(&paths).unwrap();

        let expected = json!([{"id": 1}, {"id": 2}, {"id": 3}]);
        assert_eq!(result, expected);
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "London", "active": true}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap(), 30);
        assert_eq!(obj.get("city").unwrap(), "London");
        assert_eq!(obj.get("active").unwrap(), true);
    }
}use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(_) => {
                merged_array.push(json_value);
            }
            _ => {
                eprintln!("Warning: File {} does not contain a JSON object or array, skipping.", path_str);
            }
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
    fn test_merge_json_objects() {
        let json1 = r#"{"id": 1, "name": "Alice"}"#;
        let json2 = r#"{"id": 2, "name": "Bob"}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), json1).unwrap();
        fs::write(file2.path(), json2).unwrap();

        let inputs = &[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        merge_json_files(inputs, output_file.path().to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 2);
        assert_eq!(array[0]["name"], "Alice");
        assert_eq!(array[1]["name"], "Bob");
    }
}