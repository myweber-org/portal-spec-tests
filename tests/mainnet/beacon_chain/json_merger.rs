use serde_json::{Value, from_reader, to_writer_pretty};
use std::fs::{File, read_dir};
use std::io::{self, BufReader};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(dir_path: P) -> io::Result<Value> {
    let mut merged_array = Vec::new();
    
    for entry in read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let json_value: Value = from_reader(reader)?;
            
            if let Value::Array(arr) = json_value {
                merged_array.extend(arr);
            } else {
                merged_array.push(json_value);
            }
        }
    }
    
    Ok(Value::Array(merged_array))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, json_value: &Value) -> io::Result<()> {
    let output_file = File::create(output_path)?;
    to_writer_pretty(output_file, json_value)?;
    Ok(())
}