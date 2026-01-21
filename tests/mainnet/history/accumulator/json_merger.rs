use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let data: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = data {
            for (key, value) in map {
                merged.insert(key, value);
            }
        } else {
            eprintln!("Warning: {} does not contain a JSON object, skipping.", path_str);
        }
    }

    let output_value = json!(merged);
    let output_json = serde_json::to_string_pretty(&output_value)?;
    fs::write(output_path, output_json)?;

    println!("Merged JSON written to {}", output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(file2.path(), r#"{"c": 3, "d": 4}"#).unwrap();

        let paths = vec![file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        let result = merge_json_files(&paths, output_file.path().to_str().unwrap());

        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], 2);
        assert_eq!(parsed["c"], 3);
        assert_eq!(parsed["d"], 4);
    }
}