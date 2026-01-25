
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationFailed("ID cannot be zero".to_string()));
        }

        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        if self.values.is_empty() {
            return Err(DataError::ValidationFailed(
                "Values cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    pub fn add_value(&mut self, key: String, value: f64) {
        self.values.insert(key, value);
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }

        let sum: f64 = self.values.values().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> HashMap<String, f64> {
        self.values
            .iter()
            .filter(|(_, &value)| value >= threshold)
            .map(|(key, value)| (key.clone(), *value))
            .collect()
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::new();

    for record in records {
        record.validate()?;

        let mut processed_record = record.clone();

        if let Some(avg) = processed_record.calculate_average() {
            processed_record.add_value("average".to_string(), avg);
        }

        processed_record.add_tag("processed".to_string());
        processed.push(processed_record);
    }

    Ok(processed)
}

pub fn merge_records(records: &[DataRecord]) -> Option<DataRecord> {
    if records.is_empty() {
        return None;
    }

    let first_record = &records[0];
    let mut merged = DataRecord::new(first_record.id, first_record.timestamp);

    for record in records {
        for (key, value) in &record.values {
            merged.add_value(key.clone(), *value);
        }

        for tag in &record.tags {
            merged.add_tag(tag.clone());
        }
    }

    Some(merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature".to_string(), 25.5);
        assert!(record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_average_calculation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("a".to_string(), 10.0);
        record.add_value("b".to_string(), 20.0);
        record.add_value("c".to_string(), 30.0);

        assert_eq!(record.calculate_average(), Some(20.0));
    }

    #[test]
    fn test_filter_by_threshold() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("low".to_string(), 5.0);
        record.add_value("medium".to_string(), 15.0);
        record.add_value("high".to_string(), 25.0);

        let filtered = record.filter_by_threshold(10.0);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains_key("medium"));
        assert!(filtered.contains_key("high"));
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum DataError {
    InvalidFormat,
    OutOfRange,
    ConversionFailed,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidFormat => write!(f, "Data format is invalid"),
            DataError::OutOfRange => write!(f, "Value is out of acceptable range"),
            DataError::ConversionFailed => write!(f, "Failed to convert data type"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, DataError> {
        if threshold <= 0.0 || threshold > 100.0 {
            return Err(DataError::OutOfRange);
        }
        Ok(Self { threshold })
    }

    pub fn process_value(&self, raw_value: &str) -> Result<f64, DataError> {
        let parsed = raw_value.parse::<f64>().map_err(|_| DataError::InvalidFormat)?;
        
        if parsed < 0.0 {
            return Err(DataError::OutOfRange);
        }

        let processed = (parsed * 100.0) / self.threshold;
        
        if processed.is_nan() || processed.is_infinite() {
            return Err(DataError::ConversionFailed);
        }

        Ok(processed)
    }

    pub fn normalize_data(&self, values: &[f64]) -> Vec<f64> {
        if values.is_empty() {
            return Vec::new();
        }

        let max_value = values.iter().fold(f64::MIN, |a, &b| a.max(b));
        
        values.iter()
            .map(|&v| v / max_value)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(50.0);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_threshold() {
        let processor = DataProcessor::new(0.0);
        assert!(processor.is_err());
    }

    #[test]
    fn test_process_valid_value() {
        let processor = DataProcessor::new(50.0).unwrap();
        let result = processor.process_value("25.5");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 51.0);
    }

    #[test]
    fn test_normalize_data() {
        let processor = DataProcessor::new(10.0).unwrap();
        let data = vec![2.0, 4.0, 6.0, 8.0];
        let normalized = processor.normalize_data(&data);
        
        assert_eq!(normalized.len(), 4);
        assert_eq!(normalized[3], 1.0);
    }
}