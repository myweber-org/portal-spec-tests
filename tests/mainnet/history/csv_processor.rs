use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut rdr = Reader::from_reader(input_file);
    
    let output_file = File::create(Path::new(output_path))?;
    let mut wtr = Writer::from_writer(output_file);
    
    for result in rdr.deserialize() {
        let record: Record = result?;
        
        match record.validate() {
            Ok(_) => {
                wtr.serialize(&record)?;
                println!("Processed record: {:?}", record);
            }
            Err(e) => {
                eprintln!("Validation failed for record {:?}: {}", record, e);
            }
        }
    }
    
    wtr.flush()?;
    Ok(())
}

fn main() {
    let input = "data/input.csv";
    let output = "data/output.csv";
    
    match process_csv(input, output) {
        Ok(_) => println!("CSV processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }
}