
use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, update: &Value, resolve_conflict: fn(&Value, &Value) -> Value) -> Value {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            let mut result = Map::new();
            let mut all_keys: Vec<String> = base_map.keys().chain(update_map.keys())
                .map(|k| k.to_string())
                .collect();
            all_keys.sort();
            all_keys.dedup();

            for key in all_keys {
                let base_val = base_map.get(&key);
                let update_val = update_map.get(&key);

                match (base_val, update_val) {
                    (Some(b), Some(u)) => {
                        let mut b_clone = b.clone();
                        let merged = merge_json(&mut b_clone, u, resolve_conflict);
                        result.insert(key, merged);
                    }
                    (Some(b), None) => {
                        result.insert(key, b.clone());
                    }
                    (None, Some(u)) => {
                        result.insert(key, u.clone());
                    }
                    (None, None) => unreachable!(),
                }
            }
            Value::Object(result)
        }
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            let mut merged = base_arr.clone();
            merged.extend_from_slice(update_arr);
            Value::Array(merged)
        }
        (base_val, update_val) => {
            if base_val == update_val {
                base_val.clone()
            } else {
                resolve_conflict(base_val, update_val)
            }
        }
    }
}

pub fn default_resolver(left: &Value, right: &Value) -> Value {
    right.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects() {
        let mut base = json!({
            "name": "Alice",
            "age": 30,
            "skills": ["Rust", "Python"]
        });
        let update = json!({
            "age": 31,
            "city": "Berlin",
            "skills": ["JavaScript"]
        });

        let merged = merge_json(&mut base, &update, default_resolver);
        assert_eq!(merged["age"], 31);
        assert_eq!(merged["city"], "Berlin");
        assert_eq!(merged["skills"], json!(["Rust", "Python", "JavaScript"]));
    }

    #[test]
    fn test_custom_resolver() {
        let mut base = json!({"priority": "low"});
        let update = json!({"priority": "high"});

        let keep_left = |left: &Value, _right: &Value| left.clone();
        let merged = merge_json(&mut base, &update, keep_left);
        assert_eq!(merged["priority"], "low");
    }
}
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(serde_json::Value::Object(
        merged_map.into_iter().collect()
    ))
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

        writeln!(file1, r#"{"name": "test", "value": 42}"#).unwrap();
        writeln!(file2, r#"{"extra": true, "nested": {"key": "value"}}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let result_obj = result.as_object().unwrap();

        assert_eq!(result_obj.get("name").unwrap().as_str().unwrap(), "test");
        assert_eq!(result_obj.get("value").unwrap().as_i64().unwrap(), 42);
        assert_eq!(result_obj.get("extra").unwrap().as_bool().unwrap(), true);
        assert!(result_obj.get("nested").is_some());
    }
}
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let json: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = merged.get(&key).unwrap();
                    if existing != &value {
                        return Err(format!("Conflict detected for key '{}'", key));
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        } else {
            return Err("Root JSON element must be an object".to_string());
        }
    }

    Ok(Value::Object(merged))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_successful_merge() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"a": 1, "b": "test"}"#).unwrap();
        writeln!(file2, r#"{"c": true, "d": [1,2,3]}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.len(), 4);
        assert_eq!(obj.get("a").unwrap(), &Value::from(1));
        assert_eq!(obj.get("b").unwrap(), &Value::from("test"));
        assert_eq!(obj.get("c").unwrap(), &Value::from(true));
    }

    #[test]
    fn test_conflict_detection() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"key": "value1"}"#).unwrap();
        writeln!(file2, r#"{"key": "value2"}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Conflict"));
    }
}