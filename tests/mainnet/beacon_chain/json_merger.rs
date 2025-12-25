use serde_json::{Value, json};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;
        merged_array.push(json_value);
    }

    let output_json = json!(merged_array);
    let output_str = serde_json::to_string_pretty(&output_json)?;

    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output_path, output_str)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"{"id": 1, "name": "Alice"}"#).unwrap();
        fs::write(file2.path(), r#"{"id": 2, "name": "Bob"}"#).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        merge_json_files(&paths, output_file.path().to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
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
            "Conflict between values: {:?} and {:?}",
            first, second
        )),
        ConflictResolution::MergeArrays => Err("Cannot merge non-array values with MergeArrays strategy".to_string()),
    }
}