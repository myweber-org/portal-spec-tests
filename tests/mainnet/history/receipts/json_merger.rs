
use std::collections::HashMap;
use std::fs::{self, File};
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
        let path_ref = path.as_ref();
        let file_name = path_ref
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file = File::open(path_ref)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;
        self.data.insert(file_name, json_data);

        Ok(())
    }

    pub fn add_directory<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<()> {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                self.add_file(&path)?;
            }
        }
        Ok(())
    }

    pub fn merge(&self) -> JsonValue {
        let mut merged = JsonValue::Object(serde_json::Map::new());
        for (key, value) in &self.data {
            merged[key] = value.clone();
        }
        merged
    }

    pub fn save_merged<P: AsRef<Path>>(&self, output_path: P) -> Result<()> {
        let merged = self.merge();
        let json_string = serde_json::to_string_pretty(&merged)?;
        fs::write(output_path, json_string)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_merge_json_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("config.json");
        let file2_path = temp_dir.path().join("data.json");

        fs::write(&file1_path, r#"{"port": 8080, "host": "localhost"}"#).unwrap();
        fs::write(&file2_path, r#"{"users": ["alice", "bob"], "active": true}"#).unwrap();

        let mut merger = JsonMerger::new();
        merger.add_file(&file1_path).unwrap();
        merger.add_file(&file2_path).unwrap();

        let merged = merger.merge();
        assert!(merged.get("config").is_some());
        assert!(merged.get("data").is_some());
        assert_eq!(merged["config"]["port"], 8080);
        assert_eq!(merged["data"]["users"][0], "alice");
    }
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

        if let Some(obj_map) = json_value.as_object() {
            for (key, value) in obj_map {
                merged_map.insert(key.clone(), value.clone());
            }
        } else {
            return Err("Each JSON file must contain an object at the root level".into());
        }
    }

    Ok(serde_json::Value::Object(
        merged_map.into_iter().collect()
    ))
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

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
        assert_eq!(result["city"], "London");
        assert_eq!(result["active"], true);
    }

    #[test]
    fn test_merge_with_overwrite() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"id": 1, "value": "old"}"#).unwrap();
        writeln!(file2, r#"{"id": 2, "value": "new"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["id"], 2);
        assert_eq!(result["value"], "new");
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut result = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merge_value(&mut result, key, value);
            }
        }
    }

    Ok(Value::Object(result))
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
                map.insert(key, Value::Array(vec![existing_value.clone(), new_value]));
            }
        }
        None => {
            map.insert(key, new_value);
        }
    }
}