use serde_json::{Value, json};
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

        merged_array.push(json_value);
    }

    let output_file = File::create(output_path)?;
    let json_array = json!(merged_array);
    serde_json::to_writer_pretty(output_file, &json_array)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"{"id": 1, "name": "Alice"}"#;
        let json2 = r#"{"id": 2, "name": "Bob"}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), json1).unwrap();
        fs::write(file2.path(), json2).unwrap();

        let input_paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        merge_json_files(&input_paths, output_file.path().to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
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
        ]).unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
        assert_eq!(result["city"], "Berlin");
        assert_eq!(result["active"], true);
    }
}use serde_json::{Map, Value};

pub fn merge_json(base: &mut Value, update: &Value) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, update_value);
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (base, update) => *base = update.clone(),
    }
}

pub fn merge_json_with_strategy(
    base: &mut Value,
    update: &Value,
    array_strategy: fn(&mut Vec<Value>, &Vec<Value>),
) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json_with_strategy(base_value, update_value, array_strategy);
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            array_strategy(base_arr, update_arr);
        }
        (base, update) => *base = update.clone(),
    }
}

pub fn array_merge_replace(base: &mut Vec<Value>, update: &Vec<Value>) {
    base.clear();
    base.extend(update.iter().cloned());
}

pub fn array_merge_append(base: &mut Vec<Value>, update: &Vec<Value>) {
    base.extend(update.iter().cloned());
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let update = json!({"b": {"d": 3}, "e": 4});
        
        merge_json(&mut base, &update);
        
        assert_eq!(base["a"], 1);
        assert_eq!(base["b"]["c"], 2);
        assert_eq!(base["b"]["d"], 3);
        assert_eq!(base["e"], 4);
    }

    #[test]
    fn test_array_replace_strategy() {
        let mut base = json!({"items": [1, 2, 3]});
        let update = json!({"items": [4, 5]});
        
        merge_json_with_strategy(&mut base, &update, array_merge_replace);
        
        assert_eq!(base["items"], json!([4, 5]));
    }
}
use serde_json::{Map, Value};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> Result<(), Box<dyn Error>> {
    let mut merged_map = Map::new();

    for path in input_paths {
        let file = File::open(path)?;
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

    let merged_value = Value::Object(merged_map);
    let mut output_file = File::create(output_path)?;
    write!(output_file, "{}", serde_json::to_string_pretty(&merged_value)?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = r#"{"name": "Alice", "age": 30}"#;
        let file2_content = r#"{"city": "London", "active": true}"#;

        let temp_file1 = NamedTempFile::new().unwrap();
        let temp_file2 = NamedTempFile::new().unwrap();
        let output_temp = NamedTempFile::new().unwrap();

        fs::write(temp_file1.path(), file1_content).unwrap();
        fs::write(temp_file2.path(), file2_content).unwrap();

        let input_paths = [temp_file1.path(), temp_file2.path()];
        merge_json_files(&input_paths, output_temp.path()).unwrap();

        let output_content = fs::read_to_string(output_temp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert_eq!(parsed["name"], "Alice");
        assert_eq!(parsed["age"], 30);
        assert_eq!(parsed["city"], "London");
        assert_eq!(parsed["active"], true);
    }
}use serde_json::{Value, from_reader, to_writer_pretty};
use std::fs::{File, read_dir};
use std::io::{self, BufReader};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(dir_path: P) -> io::Result<Value> {
    let mut merged_array = Vec::new();

    for entry in read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let json_value: Value = from_reader(reader)?;
            merged_array.push(json_value);
        }
    }

    Ok(Value::Array(merged_array))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, json_value: &Value) -> io::Result<()> {
    let output_file = File::create(output_path)?;
    to_writer_pretty(output_file, json_value)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs::write;
    use tempfile::tempdir;

    #[test]
    fn test_merge_json_files() {
        let dir = tempdir().unwrap();
        let file1_path = dir.path().join("data1.json");
        let file2_path = dir.path().join("data2.json");

        write(&file1_path, r#"{"id": 1, "name": "Alice"}"#).unwrap();
        write(&file2_path, r#"{"id": 2, "name": "Bob"}"#).unwrap();

        let merged = merge_json_files(dir.path()).unwrap();
        let expected = json!([{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]);

        assert_eq!(merged, expected);
    }
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(serde_json::Value::Object(merged_map))
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

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], "test");
        assert_eq!(result["c"], true);
        assert_eq!(result["d"], serde_json::json!([1, 2, 3]));
    }
}use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], deduplicate: bool) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashSet::new();

    for path in paths {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if deduplicate {
                        if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                            if seen_keys.insert(id.to_string()) {
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
            Value::Object(obj) => {
                if deduplicate {
                    if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                        if seen_keys.insert(id.to_string()) {
                            merged_array.push(Value::Object(obj));
                        }
                    } else {
                        merged_array.push(Value::Object(obj));
                    }
                } else {
                    merged_array.push(Value::Object(obj));
                }
            }
            _ => return Err("Input JSON must be an array or object".into()),
        }
    }

    Ok(Value::Array(merged_array))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output_path)?;
    serde_json::to_writer_pretty(file, value)?;
    Ok(())
}
use serde_json::{Map, Value};
use std::fs;
use std::io;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> io::Result<()> {
    let mut merged_object = Map::new();

    for path in input_paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_object.insert(key, value);
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Input JSON must be an object",
            ));
        }
    }

    let merged_json = Value::Object(merged_object);
    let pretty_json = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, pretty_json)?;

    Ok(())
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;
type JsonResult = Result<JsonValue, Box<dyn std::error::Error>>;

pub fn merge_json_files(file_paths: &[&str]) -> JsonResult {
    let mut merged = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let json_data: JsonValue = serde_json::from_str(&content)?;
        merged.insert(file_name, json_data);
    }

    Ok(serde_json::to_value(merged)?)
}

pub fn merge_and_write(output_path: &str, input_paths: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let merged = merge_json_files(input_paths)?;
    let json_string = serde_json::to_string_pretty(&merged)?;
    fs::write(output_path, json_string)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"{"key1": "value1"}"#).unwrap();
        fs::write(file2.path(), r#"{"key2": 42}"#).unwrap();

        let paths = &[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(paths).unwrap();
        assert!(result.get("file1").is_some());
        assert!(result.get("file2").is_some());
    }
}