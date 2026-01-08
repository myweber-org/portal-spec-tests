use serde_json::{Value, from_reader, to_writer_pretty};
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> io::Result<Value> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = from_reader(reader)?;

        if let Value::Array(arr) = json_value {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_value);
        }
    }

    Ok(Value::Array(merged_array))
}

pub fn write_merged_json<P: AsRef<Path>>(paths: &[P], output_path: P) -> io::Result<()> {
    let merged = merge_json_files(paths)?;
    let output_file = File::create(output_path)?;
    to_writer_pretty(output_file, &merged)?;
    Ok(())
}