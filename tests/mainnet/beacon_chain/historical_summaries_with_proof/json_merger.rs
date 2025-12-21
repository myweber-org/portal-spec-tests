
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