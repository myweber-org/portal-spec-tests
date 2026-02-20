use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Top-level JSON value must be an object".into());
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let json1 = json!({
            "name": "Alice",
            "age": 30
        });
        let json2 = json!({
            "city": "London",
            "age": 31
        });

        write!(file1, "{}", json1.to_string()).unwrap();
        write!(file2, "{}", json2.to_string()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 31,
            "city": "London"
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_file_not_found() {
        let result = merge_json_files(&["nonexistent.json"]);
        assert!(result.is_err());
    }
}
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    let mut conflict_log = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = merged.get(&key).unwrap();
                    if existing != &value {
                        conflict_log.push(format!(
                            "Conflict for key '{}': existing {:?}, new {:?}",
                            key, existing, value
                        ));
                        merged.insert(key, resolve_conflict(existing, &value));
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        }
    }

    let output = Value::Object(merged);
    fs::write(output_path, serde_json::to_string_pretty(&output)?)?;

    if !conflict_log.is_empty() {
        let log_path = "merge_conflicts.log";
        fs::write(log_path, conflict_log.join("\n"))?;
        println!("Conflicts detected, see {}", log_path);
    }

    Ok(())
}

fn resolve_conflict(v1: &Value, v2: &Value) -> Value {
    match (v1, v2) {
        (Value::Array(a1), Value::Array(a2)) => {
            let mut combined = a1.clone();
            combined.extend(a2.clone());
            Value::Array(combined)
        }
        (Value::Object(o1), Value::Object(o2)) => {
            let mut merged = o1.clone();
            for (k, v) in o2 {
                merged.insert(k.clone(), v.clone());
            }
            Value::Object(merged)
        }
        _ => v2.clone(),
    }
}

pub fn find_unique_keys<P: AsRef<Path>>(paths: &[P]) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let mut all_keys = HashSet::new();
    let mut common_keys = HashSet::new();
    let mut first = true;

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            let keys: HashSet<String> = obj.keys().cloned().collect();
            if first {
                common_keys = keys.clone();
                first = false;
            } else {
                common_keys = common_keys.intersection(&keys).cloned().collect();
            }
            all_keys.extend(keys);
        }
    }

    let unique_keys: HashSet<String> = all_keys.difference(&common_keys).cloned().collect();
    Ok(unique_keys)
}use serde_json::{Map, Value};
use std::env;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file> <input_file1> [input_file2 ...]", args[0]);
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
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: '{}' does not contain a JSON object, skipping.", input_path);
        }
    }

    let output_file = File::create(output_path)?;
    let pretty_json = serde_json::to_string_pretty(&Value::Object(merged_map))?;
    write!(&output_file, "{}", pretty_json)?;

    println!("Successfully merged {} file(s) into '{}'.", input_paths.len(), output_path);
    Ok(())
}