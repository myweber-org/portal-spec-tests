use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::io::{self, Write};

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File '{}' not found, skipping.", path_str);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let parsed: Value = serde_json::from_str(&content)?;

        match parsed {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            _ => {
                merged_array.push(parsed);
            }
        }
    }

    let output_value = json!(merged_array);
    let output_string = serde_json::to_string_pretty(&output_value)?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(output_string.as_bytes())?;

    println!("Successfully merged {} files into '{}'", input_paths.len(), output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_objects() {
        let file1_content = r#"{"id": 1, "name": "Alice"}"#;
        let file2_content = r#"{"id": 2, "name": "Bob"}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let input_paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        merge_json_files(&input_paths, output_file.path().to_str().unwrap()).unwrap();

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 2);
        assert_eq!(array[0]["name"], "Alice");
        assert_eq!(array[1]["name"], "Bob");
    }
}