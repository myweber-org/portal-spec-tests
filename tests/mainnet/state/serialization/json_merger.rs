
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub struct JsonMerger {
    conflict_resolution: ConflictResolution,
}

pub enum ConflictResolution {
    PreferFirst,
    PreferSecond,
    MergeArrays,
    FailOnConflict,
}

impl JsonMerger {
    pub fn new(resolution: ConflictResolution) -> Self {
        JsonMerger {
            conflict_resolution: resolution,
        }
    }

    pub fn merge_files<P: AsRef<Path>>(&self, paths: &[P]) -> Result<Value, String> {
        if paths.is_empty() {
            return Err("No files provided".to_string());
        }

        let mut merged_value = self.read_json_file(&paths[0])?;

        for path in paths.iter().skip(1) {
            let current_value = self.read_json_file(path)?;
            merged_value = self.merge_values(merged_value, current_value)?;
        }

        Ok(merged_value)
    }

    fn read_json_file<P: AsRef<Path>>(&self, path: P) -> Result<Value, String> {
        let file = File::open(path.as_ref())
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to parse JSON: {}", e))
    }

    fn merge_values(&self, mut target: Value, source: Value) -> Result<Value, String> {
        match (&mut target, source) {
            (Value::Object(target_map), Value::Object(source_map)) => {
                self.merge_objects(target_map, source_map)?;
            }
            (Value::Array(target_arr), Value::Array(source_arr)) => {
                self.merge_arrays(target_arr, source_arr);
            }
            (target_val, source_val) => {
                if target_val != &source_val {
                    return self.handle_conflict(target_val.clone(), source_val);
                }
            }
        }
        Ok(target)
    }

    fn merge_objects(&self, target: &mut Map<String, Value>, source: Map<String, Value>) -> Result<(), String> {
        for (key, source_value) in source {
            match target.get_mut(&key) {
                Some(target_value) => {
                    let merged = self.merge_values(target_value.clone(), source_value)?;
                    *target_value = merged;
                }
                None => {
                    target.insert(key, source_value);
                }
            }
        }
        Ok(())
    }

    fn merge_arrays(&self, target: &mut Vec<Value>, source: Vec<Value>) {
        target.extend(source);
        target.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
        target.dedup();
    }

    fn handle_conflict(&self, target: Value, source: Value) -> Result<Value, String> {
        match &self.conflict_resolution {
            ConflictResolution::PreferFirst => Ok(target),
            ConflictResolution::PreferSecond => Ok(source),
            ConflictResolution::MergeArrays => {
                let mut merged = Vec::new();
                if let Value::Array(arr) = target {
                    merged.extend(arr);
                } else {
                    merged.push(target);
                }
                if let Value::Array(arr) = source {
                    merged.extend(arr);
                } else {
                    merged.push(source);
                }
                Ok(Value::Array(merged))
            }
            ConflictResolution::FailOnConflict => {
                Err(format!("Conflict detected: {:?} vs {:?}", target, source))
            }
        }
    }

    pub fn write_output<P: AsRef<Path>>(&self, value: &Value, output_path: P) -> Result<(), String> {
        let mut file = File::create(output_path.as_ref())
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        let json_string = serde_json::to_string_pretty(value)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
        file.write_all(json_string.as_bytes())
            .map_err(|e| format!("Failed to write output: {}", e))?;
        Ok(())
    }
}

pub fn merge_json_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    resolution: ConflictResolution,
) -> Result<(), String> {
    let merger = JsonMerger::new(resolution);
    let merged = merger.merge_files(input_paths)?;
    merger.write_output(&merged, output_path)
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
        let data2 = json!({"city": "Berlin", "active": true});

        write!(file1, "{}", data1).unwrap();
        write!(file2, "{}", data2).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
            "non_existent.json",
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
}use serde_json::{Map, Value};
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
            "country": "UK"
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
            "age": 30,
            "city": "London",
            "country": "UK"
        });

        assert_eq!(result, expected);
    }
}