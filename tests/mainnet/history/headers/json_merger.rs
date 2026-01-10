
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use serde_json::{Value, Map};

pub fn merge_json_files(file_paths: &[String], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map: Map<String, Value> = Map::new();

    for path in file_paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(merged_map))?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = vec![
        "data1.json".to_string(),
        "data2.json".to_string(),
    ];
    merge_json_files(&files, "merged_output.json")?;
    println!("JSON files merged successfully.");
    Ok(())
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged_map = Map::new();

    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        let json_value: Value = serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json_value {
            merge_objects(&mut merged_map, obj);
        } else {
            return Err(format!("Top-level element in {} must be a JSON object", path.as_ref().display()));
        }
    }

    Ok(Value::Object(merged_map))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        if let Some(existing_value) = target.get_mut(&key) {
            match (existing_value, source_value) {
                (Value::Object(existing_obj), Value::Object(source_obj)) => {
                    merge_objects(existing_obj, source_obj);
                }
                (Value::Array(existing_arr), Value::Array(source_arr)) => {
                    existing_arr.extend(source_arr);
                }
                (existing, new) => {
                    *existing = new;
                }
            }
        } else {
            target.insert(key, source_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_basic_objects() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "b": {"y": 20}}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": {"x": 10, "y": 20},
            "c": 3
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_arrays() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"items": [1, 2]}"#).unwrap();
        fs::write(&file2, r#"{"items": [3, 4]}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "items": [1, 2, 3, 4]
        });

        assert_eq!(result, expected);
    }
}