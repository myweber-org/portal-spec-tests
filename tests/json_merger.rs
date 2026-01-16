use serde_json::{Map, Value};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, update: &Value, deep: bool) -> Value {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            if deep {
                for (key, update_value) in update_map {
                    if let Some(base_value) = base_map.get_mut(key) {
                        *base_value = merge_json(base_value, update_value, deep);
                    } else {
                        base_map.insert(key.clone(), update_value.clone());
                    }
                }
            } else {
                base_map.extend(update_map.clone());
            }
            Value::Object(base_map.clone())
        }
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            let mut seen = HashSet::new();
            let mut merged = base_arr.clone();
            
            for item in update_arr {
                let item_str = item.to_string();
                if !seen.contains(&item_str) {
                    seen.insert(item_str);
                    merged.push(item.clone());
                }
            }
            Value::Array(merged)
        }
        _ => update.clone(),
    }
}

pub fn merge_json_with_strategy(
    base: &Value,
    update: &Value,
    strategy: MergeStrategy,
) -> Result<Value, String> {
    let mut result = base.clone();
    
    match strategy {
        MergeStrategy::Shallow => Ok(merge_json(&mut result, update, false)),
        MergeStrategy::Deep => Ok(merge_json(&mut result, update, true)),
        MergeStrategy::PreferUpdate => Ok(update.clone()),
        MergeStrategy::PreferBase => Ok(base.clone()),
        MergeStrategy::Custom(merge_fn) => merge_fn(base, update),
    }
}

#[derive(Clone)]
pub enum MergeStrategy {
    Shallow,
    Deep,
    PreferUpdate,
    PreferBase,
    Custom(fn(&Value, &Value) -> Result<Value, String>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shallow_merge() {
        let mut base = json!({"a": 1, "b": {"x": 10}});
        let update = json!({"b": {"y": 20}, "c": 3});
        
        let result = merge_json(&mut base, &update, false);
        assert_eq!(result["b"], json!({"y": 20}));
    }

    #[test]
    fn test_deep_merge() {
        let mut base = json!({"a": 1, "b": {"x": 10}});
        let update = json!({"b": {"y": 20}, "c": 3});
        
        let result = merge_json(&mut base, &update, true);
        assert_eq!(result["b"], json!({"x": 10, "y": 20}));
    }
}