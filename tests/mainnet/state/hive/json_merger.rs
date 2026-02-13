
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = merged.get(&key).unwrap();
                    if existing != &value {
                        let conflict_key = format!("{}_conflict", key);
                        merged.insert(conflict_key, value);
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        }
    }

    Ok(Value::Object(merged))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        fs::write(&file2, r#"{"city": "Berlin", "age": 31}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        
        assert_eq!(result["name"], "Alice");
        assert_eq!(result["city"], "Berlin");
        assert_eq!(result["age"], 30);
        assert_eq!(result["age_conflict"], 31);
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

        if let serde_json::Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(serde_json::Value::Object(merged_map.into_iter().collect()))
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

    pub fn merge(&self) -> JsonValue {
        let mut result_map = serde_json::Map::new();
        for (key, value) in &self.data {
            result_map.insert(key.clone(), value.clone());
        }
        JsonValue::Object(result_map)
    }

    pub fn merge_files<P: AsRef<Path>>(paths: &[P]) -> Result<JsonValue> {
        let mut merger = JsonMerger::new();
        for path in paths {
            merger.add_file(path)?;
        }
        Ok(merger.merge())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_json(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_basic_merge() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"c": 3, "d": 4}"#);
        
        let result = JsonMerger::merge_files(&[file1.path(), file2.path()]).unwrap();
        let obj = result.as_object().unwrap();
        
        assert_eq!(obj.get("a").unwrap().as_i64().unwrap(), 1);
        assert_eq!(obj.get("b").unwrap().as_i64().unwrap(), 2);
        assert_eq!(obj.get("c").unwrap().as_i64().unwrap(), 3);
        assert_eq!(obj.get("d").unwrap().as_i64().unwrap(), 4);
    }

    #[test]
    fn test_overwrite_behavior() {
        let file1 = create_temp_json(r#"{"key": "first"}"#);
        let file2 = create_temp_json(r#"{"key": "second"}"#);
        
        let result = JsonMerger::merge_files(&[file1.path(), file2.path()]).unwrap();
        let obj = result.as_object().unwrap();
        
        assert_eq!(obj.get("key").unwrap().as_str().unwrap(), "second");
    }
}