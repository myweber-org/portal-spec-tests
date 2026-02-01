use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashMap::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Array(arr) = json_value {
            for item in arr {
                if let Some(obj) = item.as_object() {
                    if let Some(id_value) = obj.get("id") {
                        if let Some(id_str) = id_value.as_str() {
                            if seen_ids.contains_key(id_str) {
                                eprintln!("Duplicate ID '{}' found in {}, skipping.", id_str, file_path);
                                continue;
                            }
                            seen_ids.insert(id_str.to_string(), ());
                        }
                    }
                }
                merged_array.push(item);
            }
        } else {
            return Err("Each JSON file must contain a JSON array at its root.".into());
        }
    }

    let output_json = json!(merged_array);
    fs::write(output_path, output_json.to_string())?;
    println!("Successfully merged {} files into {}", file_paths.len(), output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = r#"[{"id": "a", "data": 1}, {"id": "b", "data": 2}]"#;
        let file2_content = r#"[{"id": "c", "data": 3}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let paths = vec![file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        let result = merge_json_files(&paths, output_file.path().to_str().unwrap());

        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}