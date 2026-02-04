use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(_) => {
                merged_array.push(json_value);
            }
            _ => {
                eprintln!("Warning: File {} does not contain a JSON object or array, skipping.", input_path);
            }
        }
    }

    let output_file = File::create(output_path)?;
    let merged_json = json!(merged_array);
    serde_json::to_writer_pretty(output_file, &merged_json)?;

    Ok(())
}

pub fn merge_json_from_directory(dir_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut json_files = Vec::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    json_files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    let input_refs: Vec<&str> = json_files.iter().map(|s| s.as_str()).collect();
    merge_json_files(&input_refs, output_path)
}
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(obj) = json_value {
            for (key, value) in obj {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting with value from {}", key, path_str);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object at the root".into());
        }
    }

    Ok(serde_json::Value::Object(serde_json::Map::from_iter(merged_map)))
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
        writeln!(file2, r#"{"city": "London", "age": 31}"#).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let result_obj = result.as_object().unwrap();

        assert_eq!(result_obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(result_obj.get("city").unwrap().as_str().unwrap(), "London");
        assert_eq!(result_obj.get("age").unwrap().as_u64().unwrap(), 31);
    }

    #[test]
    fn test_file_not_found() {
        let result = merge_json_files(&["nonexistent.json"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_json_structure() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"["array", "not", "object"]"#).unwrap();

        let paths = vec![file.path().to_str().unwrap()];
        let result = merge_json_files(&paths);
        assert!(result.is_err());
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(obj) = json {
            for (key, value) in obj {
                merge_value(&mut merged, key, value);
            }
        }
    }
    
    let output_json = Value::Object(merged);
    let output_str = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_str)?;
    
    Ok(())
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get_mut(&key) {
        Some(existing_value) => {
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing_value, &new_value) {
                let mut existing_obj = existing_obj.clone();
                for (nested_key, nested_value) in new_obj {
                    merge_value(&mut existing_obj, nested_key.clone(), nested_value.clone());
                }
                map.insert(key, Value::Object(existing_obj));
            } else if existing_value != &new_value {
                let conflict_key = format!("{}_conflict", key);
                map.insert(conflict_key, new_value);
            }
        }
        None => {
            map.insert(key, new_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use serde_json::json;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"name": "test", "value": 1}"#).unwrap();
        fs::write(&file2, r#"{"name": "test2", "extra": true}"#).unwrap();
        
        merge_json_files(&[file1.path(), file2.path()], output.path()).unwrap();
        
        let result_content = fs::read_to_string(output.path()).unwrap();
        let result: Value = serde_json::from_str(&result_content).unwrap();
        
        assert!(result.get("name_conflict").is_some());
        assert_eq!(result["value"], 1);
        assert_eq!(result["extra"], true);
    }
}