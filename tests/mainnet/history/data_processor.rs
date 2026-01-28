
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

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

pub fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);
    
    let mut valid_count = 0;
    let mut invalid_count = 0;
    
    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.is_valid() {
            writer.serialize(&record)?;
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
    }
    
    writer.flush()?;
    
    println!("Processing complete:");
    println!("  Valid records: {}", valid_count);
    println!("  Invalid records: {}", invalid_count);
    
    Ok(())
}

pub fn generate_sample_data() -> Result<(), Box<dyn Error>> {
    let records = vec![
        Record { id: 1, name: "Item A".to_string(), value: 100.5, active: true },
        Record { id: 2, name: "Item B".to_string(), value: 250.0, active: false },
        Record { id: 3, name: "".to_string(), value: -50.0, active: true },
        Record { id: 4, name: "Item D".to_string(), value: 75.25, active: true },
    ];
    
    let file = File::create("sample_data.csv")?;
    let mut writer = Writer::from_writer(file);
    
    for record in records {
        writer.serialize(&record)?;
    }
    
    writer.flush()?;
    println!("Sample data generated: sample_data.csv");
    
    Ok(())
}