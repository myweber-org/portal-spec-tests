use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file> <input_file1> [input_file2 ...]", args[0]);
        process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let mut merged_map = Map::new();

    for path in input_paths {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to read file {}: {}", path, e);
                process::exit(1);
            }
        };

        let json_value: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse JSON from {}: {}", path, e);
                process::exit(1);
            }
        };

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("File {} does not contain a JSON object", path);
            process::exit(1);
        }
    }

    let merged_json = Value::Object(merged_map);
    let json_string = match serde_json::to_string_pretty(&merged_json) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to serialize merged JSON: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = fs::write(output_path, json_string) {
        eprintln!("Failed to write output file {}: {}", output_path, e);
        process::exit(1);
    }

    println!("Successfully merged {} files into {}", input_paths.len(), output_path);
}