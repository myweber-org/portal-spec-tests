use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
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

    pub fn add_value(&mut self, key: String, value: f64) {
        self.values.insert(key, value);
    }

    pub fn add_tag(&mut self, tag: String) {
        self.tags.push(tag);
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationFailed("ID cannot be zero".to_string()));
        }

        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed("Timestamp cannot be negative".to_string()));
        }

        if self.values.is_empty() {
            return Err(DataError::ValidationFailed("Values cannot be empty".to_string()));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationFailed("Key cannot be empty".to_string()));
            }
            if !value.is_finite() {
                return Err(DataError::ValidationFailed(format!("Value for {} is not finite", key)));
            }
        }

        Ok(())
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records {
        record.validate()?;
        
        let mut processed_record = record.clone();
        
        processed_record.tags.retain(|tag| !tag.is_empty());
        processed_record.tags.sort();
        processed_record.tags.dedup();

        for value in processed_record.values.values_mut() {
            *value = (*value * 100.0).round() / 100.0;
        }

        processed.push(processed_record);
    }

    processed.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();

    if records.is_empty() {
        return stats;
    }

    for record in records {
        for (key, value) in &record.values {
            let entry = stats.entry(key.clone()).or_insert(Vec::new());
            entry.push(*value);
        }
    }

    let mut result = HashMap::new();
    for (key, values) in stats {
        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        result.insert(format!("{}_mean", key), mean);
        result.insert(format!("{}_variance", key), variance);
        result.insert(format!("{}_count", key), count);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature".to_string(), 25.5);
        
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, 1234567890);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let mut record1 = DataRecord::new(1, 1000);
        record1.add_value("pressure".to_string(), 1013.256);
        record1.add_tag("sensor".to_string());
        record1.add_tag("sensor".to_string());

        let mut record2 = DataRecord::new(2, 900);
        record2.add_value("pressure".to_string(), 1012.789);
        record2.add_tag("".to_string());

        let records = vec![record1, record2];
        let processed = process_records(records).unwrap();

        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].timestamp, 900);
        assert_eq!(processed[0].tags.len(), 0);
        assert_eq!(processed[1].tags.len(), 1);
    }
}