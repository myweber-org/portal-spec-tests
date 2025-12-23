use std::collections::HashMap;
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

        if let serde_json::Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        }
    }

    let merged_value = serde_json::Value::Object(
        merged_map
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect()
    );

    Ok(merged_value)
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
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged = Map::new();
    let mut processed_keys = HashSet::new();

    for path in paths {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if processed_keys.contains(&key) {
                    return Err(format!("Duplicate key '{}' found in multiple files", key));
                }
                merged.insert(key.clone(), value);
                processed_keys.insert(key);
            }
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }

    Ok(Value::Object(merged))
}

pub fn merge_json_with_strategy<P: AsRef<Path>>(
    paths: &[P],
    conflict_strategy: ConflictStrategy
) -> Result<Value, String> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        merged.insert(key, value);
                    }
                    ConflictStrategy::Skip => {
                        merged.entry(key).or_insert(value);
                    }
                    ConflictStrategy::MergeObjects => {
                        if let Some(Value::Object(existing)) = merged.get(&key) {
                            if let Value::Object(new_obj) = &value {
                                let mut combined = existing.clone();
                                for (k, v) in new_obj {
                                    combined.insert(k.clone(), v.clone());
                                }
                                merged.insert(key, Value::Object(combined));
                            } else {
                                return Err(format!("Cannot merge non-object value for key '{}'", key));
                            }
                        } else {
                            merged.insert(key, value);
                        }
                    }
                }
            }
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }

    Ok(Value::Object(merged))
}

#[derive(Debug, Clone, Copy)]
pub enum ConflictStrategy {
    Overwrite,
    Skip,
    MergeObjects,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_json(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_merge_no_conflict() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"c": 3, "d": 4}"#);
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let obj = result.as_object().unwrap();
        
        assert_eq!(obj.len(), 4);
        assert_eq!(obj.get("a").unwrap().as_i64().unwrap(), 1);
        assert_eq!(obj.get("d").unwrap().as_i64().unwrap(), 4);
    }

    #[test]
    fn test_merge_with_conflict() {
        let file1 = create_temp_json(r#"{"a": 1}"#);
        let file2 = create_temp_json(r#"{"a": 2}"#);
        
        let result = merge_json_files(&[file1.path(), file2.path()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate key"));
    }

    #[test]
    fn test_merge_with_strategy() {
        let file1 = create_temp_json(r#"{"a": 1, "b": {"x": 10}}"#);
        let file2 = create_temp_json(r#"{"a": 2, "b": {"y": 20}}"#);
        
        let result = merge_json_with_strategy(
            &[file1.path(), file2.path()],
            ConflictStrategy::MergeObjects
        ).unwrap();
        
        let obj = result.as_object().unwrap();
        let b_obj = obj.get("b").unwrap().as_object().unwrap();
        
        assert_eq!(b_obj.get("x").unwrap().as_i64().unwrap(), 10);
        assert_eq!(b_obj.get("y").unwrap().as_i64().unwrap(), 20);
    }
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
                        if let Some(existing) = accumulator.get(&key) {
                            if existing.is_object() && value.is_object() {
                                if let (Some(existing_obj), Some(new_obj)) =
                                    (existing.as_object(), value.as_object())
                                {
                                    let mut merged = existing_obj.clone();
                                    for (k, v) in new_obj {
                                        merged.insert(k.clone(), v.clone());
                                    }
                                    accumulator.insert(key, Value::Object(merged));
                                } else {
                                    accumulator.insert(key, value);
                                }
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

    let mut map = Map::new();
    for (key, value) in accumulator {
        map.insert(key, value);
    }
    Ok(Value::Object(map))
}

#[derive(Debug, Clone, Copy)]
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

    #[test]
    fn test_basic_merge() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        file1.write_all(b"{\"a\": 1, \"b\": 2}").unwrap();
        file2.write_all(b"{\"c\": 3, \"d\": 4}").unwrap();

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
    fn test_conflict_overwrite() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        file1.write_all(b"{\"key\": \"first\"}").unwrap();
        file2.write_all(b"{\"key\": \"second\"}").unwrap();

        let result = merge_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            ConflictStrategy::Overwrite,
        )
        .unwrap();

        assert_eq!(result["key"], "second");
    }

    #[test]
    fn test_object_merge_strategy() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        file1
            .write_all(b"{\"config\": {\"timeout\": 30, \"retries\": 3}}")
            .unwrap();
        file2
            .write_all(b"{\"config\": {\"timeout\": 60, \"debug\": true}}")
            .unwrap();

        let result = merge_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            ConflictStrategy::MergeObjects,
        )
        .unwrap();

        let config = &result["config"];
        assert_eq!(config["timeout"], 60);
        assert_eq!(config["retries"], 3);
        assert_eq!(config["debug"], true);
    }
}