use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"a": 1, "b": "test"}"#).unwrap();
        writeln!(file2, r#"{"c": true, "d": [1,2,3]}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];
        let result = merge_json_files(&paths).unwrap();

        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], "test");
        assert_eq!(result["c"], true);
        assert_eq!(result["d"], serde_json::json!([1, 2, 3]));
    }
}
use serde_json::{Map, Value};
use std::env;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file.json> <input1.json> [input2.json ...]", args[0]);
        std::process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let mut merged_map = Map::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File '{}' not found, skipping.", input_path);
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
            eprintln!("Warning: '{}' does not contain a JSON object, skipping.", input_path);
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(merged_map))?;

    println!("Successfully merged {} file(s) into '{}'", input_paths.len(), output_path);
    Ok(())
}use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "Berlin", "active": true}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
        assert_eq!(result["city"], "Berlin");
        assert_eq!(result["active"], true);
    }

    #[test]
    fn test_merge_with_missing_file() {
        let mut file1 = NamedTempFile::new().unwrap();
        writeln!(file1, r#"{"data": "test"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            "non_existent_file.json",
        ])
        .unwrap();

        assert_eq!(result["data"], "test");
        assert!(result.get("non_existent").is_none());
    }
}