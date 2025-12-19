
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
}