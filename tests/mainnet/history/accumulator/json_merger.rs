use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;
        merged_array.push(json_value);
    }

    Ok(json!(merged_array))
}

pub fn merge_and_write<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let merged = merge_json_files(input_paths)?;
    let json_string = serde_json::to_string_pretty(&merged)?;
    fs::write(output_path, json_string)?;
    Ok(())
}