
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, String> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path_str, e))?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                if merged_map.contains_key(&key) {
                    return Err(format!("Duplicate key '{}' found in {}", key, path_str));
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err(format!("Root element in {} is not a JSON object", path_str));
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    conflict_strategy: ConflictStrategy,
) -> Result<Value, String> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path_str, e))?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        accumulator.insert(key, value);
                    }
                    ConflictStrategy::Skip => {
                        accumulator.entry(key).or_insert(value);
                    }
                    ConflictStrategy::Error => {
                        if accumulator.contains_key(&key) {
                            return Err(format!(
                                "Duplicate key '{}' found in {} (conflict strategy: error)",
                                key, path_str
                            ));
                        }
                        accumulator.insert(key, value);
                    }
                }
            }
        } else {
            return Err(format!("Root element in {} is not a JSON object", path_str));
        }
    }

    let mut map = Map::new();
    for (key, value) in accumulator {
        map.insert(key, value);
    }
    Ok(Value::Object(map))
}

pub enum ConflictStrategy {
    Overwrite,
    Skip,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_json(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_merge_json_files() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"c": 3, "d": 4}"#);

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .unwrap();

        let expected: Value = serde_json::from_str(r#"{"a": 1, "b": 2, "c": 3, "d": 4}"#).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_overwrite_strategy() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"b": 99, "c": 3}"#);

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            ConflictStrategy::Overwrite,
        )
        .unwrap();

        let expected: Value = serde_json::from_str(r#"{"a": 1, "b": 99, "c": 3}"#).unwrap();
        assert_eq!(result, expected);
    }
}