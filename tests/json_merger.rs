
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_object = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_object.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a top-level object".into());
        }
    }

    let merged_json = Value::Object(merged_object);
    let json_string = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, json_string)?;

    Ok(())
}