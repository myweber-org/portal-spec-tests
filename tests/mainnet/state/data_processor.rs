
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    MissingName,
    ValidationFailed(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid numeric value"),
            DataError::MissingName => write!(f, "Record name is required"),
            DataError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if name.trim().is_empty() {
            return Err(DataError::MissingName);
        }
        if !value.is_finite() {
            return Err(DataError::InvalidValue);
        }

        Ok(Self {
            id,
            name,
            value,
            metadata: HashMap::new(),
        })
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn transform_value<F>(&mut self, transformer: F) -> Result<(), DataError>
    where
        F: Fn(f64) -> f64,
    {
        let new_value = transformer(self.value);
        if !new_value.is_finite() {
            return Err(DataError::InvalidValue);
        }
        self.value = new_value;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidId);
        }
        if self.name.trim().is_empty() {
            return Err(DataError::MissingName);
        }
        if !self.value.is_finite() {
            return Err(DataError::InvalidValue);
        }
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        
        record.transform_value(|v| v * 1.1)?;
        
        if record.value > 1000.0 {
            record.add_metadata("category".to_string(), "high_value".to_string());
        } else {
            record.add_metadata("category".to_string(), "normal".to_string());
        }
        
        processed.push(record.clone());
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, "Test".to_string(), 42.5).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test");
        assert_eq!(record.value, 42.5);
    }

    #[test]
    fn test_invalid_id() {
        let result = DataRecord::new(0, "Test".to_string(), 42.5);
        assert!(matches!(result, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_transform_value() {
        let mut record = DataRecord::new(1, "Test".to_string(), 100.0).unwrap();
        record.transform_value(|v| v * 2.0).unwrap();
        assert_eq!(record.value, 200.0);
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord::new(1, "Record1".to_string(), 500.0).unwrap(),
            DataRecord::new(2, "Record2".to_string(), 1500.0).unwrap(),
        ];
        
        let processed = process_records(&mut records).unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].value, 550.0);
        assert_eq!(processed[1].value, 1650.0);
        assert_eq!(processed[1].get_metadata("category").unwrap(), "high_value");
    }
}use csv::Reader;
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

pub fn process_data_file(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 && !record.name.is_empty() {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let avg = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let max = records.iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, f64::max);
    
    (avg, max, count)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}