
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
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(serde_json::Value::Object(merged_map))
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
        writeln!(file2, r#"{"city": "Berlin", "age": 35}"#).unwrap();

        let result = merge_json_files(&[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()]).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "Berlin");
        assert_eq!(obj.get("age").unwrap().as_u64().unwrap(), 35);
    }
}
use serde_json::{Map, Value};
use std::collections::HashSet;

pub enum ConflictResolution {
    PreferFirst,
    PreferSecond,
    MergeArrays,
    FailOnConflict,
}

pub fn merge_json_objects(
    first: &Map<String, Value>,
    second: &Map<String, Value>,
    resolution: ConflictResolution,
) -> Result<Map<String, Value>, String> {
    let mut result = Map::new();
    let mut all_keys: HashSet<&String> = first.keys().chain(second.keys()).collect();

    for key in all_keys {
        let first_val = first.get(key);
        let second_val = second.get(key);

        match (first_val, second_val) {
            (Some(f), None) => {
                result.insert(key.clone(), f.clone());
            }
            (None, Some(s)) => {
                result.insert(key.clone(), s.clone());
            }
            (Some(f), Some(s)) => {
                let merged = merge_values(f, s, &resolution)?;
                result.insert(key.clone(), merged);
            }
            _ => unreachable!(),
        }
    }

    Ok(result)
}

fn merge_values(
    first: &Value,
    second: &Value,
    resolution: &ConflictResolution,
) -> Result<Value, String> {
    match (first, second) {
        (Value::Object(f_obj), Value::Object(s_obj)) => {
            let merged_map = merge_json_objects(f_obj, s_obj, resolution.clone())?;
            Ok(Value::Object(merged_map))
        }
        (Value::Array(f_arr), Value::Array(s_arr)) => match resolution {
            ConflictResolution::MergeArrays => {
                let mut merged = f_arr.clone();
                merged.extend(s_arr.clone());
                Ok(Value::Array(merged))
            }
            _ => resolve_conflict(first, second, resolution),
        },
        _ => resolve_conflict(first, second, resolution),
    }
}

fn resolve_conflict(first: &Value, second: &Value, resolution: &ConflictResolution) -> Result<Value, String> {
    match resolution {
        ConflictResolution::PreferFirst => Ok(first.clone()),
        ConflictResolution::PreferSecond => Ok(second.clone()),
        ConflictResolution::FailOnConflict => Err(format!(
            "Conflict detected between values: {:?} and {:?}",
            first, second
        )),
        ConflictResolution::MergeArrays => Err("Cannot merge non-array values with MergeArrays strategy".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_with_prefer_first() {
        let first = json!({
            "name": "Alice",
            "age": 30,
            "tags": ["rust", "python"]
        })
        .as_object()
        .unwrap()
        .clone();

        let second = json!({
            "name": "Bob",
            "city": "Berlin",
            "tags": ["go"]
        })
        .as_object()
        .unwrap()
        .clone();

        let merged = merge_json_objects(&first, &second, ConflictResolution::PreferFirst).unwrap();

        assert_eq!(merged.get("name").unwrap(), "Alice");
        assert_eq!(merged.get("age").unwrap(), 30);
        assert_eq!(merged.get("city").unwrap(), "Berlin");
        assert_eq!(merged.get("tags").unwrap(), &json!(["rust", "python"]));
    }

    #[test]
    fn test_merge_arrays() {
        let first = json!({"items": [1, 2]}).as_object().unwrap().clone();
        let second = json!({"items": [3, 4]}).as_object().unwrap().clone();

        let merged = merge_json_objects(&first, &second, ConflictResolution::MergeArrays).unwrap();
        assert_eq!(merged.get("items").unwrap(), &json!([1, 2, 3, 4]));
    }
}use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Array(arr) = json_value {
            for item in arr {
                merged_array.push(item);
            }
        } else {
            merged_array.push(json_value);
        }
    }

    let output_file = File::create(output_path)?;
    let merged_json = json!(merged_array);
    serde_json::to_writer_pretty(output_file, &merged_json)?;

    Ok(())
}

pub fn merge_json_directories(dir_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut json_files = Vec::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    json_files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    let input_refs: Vec<&str> = json_files.iter().map(|s| s.as_str()).collect();
    merge_json_files(&input_refs, output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_merge_json_files() {
        let dir = tempdir().unwrap();
        let file1_path = dir.path().join("a.json");
        let file2_path = dir.path().join("b.json");
        let output_path = dir.path().join("merged.json");

        fs::write(&file1_path, r#"[1, 2, 3]"#).unwrap();
        fs::write(&file2_path, r#"{"key": "value"}"#).unwrap();

        let inputs = [file1_path.to_str().unwrap(), file2_path.to_str().unwrap()];
        merge_json_files(&inputs, output_path.to_str().unwrap()).unwrap();

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("1") && content.contains("value"));
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    let mut processed_keys = HashSet::new();
    let mut conflict_log = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if processed_keys.contains(&key) {
                    conflict_log.push(format!("Conflict detected for key '{}'", key));
                    continue;
                }
                merged.insert(key.clone(), value);
                processed_keys.insert(key);
            }
        }
    }

    if !conflict_log.is_empty() {
        eprintln!("Conflicts found during merge:");
        for log in &conflict_log {
            eprintln!("  {}", log);
        }
    }

    Ok(Value::Object(merged))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "d": 4}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.len(), 4);
        assert_eq!(obj.get("a").unwrap().as_i64(), Some(1));
        assert_eq!(obj.get("c").unwrap().as_i64(), Some(3));
    }

    #[test]
    fn test_merge_with_conflict() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"a": 99, "c": 3}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get("a").unwrap().as_i64(), Some(1));
        assert_eq!(obj.get("b").unwrap().as_i64(), Some(2));
        assert!(!obj.contains_key("c"));
    }
}
use std::collections::HashMap;
use std::fs;
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged = HashMap::new();

    for path in file_paths {
        let content = fs::read_to_string(path)?;
        let json_data: JsonValue = serde_json::from_str(&content)?;

        if let JsonValue::Object(map) = json_data {
            for (key, value) in map {
                merged.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain an object".into());
        }
    }

    Ok(JsonValue::Object(merged.into_iter().collect()))
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

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected: JsonValue = serde_json::from_str(
            r#"{"name": "Alice", "age": 30, "city": "London", "active": true}"#
        ).unwrap();

        assert_eq!(result, expected);
    }
}