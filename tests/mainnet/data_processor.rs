
use csv::Reader;
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

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    
    let mut records = Vec::new();
    let mut validation_errors = Vec::new();
    
    for (index, result) in reader.deserialize().enumerate() {
        match result {
            Ok(record) => {
                let rec: Record = record;
                if validate_record(&rec) {
                    records.push(rec);
                } else {
                    validation_errors.push(format!("Invalid record at line {}", index + 1));
                }
            }
            Err(e) => {
                validation_errors.push(format!("Parse error at line {}: {}", index + 1, e));
            }
        }
    }
    
    if !validation_errors.is_empty() {
        eprintln!("Validation warnings: {:?}", validation_errors);
    }
    
    Ok(records)
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && 
    record.value >= 0.0 && 
    !record.category.is_empty() &&
    record.id > 0
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}