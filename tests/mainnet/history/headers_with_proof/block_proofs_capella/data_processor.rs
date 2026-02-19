
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
            return Err(DataError::ValidationFailed(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        if self.values.is_empty() {
            return Err(DataError::ValidationFailed(
                "Record must contain at least one value".to_string(),
            ));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationFailed(
                    "Value key cannot be empty".to_string(),
                ));
            }
            if !value.is_finite() {
                return Err(DataError::ValidationFailed(format!(
                    "Value for key '{}' must be finite",
                    key
                )));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&mut self) {
        let sum: f64 = self.values.values().sum();
        if sum != 0.0 {
            for value in self.values.values_mut() {
                *value /= sum;
            }
        }
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }

    pub fn process_records(&mut self) {
        for record in &mut self.records {
            record.normalize_values();
        }
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn calculate_statistics(&self) -> HashMap<String, (f64, f64, f64)> {
        let mut stats = HashMap::new();

        for record in &self.records {
            for (key, value) in &record.values {
                let entry = stats.entry(key.clone()).or_insert((0.0, f64::MAX, f64::MIN));
                entry.0 += value;
                entry.1 = entry.1.min(*value);
                entry.2 = entry.2.max(*value);
            }
        }

        for (_, (sum, min, max)) in stats.iter_mut() {
            *sum /= self.records.len() as f64;
        }

        stats
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
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
    fn test_value_normalization() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("a".to_string(), 1.0);
        record.add_value("b".to_string(), 2.0);
        record.add_value("c".to_string(), 3.0);

        record.normalize_values();

        let sum: f64 = record.values.values().sum();
        assert!((sum - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();

        let mut record1 = DataRecord::new(1, 1234567890);
        record1.add_value("value".to_string(), 10.0);
        record1.add_tag("test".to_string());

        let mut record2 = DataRecord::new(2, 1234567891);
        record2.add_value("value".to_string(), 20.0);
        record2.add_tag("test".to_string());

        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        assert_eq!(processor.record_count(), 2);

        let filtered = processor.filter_by_tag("test");
        assert_eq!(filtered.len(), 2);

        processor.process_records();
        let stats = processor.calculate_statistics();
        assert!(stats.contains_key("value"));

        processor.clear();
        assert_eq!(processor.record_count(), 0);
    }
}