use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: serde_json::Value = serde_json::from_str(&contents)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(serde_json::Value::Object(merged_map.into_iter().collect()))
}use serde_json::{Value, Map};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, extension: &Value, overwrite_arrays: bool) {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(extension_map)) => {
            for (key, ext_value) in extension_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, ext_value, overwrite_arrays);
                } else {
                    base_map.insert(key.clone(), ext_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(extension_arr)) => {
            if overwrite_arrays {
                *base_arr = extension_arr.clone();
            } else {
                let mut seen = HashSet::new();
                for item in base_arr.iter() {
                    if let Value::Object(map) = item {
                        if let Some(Value::String(id)) = map.get("id") {
                            seen.insert(id.clone());
                        }
                    }
                }
                
                for item in extension_arr {
                    if let Value::Object(map) = item {
                        if let Some(Value::String(id)) = map.get("id") {
                            if !seen.contains(id) {
                                base_arr.push(item.clone());
                                seen.insert(id.clone());
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
            if !extension.is_null() {
                *base = extension.clone();
            }
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
                for (k, v) in ext_map {
                    merged.insert(k.clone(), v.clone());
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
            "a": {
                "b": 1,
                "c": [1, 2]
            }
        });
        
        let extension = json!({
            "a": {
                "c": [3],
                "d": 2
            },
            "e": 3
        });
        
        merge_json(&mut base, &extension, false);
        
        assert_eq!(base["a"]["b"], 1);
        assert_eq!(base["a"]["d"], 2);
        assert_eq!(base["e"], 3);
        assert_eq!(base["a"]["c"], json!([1, 2, 3]));
    }
}