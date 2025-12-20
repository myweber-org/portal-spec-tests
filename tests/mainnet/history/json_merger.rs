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