
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, String> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let mut file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path_str, e))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path_str, e))?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        } else {
            return Err(format!("File {} does not contain a JSON object", path_str));
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    conflict_strategy: ConflictStrategy,
) -> Result<Value, String> {
    let mut merged_map: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let mut file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path_str, e))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path_str, e))?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        merged_map.insert(key, value);
                    }
                    ConflictStrategy::Skip => {
                        merged_map.entry(key).or_insert(value);
                    }
                    ConflictStrategy::MergeObjects => {
                        if let Some(existing) = merged_map.get_mut(&key) {
                            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, &value) {
                                let mut combined = existing_obj.clone();
                                for (k, v) in new_obj {
                                    combined.insert(k.clone(), v.clone());
                                }
                                *existing = Value::Object(combined);
                            } else {
                                merged_map.insert(key, value);
                            }
                        } else {
                            merged_map.insert(key, value);
                        }
                    }
                }
            }
        } else {
            return Err(format!("File {} does not contain a JSON object", path_str));
        }
    }

    let final_map: Map<String, Value> = merged_map.into_iter().collect();
    Ok(Value::Object(final_map))
}

pub enum ConflictStrategy {
    Overwrite,
    Skip,
    MergeObjects,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;
    use tempfile::tempdir;

    #[test]
    fn test_merge_json_files() {
        let dir = tempdir().unwrap();
        
        let file1_path = dir.path().join("file1.json");
        write(&file1_path, r#"{"a": 1, "b": 2}"#).unwrap();
        
        let file2_path = dir.path().join("file2.json");
        write(&file2_path, r#"{"c": 3, "d": 4}"#).unwrap();

        let result = merge_json_files(&[
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], 2);
        assert_eq!(result["c"], 3);
        assert_eq!(result["d"], 4);
    }

    #[test]
    fn test_merge_with_conflict_overwrite() {
        let dir = tempdir().unwrap();
        
        let file1_path = dir.path().join("file1.json");
        write(&file1_path, r#"{"a": 1, "b": 2}"#).unwrap();
        
        let file2_path = dir.path().join("file2.json");
        write(&file2_path, r#"{"b": 99, "c": 3}"#).unwrap();

        let result = merge_json_with_strategy(
            &[
                file1_path.to_str().unwrap(),
                file2_path.to_str().unwrap(),
            ],
            ConflictStrategy::Overwrite,
        ).unwrap();

        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], 99);
        assert_eq!(result["c"], 3);
    }
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error>> {
    let mut merged = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged.insert(key, value);
            }
        }
    }

    Ok(merged)
}

pub fn write_merged_json(output_path: &str, data: &HashMap<String, serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(data)?;
    fs::write(output_path, json_string)?;
    Ok(())
}
use std::collections::HashMap;
use std::fs;
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged = HashMap::new();

    for path in file_paths {
        let content = fs::read_to_string(path)?;
        let json: JsonValue = serde_json::from_str(&content)?;

        if let JsonValue::Object(map) = json {
            for (key, value) in map {
                merged.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain an object".into());
        }
    }

    Ok(JsonValue::Object(merged.into_iter().collect()))
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

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("a").unwrap(), &JsonValue::from(1));
        assert_eq!(obj.get("b").unwrap(), &JsonValue::from("test"));
        assert_eq!(obj.get("c").unwrap(), &JsonValue::from(true));
        assert_eq!(obj.get("d").unwrap(), &JsonValue::from(vec![1, 2, 3]));
    }
}