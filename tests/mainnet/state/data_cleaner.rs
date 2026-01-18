use csv::ReaderBuilder;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.value < 0.0 {
        return Err("Value must be non-negative".to_string());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Category must be A, B, or C".to_string());
    }
    Ok(())
}

pub fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input_file);
    
    let mut valid_records = Vec::new();
    let mut error_count = 0;

    for result in rdr.deserialize() {
        let record: Record = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Parsing error: {}", e);
                error_count += 1;
                continue;
            }
        };

        match validate_record(&record) {
            Ok(_) => valid_records.push(record),
            Err(e) => {
                eprintln!("Validation error for ID {}: {}", record.id, e);
                error_count += 1;
            }
        }
    }

    println!("Processed {} records", valid_records.len() + error_count);
    println!("Valid records: {}", valid_records.len());
    println!("Invalid records: {}", error_count);

    if !valid_records.is_empty() {
        let output_file = File::create(output_path)?;
        let mut wtr = csv::Writer::from_writer(output_file);
        
        for record in valid_records {
            wtr.serialize(record)?;
        }
        wtr.flush()?;
        println!("Cleaned data written to {}", output_path);
    }

    Ok(())
}