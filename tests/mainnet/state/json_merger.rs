
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::Path;

use serde_json::{Value, Map};

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> io::Result<()> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(obj) => {
                merged_array.push(Value::Object(obj));
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "JSON file must contain either an array or an object",
                ));
            }
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &merged_array)?;

    Ok(())
}

pub fn merge_json_with_key_deduplication<P: AsRef<Path>>(
    paths: &[P],
    output_path: P,
    key_field: &str,
) -> io::Result<()> {
    let mut unique_items: HashMap<String, Value> = HashMap::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        let items = match json_value {
            Value::Array(arr) => arr,
            Value::Object(_) => vec![json_value],
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "JSON file must contain either an array or an object",
                ));
            }
        };

        for item in items {
            if let Value::Object(map) = item {
                if let Some(key_value) = map.get(key_field) {
                    if let Some(key_str) = key_value.as_str() {
                        unique_items.insert(key_str.to_string(), Value::Object(map));
                    }
                }
            }
        }
    }

    let deduplicated_array: Vec<Value> = unique_items.into_values().collect();
    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &deduplicated_array)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_arrays() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(&file2, r#"[{"id": 3}, {"id": 4}]"#).unwrap();

        let paths = [file1.path(), file2.path()];
        merge_json_files(&paths, output_file.path()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_merge_with_deduplication() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": "a", "value": 1}, {"id": "b", "value": 2}]"#).unwrap();
        fs::write(&file2, r#"[{"id": "a", "value": 3}, {"id": "c", "value": 4}]"#).unwrap();

        let paths = [file1.path(), file2.path()];
        merge_json_with_key_deduplication(&paths, output_file.path(), "id").unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);

        let ids: Vec<&str> = array
            .iter()
            .filter_map(|item| item.get("id").and_then(|v| v.as_str()))
            .collect();
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"));
        assert!(ids.contains(&"c"));
    }
}