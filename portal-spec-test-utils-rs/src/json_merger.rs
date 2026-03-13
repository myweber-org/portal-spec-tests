use serde_json::{Map, Value};

pub fn merge_json(a: &mut Value, b: Value) {
    match (a, b) {
        (Value::Object(a_map), Value::Object(b_map)) => {
            for (key, b_val) in b_map {
                if let Some(a_val) = a_map.get_mut(&key) {
                    merge_json(a_val, b_val);
                } else {
                    a_map.insert(key, b_val);
                }
            }
        }
        (a, b) => *a = b,
    }
}

pub fn merge_json_array(arrays: Vec<Value>) -> Value {
    let mut result = Map::new();
    
    for item in arrays {
        if let Value::Object(map) = item {
            for (key, value) in map {
                if let Some(existing) = result.get_mut(&key) {
                    merge_json(existing, value);
                } else {
                    result.insert(key, value);
                }
            }
        }
    }
    
    Value::Object(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let mut a = json!({"a": 1, "b": {"c": 2}});
        let b = json!({"b": {"d": 3}, "e": 4});
        
        merge_json(&mut a, b);
        
        assert_eq!(a, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_array_merge() {
        let arrays = vec![
            json!({"a": 1, "b": {"c": 2}}),
            json!({"b": {"d": 3}, "e": 4}),
            json!({"a": 5, "f": 6})
        ];
        
        let result = merge_json_array(arrays);
        
        assert_eq!(result, json!({"a": 5, "b": {"c": 2, "d": 3}, "e": 4, "f": 6}));
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
        Some(existing) => {
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, &new_value) {
                let mut existing_obj = existing_obj.clone();
                for (k, v) in new_obj {
                    merge_value(&mut existing_obj, k.clone(), v.clone());
                }
                map.insert(key, Value::Object(existing_obj));
            } else if let (Value::Array(existing_arr), Value::Array(new_arr)) = (existing, &new_value) {
                let mut combined = existing_arr.clone();
                combined.extend(new_arr.clone());
                map.insert(key, Value::Array(combined));
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
    fn test_merge_nested_objects() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"config": {"timeout": 30}}"#).unwrap();
        fs::write(&file2, r#"{"config": {"retries": 5}}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "config": {
                "timeout": 30,
                "retries": 5
            }
        });

        assert_eq!(result, expected);
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        let json: Value = serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }

    Ok(Value::Object(merged))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        match target.get_mut(&key) {
            Some(target_value) => {
                if let (Value::Object(mut target_obj), Value::Object(source_obj)) = (target_value.clone(), source_value) {
                    merge_objects(&mut target_obj, source_obj);
                    target.insert(key, Value::Object(target_obj));
                } else if target_value != &source_value {
                    eprintln!("Conflict for key '{}': {:?} vs {:?}", key, target_value, source_value);
                }
            }
            None => {
                target.insert(key, source_value);
            }
        }
    }
}