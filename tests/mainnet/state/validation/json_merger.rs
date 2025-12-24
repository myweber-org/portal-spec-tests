use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();
    let mut key_counter: HashMap<String, usize> = HashMap::new();

    for file_path in file_paths {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                let mut final_key = key.clone();
                while merged_map.contains_key(&final_key) {
                    let count = key_counter.entry(key.clone()).or_insert(1);
                    final_key = format!("{}_{}", key, count);
                    *count += 1;
                }
                merged_map.insert(final_key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object at its root".into());
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn write_merged_json(output_path: impl AsRef<Path>, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output_path)?;
    serde_json::to_writer_pretty(file, value)?;
    Ok(())
}use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Array(arr) = json_value {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_value);
        }
    }

    let output = json!(merged_array);
    let mut output_file = File::create(output_path)?;
    write!(output_file, "{}", output.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"{"id": 1, "name": "Alice"}"#;
        let json2 = r#"[{"id": 2, "name": "Bob"}, {"id": 3, "name": "Charlie"}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), json1).unwrap();
        fs::write(file2.path(), json2).unwrap();

        let input_paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        merge_json_files(&input_paths, output_file.path().to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert!(parsed.is_array());
        let arr = parsed.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["id"], 1);
        assert_eq!(arr[1]["id"], 2);
        assert_eq!(arr[2]["id"], 3);
    }
}
use serde_json::{Value, Map};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use std::env;

fn merge_json_files(file_paths: &[String]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file> <input_file1> [input_file2 ...]", args[0]);
        std::process::exit(1);
    }

    let output_path = &args[1];
    let input_files = &args[2..];

    let merged_json = merge_json_files(input_files)?;

    let mut output_file = File::create(output_path)?;
    let json_string = serde_json::to_string_pretty(&merged_json)?;
    output_file.write_all(json_string.as_bytes())?;

    println!("Successfully merged {} JSON files into {}", input_files.len(), output_path);
    Ok(())
}