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

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(serde_json::Value::Object(merged_map))
}use serde_json::{Value, Map};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, update: &Value, deep: bool) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if deep {
                    if let Some(base_value) = base_map.get_mut(key) {
                        merge_json(base_value, update_value, deep);
                    } else {
                        base_map.insert(key.clone(), update_value.clone());
                    }
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (base, update) => *base = update.clone(),
    }
}

pub fn merge_json_with_conflict_resolution(
    base: &mut Value,
    update: &Value,
    strategy: MergeStrategy,
) -> Result<(), MergeError> {
    match strategy {
        MergeStrategy::Overwrite => {
            merge_json(base, update, false);
            Ok(())
        }
        MergeStrategy::DeepMerge => {
            merge_json(base, update, true);
            Ok(())
        }
        MergeStrategy::PreferBase => Ok(()),
        MergeStrategy::PreferUpdate => {
            *base = update.clone();
            Ok(())
        }
        MergeStrategy::UnionArrays => merge_union_arrays(base, update),
    }
}

fn merge_union_arrays(base: &mut Value, update: &Value) -> Result<(), MergeError> {
    match (base, update) {
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            let mut seen = HashSet::new();
            for item in base_arr.iter() {
                if let Ok(serialized) = serde_json::to_string(item) {
                    seen.insert(serialized);
                }
            }
            
            for item in update_arr {
                if let Ok(serialized) = serde_json::to_string(item) {
                    if !seen.contains(&serialized) {
                        base_arr.push(item.clone());
                        seen.insert(serialized);
                    }
                }
            }
            Ok(())
        }
        _ => Err(MergeError::TypeMismatch),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Overwrite,
    DeepMerge,
    PreferBase,
    PreferUpdate,
    UnionArrays,
}

#[derive(Debug)]
pub enum MergeError {
    TypeMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shallow_merge() {
        let mut base = json!({"a": 1, "b": 2});
        let update = json!({"b": 3, "c": 4});
        
        merge_json(&mut base, &update, false);
        
        assert_eq!(base, json!({"a": 1, "b": 3, "c": 4}));
    }

    #[test]
    fn test_deep_merge() {
        let mut base = json!({"a": {"x": 1}, "b": 2});
        let update = json!({"a": {"y": 3}, "c": 4});
        
        merge_json(&mut base, &update, true);
        
        assert_eq!(base, json!({"a": {"x": 1, "y": 3}, "b": 2, "c": 4}));
    }

    #[test]
    fn test_union_arrays() {
        let mut base = json!([1, 2, 3]);
        let update = json!([3, 4, 5]);
        
        let result = merge_json_with_conflict_resolution(
            &mut base,
            &update,
            MergeStrategy::UnionArrays,
        );
        
        assert!(result.is_ok());
        let result_vec: Vec<i32> = base.as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_i64().map(|n| n as i32))
            .collect();
        assert!(result_vec.contains(&1));
        assert!(result_vec.contains(&2));
        assert!(result_vec.contains(&3));
        assert!(result_vec.contains(&4));
        assert!(result_vec.contains(&5));
        assert_eq!(result_vec.len(), 5);
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }
    
    Ok(Value::Object(merged))
}

fn merge_objects(base: &mut Map<String, Value>, new: Map<String, Value>) {
    for (key, new_value) in new {
        match base.get_mut(&key) {
            Some(existing_value) => {
                if let (Value::Object(mut existing_obj), Value::Object(new_obj)) = (existing_value, new_value) {
                    if let Value::Object(ref mut obj) = existing_value {
                        merge_objects(obj, new_obj);
                    }
                } else {
                    base.insert(key, new_value);
                }
            }
            None => {
                base.insert(key, new_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_basic() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#).unwrap();
        fs::write(&file2, r#"{"b": {"y": 20}, "c": 3}"#).unwrap();
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": {"x": 10, "y": 20},
            "c": 3
        });
        
        assert_eq!(result, expected);
    }
}