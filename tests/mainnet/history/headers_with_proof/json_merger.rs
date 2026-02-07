
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
}