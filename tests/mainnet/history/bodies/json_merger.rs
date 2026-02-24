
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map: HashMap<String, Value> = HashMap::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object, skipping.", input_path);
        }
    }

    let mut sorted_map = Map::new();
    let mut keys: Vec<&String> = merged_map.keys().collect();
    keys.sort();

    for key in keys {
        sorted_map.insert(key.clone(), merged_map[key].clone());
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(sorted_map))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"{"name": "Alice", "age": 30}"#;
        let json2 = r#"{"city": "London", "country": "UK"}"#;
        let json3 = r#"{"name": "Bob", "active": true}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let file3 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        std::fs::write(file1.path(), json1).unwrap();
        std::fs::write(file2.path(), json2).unwrap();
        std::fs::write(file3.path(), json3).unwrap();

        let inputs = &[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
            file3.path().to_str().unwrap(),
        ];

        merge_json_files(inputs, output_file.path().to_str().unwrap()).unwrap();

        let mut content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["name"], "Bob");
        assert_eq!(parsed["age"], 30);
        assert_eq!(parsed["city"], "London");
        assert_eq!(parsed["country"], "UK");
        assert_eq!(parsed["active"], true);
    }

    #[test]
    fn test_merge_with_missing_file() {
        let json1 = r#"{"data": "test"}"#;
        let file1 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        std::fs::write(file1.path(), json1).unwrap();

        let inputs = &[
            file1.path().to_str().unwrap(),
            "non_existent_file.json",
        ];

        let result = merge_json_files(inputs, output_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let mut content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["data"], "test");
    }
}