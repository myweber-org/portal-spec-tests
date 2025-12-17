
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn read_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)?;
    
    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

pub fn write_csv_file<P: AsRef<Path>>(path: P, records: &[Record]) -> Result<(), Box<dyn Error>> {
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_path(path)?;
    
    for record in records {
        writer.serialize(record)?;
    }
    
    writer.flush()?;
    Ok(())
}

pub fn filter_records_by_category(records: &[Record], category: &str) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

pub fn calculate_average_value(records: &[Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    sum / records.len() as f64
}

pub fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    
    if record.value < 0.0 {
        return Err("Value cannot be negative".to_string());
    }
    
    if record.category.trim().is_empty() {
        return Err("Category cannot be empty".to_string());
    }
    
    Ok(())
}

pub fn transform_records(records: &[Record]) -> Vec<Record> {
    records
        .iter()
        .map(|r| {
            let mut transformed = r.clone();
            transformed.name = transformed.name.to_uppercase();
            transformed.value = (transformed.value * 100.0).round() / 100.0;
            transformed
        })
        .collect()
}