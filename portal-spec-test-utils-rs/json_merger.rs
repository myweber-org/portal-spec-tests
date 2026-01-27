
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
                base_arr.clear();
                base_arr.extend(ext_arr.iter().cloned());
            } else {
                let existing_set: HashSet<_> = base_arr.iter().collect();
                for item in ext_arr {
                    if !existing_set.contains(item) {
                        base_arr.push(item.clone());
                    }
                }
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

pub fn merge_with_strategy(
    base: &mut Value,
    extension: &Value,
    strategy: MergeStrategy,
) -> Result<(), String> {
    match strategy {
        MergeStrategy::Overwrite => merge_json(base, extension, true),
        MergeStrategy::Recursive => merge_json(base, extension, false),
        MergeStrategy::PreferBase => Ok(()),
        MergeStrategy::PreferExtension => {
            *base = extension.clone();
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Overwrite,
    Recursive,
    PreferBase,
    PreferExtension,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects() {
        let mut base = json!({"a": 1, "b": {"x": 10}});
        let ext = json!({"b": {"y": 20}, "c": 3});
        
        merge_json(&mut base, &ext, false).unwrap();
        assert_eq!(base["b"]["x"], 10);
        assert_eq!(base["b"]["y"], 20);
        assert_eq!(base["c"], 3);
    }

    #[test]
    fn test_merge_arrays() {
        let mut base = json!([1, 2, 3]);
        let ext = json!([3, 4, 5]);
        
        merge_json(&mut base, &ext, false).unwrap();
        assert_eq!(base.as_array().unwrap().len(), 5);
    }
}