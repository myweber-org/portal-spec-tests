use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_objects = HashSet::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", file_path);
            continue;
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    let serialized = serde_json::to_string(&item)?;
                    if seen_objects.insert(serialized) {
                        merged_array.push(item);
                    }
                }
            }
            Value::Object(_) => {
                let serialized = serde_json::to_string(&json_value)?;
                if seen_objects.insert(serialized) {
                    merged_array.push(json_value);
                }
            }
            _ => {
                eprintln!("Warning: File {} does not contain a JSON object or array, skipping.", file_path);
            }
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &json!(merged_array))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#;
        let json2 = r#"{"id": 3, "name": "Charlie"}"#;
        let json3 = r#"[{"id": 1, "name": "Alice"}, {"id": 4, "name": "David"}]"#;

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        let mut file3 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        file1.write_all(json1.as_bytes()).unwrap();
        file2.write_all(json2.as_bytes()).unwrap();
        file3.write_all(json3.as_bytes()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
            file3.path().to_str().unwrap(),
        ];

        merge_json_files(&paths, output_file.path().to_str().unwrap()).unwrap();

        let output_contents = std::fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_contents).unwrap();

        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 4);
    }
}