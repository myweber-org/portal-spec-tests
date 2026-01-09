
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }

    Ok(Value::Object(merged))
}

fn merge_objects(base: &mut Map<String, Value>, new: Map<String, Value>) {
    for (key, new_value) in new {
        match base.get_mut(&key) {
            Some(Value::Object(existing_obj)) if new_value.is_object() => {
                if let Value::Object(new_obj) = new_value {
                    merge_objects(existing_obj, new_obj);
                }
            }
            Some(existing) if *existing != new_value => {
                let conflict_key = format!("{}_conflict", key);
                let mut conflict_array = vec![existing.clone(), new_value];
                base.insert(conflict_key, Value::Array(conflict_array));
                base.remove(&key);
            }
            _ => {
                base.insert(key, new_value);
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
    fn test_merge_basic() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": {"c": 2}}"#).unwrap();
        fs::write(&file2, r#"{"b": {"d": 3}, "e": 4}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": {"c": 2, "d": 3},
            "e": 4
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_conflict() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1}"#).unwrap();
        fs::write(&file2, r#"{"a": 2}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let conflict_array = result.get("a_conflict").unwrap().as_array().unwrap();
        
        assert!(conflict_array.contains(&json!(1)));
        assert!(conflict_array.contains(&json!(2)));
        assert!(!result.get("a").is_some());
    }
}