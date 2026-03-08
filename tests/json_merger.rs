
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

type JsonValue = serde_json::Value;
type JsonObject = serde_json::Map<String, JsonValue>;

pub fn merge_json_files(file_paths: &[&str]) -> Result<JsonValue, String> {
    let mut merged_object = JsonObject::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open {}: {}", path_str, e))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        let json_value: JsonValue = serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid JSON in {}: {}", path_str, e))?;

        if let JsonValue::Object(obj) = json_value {
            for (key, value) in obj {
                merged_object.insert(key, value);
            }
        } else {
            return Err(format!("JSON root in {} is not an object", path_str));
        }
    }

    Ok(JsonValue::Object(merged_object))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    conflict_strategy: ConflictStrategy,
) -> Result<JsonValue, String> {
    let mut merged_object = JsonObject::new();
    let mut key_sources: HashMap<String, Vec<String>> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open {}: {}", path_str, e))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        let json_value: JsonValue = serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid JSON in {}: {}", path_str, e))?;

        if let JsonValue::Object(obj) = json_value {
            for (key, value) in obj {
                key_sources.entry(key.clone()).or_default().push(path_str.to_string());
                
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        merged_object.insert(key, value);
                    }
                    ConflictStrategy::Skip => {
                        if !merged_object.contains_key(&key) {
                            merged_object.insert(key, value);
                        }
                    }
                    ConflictStrategy::MergeObjects => {
                        if let Some(existing) = merged_object.get(&key) {
                            if existing.is_object() && value.is_object() {
                                if let (JsonValue::Object(mut existing_obj), JsonValue::Object(new_obj)) = (existing.clone(), value) {
                                    for (k, v) in new_obj {
                                        existing_obj.insert(k, v);
                                    }
                                    merged_object.insert(key, JsonValue::Object(existing_obj));
                                }
                            } else {
                                merged_object.insert(key, value);
                            }
                        } else {
                            merged_object.insert(key, value);
                        }
                    }
                }
            }
        } else {
            return Err(format!("JSON root in {} is not an object", path_str));
        }
    }

    if let ConflictStrategy::Skip = conflict_strategy {
        let duplicates: Vec<_> = key_sources
            .iter()
            .filter(|(_, sources)| sources.len() > 1)
            .map(|(key, sources)| (key.clone(), sources.clone()))
            .collect();
        
        if !duplicates.is_empty() {
            eprintln!("Warning: Duplicate keys found (skipped):");
            for (key, sources) in duplicates {
                eprintln!("  '{}' from: {}", key, sources.join(", "));
            }
        }
    }

    Ok(JsonValue::Object(merged_object))
}

#[derive(Debug, Clone, Copy)]
pub enum ConflictStrategy {
    Overwrite,
    Skip,
    MergeObjects,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;
    use tempfile::tempdir;

