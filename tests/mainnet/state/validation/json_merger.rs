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
use serde_json::{Map, Value};
use std::env;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file> <input_file1> [input_file2 ...]", args[0]);
        std::process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let mut merged_map = Map::new();

    for (index, path) in input_paths.iter().enumerate() {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        let key = match json_value {
            Value::Object(ref map) if map.contains_key("name") => {
                map.get("name").and_then(|v| v.as_str()).unwrap_or(&format!("file_{}", index))
            }
            _ => &format!("file_{}", index),
        };

        merged_map.insert(key.to_string(), json_value);
    }

    let merged_value = Value::Object(merged_map);
    let json_string = serde_json::to_string_pretty(&merged_value)?;
    fs::write(output_path, json_string)?;

    println!("Successfully merged {} files into {}", input_paths.len(), output_path);
    Ok(())
}
use std::collections::HashMap;
use std::fs;
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[&str]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_data: JsonValue = serde_json::from_str(&content)?;

        if let JsonValue::Object(map) = json_data {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    let merged_value = JsonValue::Object(
        merged_map
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect()
    );

    Ok(merged_value)
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

        writeln!(file1, r#"{"name": "test", "count": 42}"#).unwrap();
        writeln!(file2, r#"{"enabled": true, "tags": ["rust", "json"]}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["name"], "test");
        assert_eq!(result["count"], 42);
        assert_eq!(result["enabled"], true);
        assert!(result["tags"].is_array());
    }
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashMap::new();
    let mut duplicate_count = 0;

    for path in paths {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let json_value: JsonValue = serde_json::from_str(&content)?;

        match json_value {
            JsonValue::Array(arr) => {
                for item in arr {
                    if let Some(key) = extract_unique_key(&item) {
                        if seen_keys.contains_key(&key) {
                            duplicate_count += 1;
                            continue;
                        }
                        seen_keys.insert(key.clone(), true);
                    }
                    merged_array.push(item);
                }
            }
            JsonValue::Object(_) => {
                if let Some(key) = extract_unique_key(&json_value) {
                    if seen_keys.contains_key(&key) {
                        duplicate_count += 1;
                        continue;
                    }
                    seen_keys.insert(key.clone(), true);
                }
                merged_array.push(json_value);
            }
            _ => return Err("Input JSON must be either an object or array".into()),
        }
    }

    if duplicate_count > 0 {
        eprintln!("Warning: Skipped {} duplicate entries", duplicate_count);
    }

    let output_json = JsonValue::Array(merged_array);
    let mut output_file = File::create(output_path)?;
    write!(output_file, "{}", serde_json::to_string_pretty(&output_json)?)?;

    Ok(())
}

fn extract_unique_key(value: &JsonValue) -> Option<String> {
    if let JsonValue::Object(map) = value {
        if let Some(id) = map.get("id").and_then(|v| v.as_str()) {
            return Some(id.to_string());
        }
        if let Some(uuid) = map.get("uuid").and_then(|v| v.as_str()) {
            return Some(uuid.to_string());
        }
        if let Some(name) = map.get("name").and_then(|v| v.as_str()) {
            return Some(name.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_arrays() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]"#).unwrap();
        fs::write(&file2, r#"[{"id": "3", "name": "Charlie"}, {"id": "1", "name": "Duplicate"}]"#).unwrap();

        let paths = [file1.path(), file2.path()];
        merge_json_files(&paths, output_file.path()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: JsonValue = serde_json::from_str(&content).unwrap();

        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);
    }
}