
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
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.values.is_empty() {
            return Err(DataError::ValidationFailed(
                "Record must contain at least one value".to_string(),
            ));
        }

        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationFailed(
                    "Value key cannot be empty".to_string(),
                ));
            }
            if value.is_nan() || value.is_infinite() {
                return Err(DataError::ValidationFailed(format!(
                    "Invalid value for key '{}': {}",
                    key, value
                )));
            }
        }

        Ok(())
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    statistics: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            statistics: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        record.validate()?;
        self.records.push(record);
        self.update_statistics();
        Ok(())
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn get_average(&self, key: &str) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;

        for record in &self.records {
            if let Some(value) = record.values.get(key) {
                sum += value;
                count += 1;
            }
        }

        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }

    fn update_statistics(&mut self) {
        self.statistics.clear();
        let count = self.records.len() as f64;

        if count == 0.0 {
            return;
        }

        let mut value_keys = Vec::new();
        for record in &self.records {
            for key in record.values.keys() {
                if !value_keys.contains(key) {
                    value_keys.push(key.clone());
                }
            }
        }

        for key in value_keys {
            if let Some(avg) = self.get_average(&key) {
                self.statistics.insert(format!("avg_{}", key), avg);
            }
        }

        self.statistics.insert("total_records".to_string(), count);
    }

    pub fn get_statistics(&self) -> &HashMap<String, f64> {
        &self.statistics
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.statistics.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature".to_string(), 25.5);
        record.add_tag("sensor".to_string());

        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_empty_record_validation() {
        let record = DataRecord::new(1, 1234567890);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();

        let mut record1 = DataRecord::new(1, 1000);
        record1.add_value("pressure".to_string(), 1013.25);
        record1.add_tag("weather".to_string());

        let mut record2 = DataRecord::new(2, 2000);
        record2.add_value("pressure".to_string(), 1012.50);
        record2.add_tag("weather".to_string());

        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());

        let weather_records = processor.filter_by_tag("weather");
        assert_eq!(weather_records.len(), 2);

        let avg_pressure = processor.get_average("pressure");
        assert!(avg_pressure.is_some());
        assert!((avg_pressure.unwrap() - 1012.875).abs() < 0.001);
    }
}