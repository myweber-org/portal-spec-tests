use std::collections::HashMap;
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

        if let serde_json::Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(serde_json::Value::Object(merged_map.into_iter().collect()))
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

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap(), 30);
        assert_eq!(obj.get("city").unwrap(), "Berlin");
        assert_eq!(obj.get("active").unwrap(), true);
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
            merge_objects(&mut merged, obj);
        }
    }
    
    let output_value = Value::Object(merged);
    let output_json = serde_json::to_string_pretty(&output_value)?;
    fs::write(output_path, output_json)?;
    
    Ok(())
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        if let Some(existing_value) = target.get_mut(&key) {
            match (existing_value, source_value) {
                (Value::Object(ref mut target_obj), Value::Object(source_obj)) => {
                    merge_objects(target_obj, source_obj);
                }
                (Value::Array(ref mut target_arr), Value::Array(source_arr)) => {
                    target_arr.extend(source_arr);
                }
                _ => {
                    *existing_value = source_value;
                }
            }
        } else {
            target.insert(key, source_value);
        }
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
            merge_objects(&mut merged, obj);
        }
    }
    
    let output_json = Value::Object(merged);
    let serialized = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, serialized)?;
    
    Ok(())
}

fn merge_objects(base: &mut Map<String, Value>, new: Map<String, Value>) {
    for (key, new_value) in new {
        match base.get_mut(&key) {
            Some(existing_value) => {
                if let (Value::Object(mut existing_obj), Value::Object(new_obj)) = (existing_value.clone(), new_value) {
                    let mut existing_map = if let Value::Object(obj) = existing_obj {
                        obj
                    } else {
                        Map::new()
                    };
                    merge_objects(&mut existing_map, new_obj);
                    base.insert(key, Value::Object(existing_map));
                } else if existing_value != &new_value {
                    base.insert(key, Value::Array(vec![existing_value.clone(), new_value]));
                }
            }
            None => {
                base.insert(key, new_value);
            }
        }
    }
}
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct JsonMerger {
    data: HashMap<String, JsonValue>,
}

impl JsonMerger {
    pub fn new() -> Self {
        JsonMerger {
            data: HashMap::new(),
        }
    }

    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;
        
        if let JsonValue::Object(map) = json_data {
            for (key, value) in map {
                self.data.insert(key, value);
            }
        } else {
            return Err("Root JSON element must be an object".into());
        }

        Ok(())
    }

    pub fn add_raw_json(&mut self, json_str: &str) -> Result<()> {
        let json_data: JsonValue = serde_json::from_str(json_str)?;
        
        if let JsonValue::Object(map) = json_data {
            for (key, value) in map {
                self.data.insert(key, value);
            }
            Ok(())
        } else {
            Err("Root JSON element must be an object".into())
        }
    }

    pub fn merge(&self) -> JsonValue {
        let mut result_map = serde_json::Map::new();
        for (key, value) in &self.data {
            result_map.insert(key.clone(), value.clone());
        }
        JsonValue::Object(result_map)
    }

    pub fn merge_to_string(&self) -> Result<String> {
        let merged = self.merge();
        serde_json::to_string_pretty(&merged).map_err(|e| e.into())
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn key_count(&self) -> usize {
        self.data.len()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}

impl Default for JsonMerger {
    fn default() -> Self {
        Self::new()
    }
}use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let parsed: Value = serde_json::from_str(&content)?;

        match parsed {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(_) => {
                merged_array.push(parsed);
            }
            _ => {
                eprintln!("Warning: File {} does not contain a JSON object or array, skipping.", file_path);
            }
        }
    }

    let output_value = json!(merged_array);
    let output_string = serde_json::to_string_pretty(&output_value)?;
    fs::write(output_path, output_string)?;

    Ok(())
}