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
}