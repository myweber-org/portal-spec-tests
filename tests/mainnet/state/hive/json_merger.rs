
use serde_json::{Value, Map};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, new: &Value, conflict_strategy: ConflictStrategy) -> Result<(), String> {
    match (base, new) {
        (Value::Object(base_map), Value::Object(new_map)) => {
            merge_objects(base_map, new_map, conflict_strategy)
        }
        (Value::Array(base_arr), Value::Array(new_arr)) => {
            merge_arrays(base_arr, new_arr, conflict_strategy)
        }
        (base_val, new_val) => {
            if base_val != new_val {
                match conflict_strategy {
                    ConflictStrategy::PreferNew => *base = new_val.clone(),
                    ConflictStrategy::PreferOld => (),
                    ConflictStrategy::Combine => return Err("Cannot combine non-object/non-array values".to_string()),
                }
            }
            Ok(())
        }
    }
}

fn merge_objects(base: &mut Map<String, Value>, new: &Map<String, Value>, strategy: ConflictStrategy) -> Result<(), String> {
    for (key, new_value) in new {
        if let Some(base_value) = base.get_mut(key) {
            merge_json(base_value, new_value, strategy)?;
        } else {
            base.insert(key.clone(), new_value.clone());
        }
    }
    Ok(())
}

fn merge_arrays(base: &mut Vec<Value>, new: &Vec<Value>, strategy: ConflictStrategy) -> Result<(), String> {
    match strategy {
        ConflictStrategy::PreferNew => {
            base.clear();
            base.extend(new.clone());
        }
        ConflictStrategy::PreferOld => (),
        ConflictStrategy::Combine => {
            let base_set: HashSet<_> = base.iter().collect();
            for item in new {
                if !base_set.contains(item) {
                    base.push(item.clone());
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConflictStrategy {
    PreferOld,
    PreferNew,
    Combine,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects_prefer_new() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let new = json!({"b": {"c": 3, "d": 4}, "e": 5});
        
        merge_json(&mut base, &new, ConflictStrategy::PreferNew).unwrap();
        
        assert_eq!(base["a"], 1);
        assert_eq!(base["b"]["c"], 3);
        assert_eq!(base["b"]["d"], 4);
        assert_eq!(base["e"], 5);
    }

    #[test]
    fn test_merge_arrays_combine() {
        let mut base = json!([1, 2, 3]);
        let new = json!([3, 4, 5]);
        
        merge_json(&mut base, &new, ConflictStrategy::Combine).unwrap();
        
        let result: Vec<_> = base.as_array().unwrap().iter().collect();
        assert!(result.contains(&&json!(1)));
        assert!(result.contains(&&json!(2)));
        assert!(result.contains(&&json!(3)));
        assert!(result.contains(&&json!(4)));
        assert!(result.contains(&&json!(5)));
    }
}
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object, skipping.", path_str);
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "London", "active": true}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap().as_u64().unwrap(), 30);
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "London");
        assert!(obj.get("active").unwrap().as_bool().unwrap());
    }
}