use csv::Reader;
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

fn process_data(file_path: &str, threshold: f64) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    
    let mut filtered_records = Vec::new();
    
    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= threshold && record.active {
            filtered_records.push(record);
        }
    }
    
    filtered_records.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
    
    Ok(filtered_records)
}

fn export_results(records: &[Record], output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = csv::Writer::from_writer(file);
    
    for record in records {
        writer.serialize(record)?;
    }
    
    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    let threshold = 50.0;
    
    let processed_data = process_data(input_file, threshold)?;
    
    println!("Found {} records meeting criteria", processed_data.len());
    
    if !processed_data.is_empty() {
        export_results(&processed_data, output_file)?;
        println!("Results exported to {}", output_file);
    }
    
    Ok(())
}