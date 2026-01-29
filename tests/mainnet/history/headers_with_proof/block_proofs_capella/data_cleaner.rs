use std::collections::HashSet;

pub fn clean_string_data(strings: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for s in strings {
        let normalized = s.trim().to_lowercase();
        if !normalized.is_empty() && seen.insert(normalized.clone()) {
            result.push(normalized);
        }
    }

    result.sort();
    result
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        if record.age > 120 {
            record.age = 120;
        }
        
        record.name = record.name.trim().to_string();
        
        if record.name.is_empty() {
            record.name = "Unknown".to_string();
        }

        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    clean_csv("input.csv", "cleaned_output.csv")?;
    println!("Data cleaning completed successfully");
    Ok(())
}