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
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let merged_value = Value::Object(merged_map);
    let json_string = serde_json::to_string_pretty(&merged_value)?;
    fs::write(output_path, json_string)?;

    Ok(())
}
use serde_json::{Map, Value};
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
                    eprintln!("Warning: Duplicate key '{}' found in {}", key, path_str);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err(format!("File {} does not contain a JSON object", path_str).into());
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    conflict_strategy: ConflictStrategy,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match accumulator.get(&key) {
                    Some(existing) => {
                        let resolved = conflict_strategy.resolve(existing, &value, &key, path_str);
                        accumulator.insert(key, resolved);
                    }
                    None => {
                        accumulator.insert(key, value);
                    }
                }
            }
        } else {
            return Err(format!("File {} does not contain a JSON object", path_str).into());
        }
    }

    let final_map: Map<String, Value> = accumulator.into_iter().collect();
    Ok(Value::Object(final_map))
}

pub enum ConflictStrategy {
    KeepFirst,
    KeepLast,
    MergeObjects,
    PrependArray,
}

impl ConflictStrategy {
    fn resolve(&self, existing: &Value, new: &Value, key: &str, file_path: &str) -> Value {
        match self {
            ConflictStrategy::KeepFirst => existing.clone(),
            ConflictStrategy::KeepLast => new.clone(),
            ConflictStrategy::MergeObjects => {
                if let (Value::Object(old_map), Value::Object(new_map)) = (existing, new) {
                    let mut merged = old_map.clone();
                    for (k, v) in new_map {
                        merged.insert(k.clone(), v.clone());
                    }
                    Value::Object(merged)
                } else {
                    eprintln!("Conflict on key '{}' from {}: types differ, keeping first", key, file_path);
                    existing.clone()
                }
            }
            ConflictStrategy::PrependArray => {
                let mut array = Vec::new();
                array.push(existing.clone());
                array.push(new.clone());
                Value::Array(array)
            }
        }
    }
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
    fn test_merge_basic() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"c": 3, "d": 4}"#);

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        let expected = json!({
            "a": 1,
            "b": 2,
            "c": 3,
            "d": 4
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_conflict_keep_last() {
        let file1 = create_temp_json(r#"{"key": "first"}"#);
        let file2 = create_temp_json(r#"{"key": "second"}"#);

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            ConflictStrategy::KeepLast,
        ).unwrap();

        assert_eq!(result["key"], "second");
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
            eprintln!("Warning: File {} not found, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            _ => {
                merged_array.push(json_value);
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
    fn test_merge_json_files() {
        let json1 = r#"["item1", "item2"]"#;
        let json2 = r#"{"key": "value"}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), json1).unwrap();
        fs::write(file2.path(), json2).unwrap();

        let inputs = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        merge_json_files(&inputs, output_file.path().to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert!(parsed.is_array());
        let arr = parsed.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], "item1");
        assert_eq!(arr[1], "item2");
        assert_eq!(arr[2]["key"], "value");
    }
}