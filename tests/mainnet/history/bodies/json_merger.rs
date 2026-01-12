
use serde_json::{Value, Map};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, overlay: &Value, strategy: MergeStrategy) -> Result<(), String> {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, overlay_val) in overlay_map {
                if base_map.contains_key(key) {
                    let base_val = base_map.get_mut(key).unwrap();
                    match strategy {
                        MergeStrategy::Overwrite => *base_val = overlay_val.clone(),
                        MergeStrategy::Recursive => merge_json(base_val, overlay_val, strategy)?,
                        MergeStrategy::CombineArrays => {
                            if let (Value::Array(base_arr), Value::Array(overlay_arr)) = (base_val, overlay_val) {
                                let mut combined = base_arr.clone();
                                combined.extend(overlay_arr.clone());
                                *base_val = Value::Array(combined);
                            } else {
                                merge_json(base_val, overlay_val, MergeStrategy::Recursive)?;
                            }
                        }
                        MergeStrategy::UniqueArrays => {
                            if let (Value::Array(base_arr), Value::Array(overlay_arr)) = (base_val, overlay_val) {
                                let mut set: HashSet<Value> = base_arr.iter().cloned().collect();
                                set.extend(overlay_arr.iter().cloned());
                                *base_val = Value::Array(set.into_iter().collect());
                            } else {
                                merge_json(base_val, overlay_val, MergeStrategy::Recursive)?;
                            }
                        }
                    }
                } else {
                    base_map.insert(key.clone(), overlay_val.clone());
                }
            }
            Ok(())
        }
        _ => Err("Both values must be JSON objects".to_string()),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Overwrite,
    Recursive,
    CombineArrays,
    UniqueArrays,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_recursive_merge() {
        let mut base = json!({
            "a": 1,
            "b": {
                "c": 2,
                "d": 3
            }
        });
        
        let overlay = json!({
            "b": {
                "d": 99,
                "e": 100
            },
            "f": 4
        });
        
        merge_json(&mut base, &overlay, MergeStrategy::Recursive).unwrap();
        
        assert_eq!(base["a"], 1);
        assert_eq!(base["b"]["c"], 2);
        assert_eq!(base["b"]["d"], 99);
        assert_eq!(base["b"]["e"], 100);
        assert_eq!(base["f"], 4);
    }
    
    #[test]
    fn test_array_combination() {
        let mut base = json!({
            "items": [1, 2, 3]
        });
        
        let overlay = json!({
            "items": [3, 4, 5]
        });
        
        merge_json(&mut base, &overlay, MergeStrategy::CombineArrays).unwrap();
        assert_eq!(base["items"], json!([1, 2, 3, 3, 4, 5]));
        
        merge_json(&mut base, &overlay, MergeStrategy::UniqueArrays).unwrap();
        let mut result = base["items"].as_array().unwrap().clone();
        result.sort();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }
}