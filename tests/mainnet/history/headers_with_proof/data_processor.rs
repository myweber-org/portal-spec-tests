
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    OutOfRange(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn add_value(&mut self, key: &str, value: f64) -> Result<(), DataError> {
        if !value.is_finite() {
            return Err(DataError::InvalidFormat);
        }
        self.values.insert(key.to_string(), value);
        Ok(())
    }

    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidFormat);
        }

        if self.timestamp < 0 {
            return Err(DataError::OutOfRange("timestamp".to_string()));
        }

        if self.values.is_empty() {
            return Err(DataError::MissingField("values".to_string()));
        }

        Ok(())
    }

    pub fn transform(&self, multiplier: f64) -> HashMap<String, f64> {
        self.values
            .iter()
            .map(|(k, v)| (k.clone(), v * multiplier))
            .collect()
    }
}

pub fn process_records(records: &[DataRecord]) -> Result<Vec<HashMap<String, f64>>, DataError> {
    let mut results = Vec::new();

    for record in records {
        record.validate()?;
        let transformed = record.transform(2.0);
        results.push(transformed);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let mut record = DataRecord::new(1, 1625097600);
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1625097600);
    }

    #[test]
    fn test_add_value() {
        let mut record = DataRecord::new(1, 1625097600);
        assert!(record.add_value("temperature", 25.5).is_ok());
        assert_eq!(record.values.get("temperature"), Some(&25.5));
    }

    #[test]
    fn test_validate_record() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("pressure", 1013.25).unwrap();
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_transform_values() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("value", 10.0).unwrap();
        let transformed = record.transform(3.0);
        assert_eq!(transformed.get("value"), Some(&30.0));
    }
}