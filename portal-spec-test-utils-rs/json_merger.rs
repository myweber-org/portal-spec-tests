
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged = JsonValue::Object(serde_json::Map::new());

    for path in file_paths {
        let mut file = File::open(path.as_ref())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(map) = json_data {
            if let JsonValue::Object(merged_map) = &mut merged {
                for (key, value) in map {
                    merged_map.insert(key, value);
                }
            }
        } else {
            return Err("Each JSON file must contain an object at root level".into());
        }
    }

    Ok(merged)
}

pub fn merge_json_with_strategy(
    file_paths: &[impl AsRef<Path>],
    conflict_strategy: ConflictStrategy,
) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map: HashMap<String, JsonValue> = HashMap::new();

    for path in file_paths {
        let mut file = File::open(path.as_ref())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(map) = json_data {
            for (key, value) in map {
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        merged_map.insert(key, value);
                    }
                    ConflictStrategy::Skip => {
                        merged_map.entry(key).or_insert(value);
                    }
                    ConflictStrategy::MergeArrays => {
                        if let Some(existing) = merged_map.get_mut(&key) {
                            if existing.is_array() && value.is_array() {
                                if let JsonValue::Array(existing_arr) = existing {
                                    if let JsonValue::Array(new_arr) = value {
                                        existing_arr.extend(new_arr);
                                    }
                                }
                            } else {
                                merged_map.insert(key, value);
                            }
                        } else {
                            merged_map.insert(key, value);
                        }
                    }
                }
            }
        } else {
            return Err("Each JSON file must contain an object at root level".into());
        }
    }

    let json_map: serde_json::Map<String, JsonValue> = merged_map.into_iter().collect();
    Ok(JsonValue::Object(json_map))
}

#[derive(Clone, Copy)]
pub enum ConflictStrategy {
    Overwrite,
    Skip,
    MergeArrays,
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
    fn test_basic_merge() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"c": 3, "d": 4}"#);

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected: JsonValue = serde_json::from_str(r#"{"a":1,"b":2,"c":3,"d":4}"#).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_conflict_overwrite() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"b": 99, "c": 3}"#);

        let result = merge_json_with_strategy(
            &[file1.path(), file2.path()],
            ConflictStrategy::Overwrite,
        )
        .unwrap();

        let expected: JsonValue = serde_json::from_str(r#"{"a":1,"b":99,"c":3}"#).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_arrays() {
        let file1 = create_temp_json(r#"{"items": [1, 2], "data": "test"}"#);
        let file2 = create_temp_json(r#"{"items": [3, 4], "extra": true}"#);

        let result = merge_json_with_strategy(
            &[file1.path(), file2.path()],
            ConflictStrategy::MergeArrays,
        )
        .unwrap();

        let items = &result["items"];
        assert!(items.is_array());
        assert_eq!(items.as_array().unwrap().len(), 4);
        assert_eq!(result["data"], "test");
        assert_eq!(result["extra"], true);
    }
}