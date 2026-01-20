
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use serde_json::{Value, Map};

fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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

    let merged_value = Value::Object(merged_map);
    let mut output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(&mut output_file, &merged_value)?;
    output_file.flush()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files_to_merge = vec!["data1.json", "data2.json", "data3.json"];
    let output_file = "merged_output.json";

    merge_json_files(&files_to_merge, output_file)?;
    println!("Successfully merged JSON files into {}", output_file);

    Ok(())
}