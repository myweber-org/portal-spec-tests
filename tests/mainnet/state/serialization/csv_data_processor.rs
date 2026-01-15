use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn process_csv(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value && record.active {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/filtered.csv";
    let threshold = 100.0;

    process_csv(input_file, output_file, threshold)?;
    println!("Processing completed successfully");
    Ok(())
}