
use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, update: &Value, strategy: MergeStrategy) -> Result<(), String> {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            merge_objects(base_map, update_map, strategy)?;
        }
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            merge_arrays(base_arr, update_arr, strategy)?;
        }
        (base_val, update_val) if base_val.is_null() => {
            *base = update_val.clone();
        }
        (base_val, update_val) if base_val != update_val => {
            return Err(format!("Type mismatch or value conflict: {:?} vs {:?}", base_val, update_val));
        }
        _ => {}
    }
    Ok(())
}

fn merge_objects(base: &mut Map<String, Value>, update: &Map<String, Value>, strategy: MergeStrategy) -> Result<(), String> {
    for (key, update_val) in update {
        match base.get_mut(key) {
            Some(base_val) => {
                merge_json(base_val, update_val, strategy)?;
            }
            None => {
                base.insert(key.clone(), update_val.clone());
            }
        }
    }
    Ok(())
}

fn merge_arrays(base: &mut Vec<Value>, update: &Vec<Value>, strategy: MergeStrategy) -> Result<(), String> {
    match strategy {
        MergeStrategy::Replace => {
            base.clear();
            base.extend(update.iter().cloned());
        }
        MergeStrategy::Append => {
            base.extend(update.iter().cloned());
        }
        MergeStrategy::MergeUnique => {
            let mut seen = HashMap::new();
            for item in base.iter() {
                let key = format!("{:?}", item);
                seen.insert(key, true);
            }
            for item in update {
                let key = format!("{:?}", item);
                if !seen.contains_key(&key) {
                    base.push(item.clone());
                    seen.insert(key, true);
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Replace,
    Append,
    MergeUnique,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let update = json!({"b": {"d": 3}, "e": 4});
        merge_json(&mut base, &update, MergeStrategy::Replace).unwrap();
        assert_eq!(base, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_merge_arrays_replace() {
        let mut base = json!([1, 2, 3]);
        let update = json!([4, 5]);
        merge_json(&mut base, &update, MergeStrategy::Replace).unwrap();
        assert_eq!(base, json!([4, 5]));
    }

    #[test]
    fn test_merge_arrays_append() {
        let mut base = json!([1, 2]);
        let update = json!([3, 4]);
        merge_json(&mut base, &update, MergeStrategy::Append).unwrap();
        assert_eq!(base, json!([1, 2, 3, 4]));
    }
}