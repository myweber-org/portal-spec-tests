
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use serde_json::{Value, Map};

pub fn merge_json_files(file_paths: &[String], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map: Map<String, Value> = Map::new();

    for path in file_paths {
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

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(merged_map))?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = vec![
        "data1.json".to_string(),
        "data2.json".to_string(),
    ];
    merge_json_files(&files, "merged_output.json")?;
    println!("JSON files merged successfully.");
    Ok(())
}