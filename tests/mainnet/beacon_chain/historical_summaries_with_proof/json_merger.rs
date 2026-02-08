use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), String> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).map_err(|e| e.to_string())?;

        let json_value: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;

        match json_value {
            serde_json::Value::Array(arr) => {
                merged_array.extend(arr);
            }
            serde_json::Value::Object(obj) => {
                merged_array.push(serde_json::Value::Object(obj));
            }
            _ => {
                return Err(format!("Unsupported JSON structure in file: {}", file_path));
            }
        }
    }

    let output_json = serde_json::Value::Array(merged_array);
    let json_string = serde_json::to_string_pretty(&output_json).map_err(|e| e.to_string())?;

    fs::write(output_path, json_string).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn merge_json_with_deduplication(
    file_paths: &[&str],
    output_path: &str,
    unique_key: &str,
) -> Result<(), String> {
    let mut unique_map: HashMap<String, serde_json::Value> = HashMap::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).map_err(|e| e.to_string())?;

        let json_value: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;

        match json_value {
            serde_json::Value::Array(arr) => {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        if let Some(key_value) = obj.get(unique_key) {
                            if let Some(key_str) = key_value.as_str() {
                                unique_map.insert(key_str.to_string(), item);
                            }
                        }
                    }
                }
            }
            serde_json::Value::Object(obj) => {
                if let Some(key_value) = obj.get(unique_key) {
                    if let Some(key_str) = key_value.as_str() {
                        unique_map.insert(key_str.to_string(), serde_json::Value::Object(obj));
                    }
                }
            }
            _ => {
                return Err(format!("Unsupported JSON structure in file: {}", file_path));
            }
        }
    }

    let deduplicated_array: Vec<serde_json::Value> = unique_map.into_values().collect();
    let output_json = serde_json::Value::Array(deduplicated_array);
    let json_string = serde_json::to_string_pretty(&output_json).map_err(|e| e.to_string())?;

    fs::write(output_path, json_string).map_err(|e| e.to_string())?;

    Ok(())
}
use std::collections::HashMap;
use serde_json::{Value, Map};

pub enum MergeStrategy {
    PreferFirst,
    PreferSecond,
    ConcatenateArrays,
    SumNumbers,
}

pub fn merge_json(a: &Value, b: &Value, strategy: &MergeStrategy) -> Value {
    match (a, b) {
        (Value::Object(map_a), Value::Object(map_b)) => {
            let mut result = Map::new();
            
            for (key, val_a) in map_a {
                result.insert(key.clone(), val_a.clone());
            }
            
            for (key, val_b) in map_b {
                if let Some(existing) = result.get_mut(key) {
                    *existing = merge_json(existing, val_b, strategy);
                } else {
                    result.insert(key.clone(), val_b.clone());
                }
            }
            
            Value::Object(result)
        }
        (Value::Array(arr_a), Value::Array(arr_b)) => {
            match strategy {
                MergeStrategy::ConcatenateArrays => {
                    let mut combined = arr_a.clone();
                    combined.extend(arr_b.clone());
                    Value::Array(combined)
                }
                _ => {
                    let merged = arr_a.iter().zip(arr_b.iter())
                        .map(|(a, b)| merge_json(a, b, strategy))
                        .collect();
                    Value::Array(merged)
                }
            }
        }
        (Value::Number(num_a), Value::Number(num_b)) => {
            match strategy {
                MergeStrategy::SumNumbers => {
                    if let (Some(f_a), Some(f_b)) = (num_a.as_f64(), num_b.as_f64()) {
                        Value::from(f_a + f_b)
                    } else {
                        Value::from(num_a.as_i64().unwrap_or(0) + num_b.as_i64().unwrap_or(0))
                    }
                }
                _ => b.clone(),
            }
        }
        _ => match strategy {
            MergeStrategy::PreferFirst => a.clone(),
            _ => b.clone(),
        },
    }
}

pub fn merge_json_with_defaults(base: &Value, defaults: &Value) -> Value {
    let mut result = defaults.clone();
    
    if let (Value::Object(base_map), Value::Object(default_map)) = (base, &result) {
        let mut result_map = default_map.clone();
        
        for (key, value) in base_map {
            if !result_map.contains_key(key) {
                result_map.insert(key.clone(), value.clone());
            } else if let Some(existing) = result_map.get_mut(key) {
                *existing = merge_json_with_defaults(value, existing);
            }
        }
        
        result = Value::Object(result_map);
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects() {
        let a = json!({"name": "Alice", "age": 30});
        let b = json!({"age": 31, "city": "London"});
        
        let merged = merge_json(&a, &b, &MergeStrategy::PreferSecond);
        
        assert_eq!(merged["name"], "Alice");
        assert_eq!(merged["age"], 31);
        assert_eq!(merged["city"], "London");
    }

    #[test]
    fn test_concat_arrays() {
        let a = json!([1, 2, 3]);
        let b = json!([4, 5, 6]);
        
        let merged = merge_json(&a, &b, &MergeStrategy::ConcatenateArrays);
        
        assert_eq!(merged, json!([1, 2, 3, 4, 5, 6]));
    }

    #[test]
    fn test_sum_numbers() {
        let a = json!({"count": 5});
        let b = json!({"count": 3});
        
        let merged = merge_json(&a, &b, &MergeStrategy::SumNumbers);
        
        assert_eq!(merged["count"], 8);
    }
}