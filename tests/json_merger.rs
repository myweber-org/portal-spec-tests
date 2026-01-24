
use std::collections::HashMap;
use std::fs;
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[&str]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path).into());
        }

        let content = fs::read_to_string(path)?;
        let json_data: JsonValue = serde_json::from_str(&content)?;

        if let JsonValue::Object(map) = json_data {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let merged_value = JsonValue::Object(
        merged_map
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect()
    );

    Ok(merged_value)
}

pub fn write_merged_json(output_path: &str, json_value: &JsonValue) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(json_value)?;
    fs::write(output_path, json_string)?;
    Ok(())
}use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], deduplicate: bool) -> Result<Value, String> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashSet::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader
            .read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if deduplicate {
                        if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                            if seen_keys.insert(id.to_string()) {
                                merged_array.push(item);
                            }
                        } else {
                            merged_array.push(item);
                        }
                    } else {
                        merged_array.push(item);
                    }
                }
            }
            _ => return Err("Expected JSON array in each file".to_string()),
        }
    }

    Ok(Value::Array(merged_array))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, value: &Value) -> Result<(), String> {
    let file = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    serde_json::to_writer_pretty(file, value)
        .map_err(|e| format!("Failed to write JSON: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = json!([{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]);
        let json2 = json!([{"id": "3", "name": "Charlie"}, {"id": "1", "name": "Duplicate"}]);

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(json1.to_string().as_bytes()).unwrap();
        file2.write_all(json2.to_string().as_bytes()).unwrap();

        let paths = [file1.path(), file2.path()];
        let result = merge_json_files(&paths, false).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 4);

        let dedup_result = merge_json_files(&paths, true).unwrap();
        assert_eq!(dedup_result.as_array().unwrap().len(), 3);
    }
}