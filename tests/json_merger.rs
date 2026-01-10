use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashSet::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        if let Some(id_val) = obj.get("id") {
                            if let Some(id_str) = id_val.as_str() {
                                if !seen_ids.contains(id_str) {
                                    seen_ids.insert(id_str.to_string());
                                    merged_array.push(item);
                                }
                                continue;
                            }
                        }
                    }
                    merged_array.push(item);
                }
            }
            Value::Object(_) => merged_array.push(json_value),
            _ => return Err("Input JSON must be an object or array".into()),
        }
    }

    let output_json = json!(merged_array);
    fs::write(output_path, output_json.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_arrays() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": "a", "value": 1}, {"value": 2}]"#).unwrap();
        fs::write(&file2, r#"[{"id": "a", "value": 3}, {"id": "b", "value": 4}]"#).unwrap();

        merge_json_files(&[file1.path(), file2.path()], output.path()).unwrap();

        let content = fs::read_to_string(output.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}