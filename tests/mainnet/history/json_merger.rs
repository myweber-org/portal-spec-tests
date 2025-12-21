use serde_json::{Map, Value};
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
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            let update_set: HashSet<_> = update_arr.iter().collect();
            for item in update_arr {
                if !base_arr.contains(item) {
                    base_arr.push(item.clone());
                }
            }
        }
        (base, update) => *base = update.clone(),
    }
}

pub fn merge_json_with_strategy(
    base: &mut Value,
    update: &Value,
    strategy: MergeStrategy,
) -> Result<(), String> {
    match strategy {
        MergeStrategy::Shallow => {
            *base = update.clone();
            Ok(())
        }
        MergeStrategy::Deep => {
            merge_json(base, update, true);
            Ok(())
        }
        MergeStrategy::ArrayAppend => {
            if let (Value::Array(base_arr), Value::Array(update_arr)) = (base, update) {
                base_arr.extend(update_arr.iter().cloned());
                Ok(())
            } else {
                Err("Both values must be arrays for ArrayAppend strategy".to_string())
            }
        }
        MergeStrategy::ArrayUnique => {
            if let (Value::Array(base_arr), Value::Array(update_arr)) = (base, update) {
                let mut seen = HashSet::new();
                for item in base_arr.iter() {
                    seen.insert(item.to_string());
                }
                for item in update_arr {
                    if !seen.contains(&item.to_string()) {
                        base_arr.push(item.clone());
                        seen.insert(item.to_string());
                    }
                }
                Ok(())
            } else {
                Err("Both values must be arrays for ArrayUnique strategy".to_string())
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Shallow,
    Deep,
    ArrayAppend,
    ArrayUnique,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shallow_merge() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let update = json!({"b": {"d": 3}, "e": 4});
        
        merge_json_with_strategy(&mut base, &update, MergeStrategy::Shallow).unwrap();
        assert_eq!(base, json!({"b": {"d": 3}, "e": 4}));
    }

    #[test]
    fn test_deep_merge() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let update = json!({"b": {"d": 3}, "e": 4});
        
        merge_json_with_strategy(&mut base, &update, MergeStrategy::Deep).unwrap();
        assert_eq!(base, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_array_append() {
        let mut base = json!([1, 2, 3]);
        let update = json!([3, 4, 5]);
        
        merge_json_with_strategy(&mut base, &update, MergeStrategy::ArrayAppend).unwrap();
        assert_eq!(base, json!([1, 2, 3, 3, 4, 5]));
    }

    #[test]
    fn test_array_unique() {
        let mut base = json!([1, 2, 3]);
        let update = json!([3, 4, 5]);
        
        merge_json_with_strategy(&mut base, &update, MergeStrategy::ArrayUnique).unwrap();
        assert_eq!(base, json!([1, 2, 3, 4, 5]));
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
    let mut conflict_log = Vec::new();

    for (idx, path) in paths.iter().enumerate() {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if processed_keys.contains(&key) {
                    conflict_log.push(format!(
                        "Conflict: key '{}' already exists from file {}, overwritten by file {}",
                        key,
                        paths[idx-1].as_ref().display(),
                        path.as_ref().display()
                    ));
                }
                merged.insert(key.clone(), value);
                processed_keys.insert(key);
            }
        } else {
            return Err(format!("Top-level must be JSON object in {}", path.as_ref().display()));
        }
    }

    if !conflict_log.is_empty() {
        eprintln!("Merge conflicts detected:");
        for log in &conflict_log {
            eprintln!("  {}", log);
        }
    }

    Ok(Value::Object(merged))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, value: &Value) -> Result<(), String> {
    let json_str = serde_json::to_string_pretty(value)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    
    fs::write(output_path, json_str)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(())
}