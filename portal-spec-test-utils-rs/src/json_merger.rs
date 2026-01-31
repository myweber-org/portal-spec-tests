
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        }
    }

    Ok(Value::Object(merged))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, value) in source {
        match (target.get(&key), value) {
            (Some(Value::Object(existing_obj)), Value::Object(new_obj)) => {
                let mut existing = existing_obj.clone();
                merge_objects(&mut existing, new_obj);
                target.insert(key, Value::Object(existing));
            }
            (Some(Value::Array(existing_arr)), Value::Array(new_arr)) => {
                let mut combined = existing_arr.clone();
                combined.extend(new_arr);
                target.insert(key, Value::Array(combined));
            }
            (Some(existing), new) if existing != &new => {
                let conflict_key = format!("{}_conflict", key);
                target.insert(conflict_key, new);
            }
            (_, new_value) => {
                target.insert(key, new_value);
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
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"name": "test", "value": 1}"#).unwrap();
        fs::write(&file2, r#"{"enabled": true, "tags": ["rust"]}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "name": "test",
            "value": 1,
            "enabled": true,
            "tags": ["rust"]
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_nested_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"config": {"timeout": 30}}"#).unwrap();
        fs::write(&file2, r#"{"config": {"retries": 3}}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "config": {
                "timeout": 30,
                "retries": 3
            }
        });

        assert_eq!(result, expected);
    }
}