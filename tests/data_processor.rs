
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
}

fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

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

    println!("Processing complete:");
    println!("  Valid records: {}", valid_count);
    println!("  Invalid records: {}", invalid_count);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "input_data.csv";
    let output_file = "processed_data.csv";
    
    process_csv(input_file, output_file)?;
    
    Ok(())
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidValue,
    InvalidTimestamp,
    ValidationFailed(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidValue => write!(f, "Invalid numeric value"),
            DataError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            DataError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<Self, DataError> {
        if value.is_nan() || value.is_infinite() {
            return Err(DataError::InvalidValue);
        }
        
        if timestamp < 0 {
            return Err(DataError::InvalidTimestamp);
        }
        
        Ok(Self {
            id,
            value,
            timestamp,
        })
    }
    
    pub fn validate(&self) -> Result<(), DataError> {
        if self.value < 0.0 || self.value > 1000.0 {
            return Err(DataError::ValidationFailed(
                format!("Value {} out of range [0, 1000]", self.value)
            ));
        }
        
        let current_time = chrono::Utc::now().timestamp();
        if self.timestamp > current_time + 3600 {
            return Err(DataError::ValidationFailed(
                "Timestamp is more than 1 hour in the future".to_string()
            ));
        }
        
        Ok(())
    }
    
    pub fn transform(&self, multiplier: f64) -> Result<Self, DataError> {
        if multiplier <= 0.0 {
            return Err(DataError::InvalidValue);
        }
        
        let transformed_value = self.value * multiplier;
        Self::new(self.id, transformed_value, self.timestamp)
    }
}

pub fn process_records(
    records: Vec<DataRecord>,
    multiplier: f64,
) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        let transformed = record.transform(multiplier)?;
        processed.push(transformed);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, 1672531200);
        assert!(record.is_ok());
        
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.timestamp, 1672531200);
    }
    
    #[test]
    fn test_invalid_value() {
        let record = DataRecord::new(1, f64::NAN, 1672531200);
        assert!(matches!(record, Err(DataError::InvalidValue)));
    }
    
    #[test]
    fn test_validation_success() {
        let record = DataRecord::new(1, 500.0, 1672531200).unwrap();
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_transform_record() {
        let record = DataRecord::new(1, 100.0, 1672531200).unwrap();
        let transformed = record.transform(2.0).unwrap();
        assert_eq!(transformed.value, 200.0);
    }
}