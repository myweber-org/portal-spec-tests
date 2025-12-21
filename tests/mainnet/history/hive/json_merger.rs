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
}