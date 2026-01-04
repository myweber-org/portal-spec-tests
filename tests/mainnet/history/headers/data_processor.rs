
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(data)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        for &value in data {
            if !value.is_finite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }
        Ok(data.to_vec())
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max - min).abs() < f64::EPSILON {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&x| (x - min) / (max - min))
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.ln_1p().exp() - 1.0)
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_dataset("test", &data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 5);
        assert!(processor.cache_size() > 0);
    }

    #[test]
    fn test_empty_data() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("empty", &[]);
        assert!(result.is_err());
    }
}
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data value: {0}")]
    InvalidValue(f64),
    #[error("Timestamp out of range: {0}")]
    InvalidTimestamp(i64),
    #[error("Data validation failed")]
    ValidationFailed,
}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
    time_window: (i64, i64),
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64, start_time: i64, end_time: i64) -> Self {
        Self {
            min_value,
            max_value,
            time_window: (start_time, end_time),
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < self.min_value || record.value > self.max_value {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if record.timestamp < self.time_window.0 || record.timestamp > self.time_window.1 {
            return Err(ProcessingError::InvalidTimestamp(record.timestamp));
        }

        Ok(())
    }

    pub fn normalize_value(&self, value: f64) -> f64 {
        (value - self.min_value) / (self.max_value - self.min_value)
    }

    pub fn process_records(&self, records: Vec<DataRecord>) -> Result<Vec<f64>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());

        for record in records {
            self.validate_record(&record)?;
            let normalized = self.normalize_value(record.value);
            results.push(normalized);
        }

        Ok(results)
    }

    pub fn filter_by_time_range(&self, records: Vec<DataRecord>) -> Vec<DataRecord> {
        records
            .into_iter()
            .filter(|r| r.timestamp >= self.time_window.0 && r.timestamp <= self.time_window.1)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(0.0, 100.0, 0, 1000);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 500,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(0.0, 100.0, 0, 1000);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 500,
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(0.0, 100.0, 0, 1000);
        assert_eq!(processor.normalize_value(50.0), 0.5);
        assert_eq!(processor.normalize_value(0.0), 0.0);
        assert_eq!(processor.normalize_value(100.0), 1.0);
    }

    #[test]
    fn test_process_records() {
        let processor = DataProcessor::new(0.0, 100.0, 0, 1000);
        let records = vec![
            DataRecord {
                id: 1,
                value: 25.0,
                timestamp: 100,
            },
            DataRecord {
                id: 2,
                value: 75.0,
                timestamp: 200,
            },
        ];

        let result = processor.process_records(records).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], 0.25);
        assert_eq!(result[1], 0.75);
    }
}