
use serde_json::{Map, Value};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, extension: &Value, overwrite: bool) -> Result<(), String> {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(ext_map)) => {
            for (key, ext_value) in ext_map {
                if base_map.contains_key(key) {
                    let base_value = base_map.get_mut(key).unwrap();
                    if overwrite {
                        *base_value = ext_value.clone();
                    } else {
                        merge_json(base_value, ext_value, overwrite)?;
                    }
                } else {
                    base_map.insert(key.clone(), ext_value.clone());
                }
            }
            Ok(())
        }
        (Value::Array(base_arr), Value::Array(ext_arr)) => {
            if overwrite {
                *base_arr = ext_arr.clone();
            } else {
                base_arr.extend(ext_arr.iter().cloned());
            }
            Ok(())
        }
        (base_val, ext_val) => {
            if overwrite {
                *base_val = ext_val.clone();
                Ok(())
            } else {
                Err("Type mismatch and overwrite disabled".to_string())
            }
        }
    }
}

pub fn merge_json_with_conflict_list(
    base: &mut Value,
    extension: &Value,
) -> Result<HashSet<String>, String> {
    let mut conflicts = HashSet::new();

    if let (Value::Object(base_map), Value::Object(ext_map)) = (base, extension) {
        for (key, ext_value) in ext_map {
            if base_map.contains_key(key) {
                let base_value = base_map.get_mut(key).unwrap();
                if let (Value::Object(_), Value::Object(_)) = (base_value, ext_value) {
                    let sub_conflicts = merge_json_with_conflict_list(base_value, ext_value)?;
                    for sub_key in sub_conflicts {
                        conflicts.insert(format!("{}.{}", key, sub_key));
                    }
                } else if base_value != ext_value {
                    conflicts.insert(key.clone());
                } else {
                    merge_json(base_value, ext_value, false)?;
                }
            } else {
                base_map.insert(key.clone(), ext_value.clone());
            }
        }
    } else {
        return Err("Both values must be JSON objects".to_string());
    }

    Ok(conflicts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let extension = json!({"b": {"d": 3}, "e": 4});
        
        merge_json(&mut base, &extension, false).unwrap();
        assert_eq!(base, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_overwrite_array() {
        let mut base = json!([1, 2, 3]);
        let extension = json!([4, 5]);
        
        merge_json(&mut base, &extension, true).unwrap();
        assert_eq!(base, json!([4, 5]));
    }

    #[test]
    fn test_conflict_detection() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let extension = json!({"a": 99, "b": {"c": 100, "d": 3}});
        
        let conflicts = merge_json_with_conflict_list(&mut base, &extension).unwrap();
        assert_eq!(conflicts, HashSet::from(["a".to_string(), "b.c".to_string()]));
    }
}use std::collections::HashMap;
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

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(serde_json::Value::Object(merged_map))
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
        writeln!(file2, r#"{"city": "Berlin", "age": 35}"#).unwrap();

        let result = merge_json_files(&[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()]).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "Berlin");
        assert_eq!(obj.get("age").unwrap().as_u64().unwrap(), 35);
    }
}