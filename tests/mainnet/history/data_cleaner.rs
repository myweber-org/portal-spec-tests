use csv::{Reader, Writer};
use std::error::Error;
use std::fs::File;
use std::io;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    let headers = reader.headers()?.clone();
    writer.write_record(&headers)?;

    for result in reader.records() {
        let record = result?;
        let filtered_record: Vec<&str> = record
            .iter()
            .filter(|field| !field.trim().is_empty())
            .collect();

        if filtered_record.len() == headers.len() {
            writer.write_record(&filtered_record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

pub fn clean_csv_from_stdin() -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_reader(io::stdin());
    let mut writer = Writer::from_writer(io::stdout());

    let headers = reader.headers()?.clone();
    writer.write_record(&headers)?;

    for result in reader.records() {
        let record = result?;
        let filtered_record: Vec<&str> = record
            .iter()
            .filter(|field| !field.trim().is_empty())
            .collect();

        if filtered_record.len() == headers.len() {
            writer.write_record(&filtered_record)?;
        }
    }

    writer.flush()?;
    Ok(())
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = Reader::from_reader(reader);
    
    let output_file = File::create(output_path)?;
    let mut csv_writer = Writer::from_writer(output_file);
    
    let mut cleaned_count = 0;
    let mut skipped_count = 0;
    
    for result in csv_reader.deserialize() {
        let record: Record = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Skipping invalid record: {}", e);
                skipped_count += 1;
                continue;
            }
        };
        
        let cleaned_record = clean_record(record);
        csv_writer.serialize(&cleaned_record)?;
        cleaned_count += 1;
    }
    
    csv_writer.flush()?;
    
    println!("Cleaning completed:");
    println!("  Records cleaned: {}", cleaned_count);
    println!("  Records skipped: {}", skipped_count);
    
    Ok(())
}

fn clean_record(mut record: Record) -> Record {
    record.name = record.name.trim().to_string();
    if record.name.is_empty() {
        record.name = "Unknown".to_string();
    }
    
    record.value = record.value.abs();
    
    record.category = record.category.to_lowercase();
    
    record
}

fn validate_file_path(path: &str) -> bool {
    let path_obj = Path::new(path);
    path_obj.exists() && path_obj.is_file()
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/raw_data.csv";
    let output_file = "data/cleaned_data.csv";
    
    if !validate_file_path(input_file) {
        eprintln!("Input file does not exist: {}", input_file);
        return Ok(());
    }
    
    match clean_csv(input_file, output_file) {
        Ok(_) => println!("Data cleaning successful!"),
        Err(e) => eprintln!("Error during cleaning: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_clean_record() {
        let record = Record {
            id: 1,
            name: "  Test Name  ".to_string(),
            value: -42.5,
            category: "CATEGORY".to_string(),
        };
        
        let cleaned = clean_record(record);
        
        assert_eq!(cleaned.name, "Test Name");
        assert_eq!(cleaned.value, 42.5);
        assert_eq!(cleaned.category, "category");
    }
    
    #[test]
    fn test_validate_file_path() {
        let temp_file = NamedTempFile::new().unwrap();
        let valid_path = temp_file.path().to_str().unwrap();
        
        assert!(validate_file_path(valid_path));
        assert!(!validate_file_path("non_existent_file.csv"));
    }
}