    #[test]
    fn test_merge_basic() {
        let dir = tempdir().unwrap();
        
        let file1 = dir.path().join("a.json");
        write(&file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        
        let file2 = dir.path().join("b.json");
        write(&file2, r#"{"city": "London", "country": "UK"}"#).unwrap();

        let result = merge_json_files(&[
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
        ]).unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "London");
        assert_eq!(obj.len(), 4);
    }

    #[test]
    fn test_conflict_overwrite() {
        let dir = tempdir().unwrap();
        
        let file1 = dir.path().join("a.json");
        write(&file1, r#"{"id": 1, "value": "first"}"#).unwrap();
        
        let file2 = dir.path().join("b.json");
        write(&file2, r#"{"id": 2, "value": "second"}"#).unwrap();

        let result = merge_json_with_strategy(
            &[file1.to_str().unwrap(), file2.to_str().unwrap()],
            ConflictStrategy::Overwrite,
        ).unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("id").unwrap().as_u64().unwrap(), 2);
        assert_eq!(obj.get("value").unwrap().as_str().unwrap(), "second");
    }
}
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let json: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                merge_value(&mut merged, key, value);
            }
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }

    Ok(Value::Object(merged))
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get_mut(&key) {
        Some(existing) => {
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, &new_value) {
                for (nested_key, nested_value) in new_obj {
                    merge_value(existing_obj, nested_key.clone(), nested_value.clone());
                }
            } else if existing != &new_value {
                let conflict_key = format!("{}_conflict", key);
                let conflict_array = match map.get_mut(&conflict_key) {
                    Some(Value::Array(arr)) => arr,
                    _ => {
                        let arr = vec![existing.clone()];
                        map.insert(conflict_key.clone(), Value::Array(arr));
                        map.get_mut(&conflict_key).unwrap().as_array_mut().unwrap()
                    }
                };
                conflict_array.push(new_value);
            }
        }
        None => {
            map.insert(key, new_value);
        }
    }
}
use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file.json> <input1.json> [input2.json ...]", args[0]);
        process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let mut merged_map = Map::new();

    for input_path in input_paths {
        let content = match fs::read_to_string(input_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to read {}: {}", input_path, e);
                process::exit(1);
            }
        };

        let json_value: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse JSON from {}: {}", input_path, e);
                process::exit(1);
            }
        };

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: {} does not contain a JSON object at root. Skipping.", input_path);
        }
    }

    let merged_value = Value::Object(merged_map);
    let json_string = match serde_json::to_string_pretty(&merged_value) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to serialize merged JSON: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = fs::write(output_path, json_string) {
        eprintln!("Failed to write to {}: {}", output_path, e);
        process::exit(1);
    }

    println!("Successfully merged {} files into {}", input_paths.len(), output_path);
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
        writeln!(file2, r#"{"city": "London", "active": true}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap(), 30);
        assert_eq!(obj.get("city").unwrap(), "London");
        assert_eq!(obj.get("active").unwrap(), true);
    }
}use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
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
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = json!({"name": "Alice", "age": 30});
        let file2_content = json!({"city": "Berlin", "active": true});

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content.to_string()).unwrap();
        fs::write(file2.path(), file2_content.to_string()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 30,
            "city": "Berlin",
            "active": true
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_missing_file() {
        let file_content = json!({"data": "test"});
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), file_content.to_string()).unwrap();

        let paths = vec![
            file.path().to_str().unwrap(),
            "non_existent_file.json",
        ];

        let result = merge_json_files(&paths).unwrap();
        assert_eq!(result, json!({"data": "test"}));
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        }
    }

    Ok(Value::Object(merged))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        match target.get_mut(&key) {
            Some(target_value) => {
                if let (Value::Object(mut target_obj), Value::Object(source_obj)) = (target_value.clone(), source_value.clone()) {
                    let mut source_map = source_obj;
                    merge_objects(&mut target_obj, source_map);
                    target.insert(key, Value::Object(target_obj));
                } else if target_value != &source_value {
                    let merged_array = Value::Array(vec![target_value.clone(), source_value]);
                    target.insert(key, merged_array);
                }
            }
            None => {
                target.insert(key, source_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;

        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#)?;
        fs::write(&file2, r#"{"b": {"y": 20}, "c": 3}"#)?;

        let result = merge_json_files(&[file1.path(), file2.path()])?;
        let expected = json!({
            "a": 1,
            "b": {"x": 10, "y": 20},
            "c": 3
        });

        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_merge_conflict() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;

        fs::write(&file1, r#"{"key": "first"}"#)?;
        fs::write(&file2, r#"{"key": "second"}"#)?;

        let result = merge_json_files(&[file1.path(), file2.path()])?;
        let expected = json!({
            "key": ["first", "second"]
        });

        assert_eq!(result, expected);
        Ok(())
    }
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
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let data1 = json!({"name": "Alice", "age": 30});
        let data2 = json!({"city": "London", "active": true});

        write!(file1, "{}", data1.to_string()).unwrap();
        write!(file2, "{}", data2.to_string()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
            "non_existent.json",
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 30,
            "city": "London",
            "active": true
        });

        assert_eq!(result, expected);
    }
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashMap::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_content: JsonValue = serde_json::from_reader(reader)?;

        match json_content {
            JsonValue::Array(arr) => {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                            if !seen_keys.contains_key(id) {
                                seen_keys.insert(id.to_string(), true);
                                merged_array.push(item);
                            }
                        } else {
                            merged_array.push(item);
                        }
                    } else {
                        merged_array.push(item);
                    }
                }
            }
            JsonValue::Object(_) => {
                merged_array.push(json_content);
            }
            _ => {
                eprintln!("Warning: File {} does not contain JSON object or array.", path_str);
            }
        }
    }

    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(&mut writer, &merged_array)?;
    writer.flush()?;

    println!("Successfully merged {} items into {}", merged_array.len(), output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"[{"id": "a", "value": 1}, {"id": "b", "value": 2}]"#;
        let json2 = r#"[{"id": "b", "value": 3}, {"id": "c", "value": 4}]"#;

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(json1.as_bytes()).unwrap();
        file2.write_all(json2.as_bytes()).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path().to_str().unwrap();

        let inputs = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&inputs, output_path);
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_path).unwrap();
        let parsed: JsonValue = serde_json::from_str(&output_content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}