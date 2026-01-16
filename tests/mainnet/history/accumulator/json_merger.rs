use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> io::Result<Value> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Array(arr) = json_value {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_value);
        }
    }

    Ok(json!(merged_array))
}

pub fn merge_and_write<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> io::Result<()> {
    let merged = merge_json_files(input_paths)?;
    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &merged)?;
    Ok(())
}
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
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

    pub fn merge_files(&self, paths: &[&str]) -> Result<Value, String> {
        let mut merged_value = Value::Object(Map::new());

        for path_str in paths {
            let path = Path::new(path_str);
            if !path.exists() {
                return Err(format!("File not found: {}", path_str));
            }

            let content = fs::read_to_string(path)
                .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

            let value: Value = serde_json::from_str(&content)
                .map_err(|e| format!("Invalid JSON in {}: {}", path_str, e))?;

            merged_value = self.merge_values(merged_value, value);
        }

        Ok(merged_value)
    }

    fn merge_values(&self, mut target: Value, source: Value) -> Value {
        match (target, source) {
            (Value::Object(mut target_map), Value::Object(source_map)) => {
                for (key, source_val) in source_map {
                    if let Some(target_val) = target_map.get_mut(&key) {
                        let merged = self.merge_values(target_val.clone(), source_val);
                        target_map.insert(key, merged);
                    } else {
                        target_map.insert(key, source_val);
                    }
                }
                Value::Object(target_map)
            }
            (Value::Array(mut target_arr), Value::Array(source_arr)) => {
                match self.conflict_resolution {
                    ConflictResolution::MergeArrays => {
                        target_arr.extend(source_arr);
                        Value::Array(target_arr)
                    }
                    ConflictResolution::PreferFirst => Value::Array(target_arr),
                    ConflictResolution::PreferSecond => Value::Array(source_arr),
                    ConflictResolution::FailOnConflict => {
                        if target_arr != source_arr {
                            Value::Array(vec![Value::String("CONFLICT".to_string())])
                        } else {
                            Value::Array(target_arr)
                        }
                    }
                }
            }
            (target_val, source_val) => {
                match self.conflict_resolution {
                    ConflictResolution::PreferFirst => target_val,
                    ConflictResolution::PreferSecond => source_val,
                    ConflictResolution::FailOnConflict => {
                        if target_val != source_val {
                            Value::String("CONFLICT".to_string())
                        } else {
                            target_val
                        }
                    }
                    ConflictResolution::MergeArrays => target_val,
                }
            }
        }
    }

    pub fn write_output(&self, value: &Value, output_path: &str) -> Result<(), String> {
        let json_string = serde_json::to_string_pretty(value)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

        fs::write(output_path, json_string)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        Ok(())
    }
}

pub fn merge_json_files(
    input_files: &[&str],
    output_file: &str,
    resolution: ConflictResolution,
) -> Result<(), String> {
    let merger = JsonMerger::new(resolution);
    let merged = merger.merge_files(input_files)?;
    merger.write_output(&merged, output_file)
}