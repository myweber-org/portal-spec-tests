
use serde_json::{Map, Value};
use std::env;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file.json> <input1.json> [input2.json ...]", args[0]);
        std::process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let mut merged_map = Map::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File '{}' not found, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting with value from '{}'.", key, input_path);
                }
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: File '{}' does not contain a JSON object at root, skipping.", input_path);
        }
    }

    let output_file = File::create(output_path)?;
    let pretty_json = serde_json::to_string_pretty(&Value::Object(merged_map))?;
    write!(&output_file, "{}", pretty_json)?;

    println!("Successfully merged {} file(s) into '{}'.", input_paths.len(), output_path);
    Ok(())
}