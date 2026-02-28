use serde_json::{Value, Map};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, extension: &Value, overwrite_arrays: bool) {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(ext_map)) => {
            for (key, ext_value) in ext_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, ext_value, overwrite_arrays);
                } else {
                    base_map.insert(key.clone(), ext_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(ext_arr)) => {
            if overwrite_arrays {
                *base_arr = ext_arr.clone();
            } else {
                let mut seen = HashSet::new();
                for item in base_arr.iter() {
                    if let Value::Object(obj) = item {
                        if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                            seen.insert(id.to_string());
                        }
                    }
                }
                
                for item in ext_arr {
                    if let Value::Object(obj) = item {
                        if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                            if !seen.contains(id) {
                                base_arr.push(item.clone());
                                seen.insert(id.to_string());
                            }
                        } else {
                            base_arr.push(item.clone());
                        }
                    } else {
                        base_arr.push(item.clone());
                    }
                }
            }
        }
        (base, extension) => {
            *base = extension.clone();
        }
    }
}

pub fn merge_json_with_strategy(
    base: &Value,
    extension: &Value,
    strategy: MergeStrategy,
) -> Value {
    let mut result = base.clone();
    match strategy {
        MergeStrategy::Deep => merge_json(&mut result, extension, false),
        MergeStrategy::Shallow => {
            if let (Value::Object(base_map), Value::Object(ext_map)) = (&result, extension) {
                let mut merged = base_map.clone();
                for (key, value) in ext_map {
                    merged.insert(key.clone(), value.clone());
                }
                result = Value::Object(merged);
            }
        }
        MergeStrategy::OverwriteArrays => merge_json(&mut result, extension, true),
    }
    result
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Deep,
    Shallow,
    OverwriteArrays,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deep_merge() {
        let mut base = json!({
            "name": "Base",
            "nested": {
                "value": 1,
                "extra": "base"
            }
        });
        
        let extension = json!({
            "nested": {
                "value": 2,
                "new": "extension"
            },
            "additional": "field"
        });
        
        merge_json(&mut base, &extension, false);
        
        assert_eq!(base["name"], "Base");
        assert_eq!(base["nested"]["value"], 2);
        assert_eq!(base["nested"]["extra"], "base");
        assert_eq!(base["nested"]["new"], "extension");
        assert_eq!(base["additional"], "field");
    }
}