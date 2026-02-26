
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut result = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            merge_objects(&mut result, obj);
        }
    }

    Ok(Value::Object(result))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        match target.get_mut(&key) {
            Some(target_value) => {
                if let (Value::Object(mut target_obj), Value::Object(source_obj)) = (target_value.clone(), source_value.clone()) {
                    let mut merged_obj = Map::new();
                    merge_objects(&mut merged_obj, target_obj);
                    merge_objects(&mut merged_obj, source_obj);
                    target.insert(key, Value::Object(merged_obj));
                } else if target_value != &source_value {
                    let conflict_key = format!("{}_conflict", key);
                    target.insert(conflict_key, source_value);
                }
            }
            None => {
                target.insert(key, source_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;

        fs::write(&file1, r#"{"common": {"a": 1}, "unique1": true}"#)?;
        fs::write(&file2, r#"{"common": {"b": 2}, "unique2": false}"#)?;

        let result = merge_json_files(&[file1.path(), file2.path()])?;
        
        assert_eq!(result["common"]["a"], 1);
        assert_eq!(result["common"]["b"], 2);
        assert_eq!(result["unique1"], true);
        assert_eq!(result["unique2"], false);

        Ok(())
    }
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let json_value: JsonValue = serde_json::from_str(&content)?;
        merged_array.push(json_value);
    }

    let output_json = JsonValue::Array(merged_array);
    let serialized = serde_json::to_string_pretty(&output_json)?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(serialized.as_bytes())?;

    Ok(())
}

pub fn deduplicate_json_array_by_key(array: &mut Vec<JsonValue>, key: &str) {
    let mut seen = HashMap::new();
    array.retain(|item| {
        if let Some(obj) = item.as_object() {
            if let Some(value) = obj.get(key) {
                let key_string = value.to_string();
                if seen.contains_key(&key_string) {
                    return false;
                }
                seen.insert(key_string, true);
                return true;
            }
        }
        true
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_and_deduplicate() {
        let json1 = r#"{"id": 1, "name": "Alice"}"#;
        let json2 = r#"{"id": 2, "name": "Bob"}"#;
        let json3 = r#"{"id": 1, "name": "Alice Duplicate"}"#;

        let files = [json1, json2, json3];
        let temp_files: Vec<NamedTempFile> = files
            .iter()
            .map(|content| {
                let mut file = NamedTempFile::new().unwrap();
                file.write_all(content.as_bytes()).unwrap();
                file
            })
            .collect();

        let paths: Vec<&Path> = temp_files.iter().map(|f| f.path()).collect();
        let output = NamedTempFile::new().unwrap();

        merge_json_files(&paths, output.path()).unwrap();

        let output_content = fs::read_to_string(output.path()).unwrap();
        let mut parsed: JsonValue = serde_json::from_str(&output_content).unwrap();

        if let JsonValue::Array(ref mut arr) = parsed {
            deduplicate_json_array_by_key(arr, "id");
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected JSON array");
        }
    }
}use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, extension: &Value) {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(extension_map)) => {
            for (key, ext_value) in extension_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, ext_value);
                } else {
                    base_map.insert(key.clone(), ext_value.clone());
                }
            }
        }
        (base, extension) => {
            *base = extension.clone();
        }
    }
}

pub fn merge_multiple_json(values: Vec<Value>) -> Option<Value> {
    let mut result = Value::Object(Map::new());
    for value in values {
        merge_json(&mut result, &value);
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let extension = json!({"b": {"d": 3}, "e": 4});
        merge_json(&mut base, &extension);
        assert_eq!(base, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_overwrite_primitive() {
        let mut base = json!({"a": 1});
        let extension = json!({"a": 2});
        merge_json(&mut base, &extension);
        assert_eq!(base, json!({"a": 2}));
    }

    #[test]
    fn test_multiple_merge() {
        let values = vec![
            json!({"a": 1}),
            json!({"b": 2}),
            json!({"a": 3, "c": 4})
        ];
        let result = merge_multiple_json(values).unwrap();
        assert_eq!(result, json!({"a": 3, "b": 2, "c": 4}));
    }
}