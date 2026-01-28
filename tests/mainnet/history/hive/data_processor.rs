
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Transformation failed: {0}")]
    TransformationFailed(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
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
            return Err(DataError::InvalidInput(
                "Value must be finite number".to_string(),
            ));
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
        if self.values.is_empty() {
            return Err(DataError::ValidationError(
                "Record must contain at least one value".to_string(),
            ));
        }

        if self.timestamp < 0 {
            return Err(DataError::ValidationError(
                "Timestamp must be non-negative".to_string(),
            ));
        }

        Ok(())
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    transformations: Vec<Box<dyn Fn(&DataRecord) -> Result<DataRecord, DataError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            transformations: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(&DataRecord) -> Result<DataRecord, DataError> + 'static,
    {
        self.transformations.push(Box::new(transform));
    }

    pub fn process(&self) -> Result<Vec<DataRecord>, DataError> {
        let mut processed = Vec::with_capacity(self.records.len());

        for record in &self.records {
            let mut current = record.clone();

            for transform in &self.transformations {
                current = transform(&current)?;
            }

            processed.push(current);
        }

        Ok(processed)
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if self.records.is_empty() {
            return stats;
        }

        let mut value_sums: HashMap<String, f64> = HashMap::new();
        let mut value_counts: HashMap<String, usize> = HashMap::new();

        for record in &self.records {
            for (key, value) in &record.values {
                *value_sums.entry(key.clone()).or_insert(0.0) += value;
                *value_counts.entry(key.clone()).or_insert(0) += 1;
            }
        }

        for (key, sum) in value_sums {
            if let Some(&count) = value_counts.get(&key) {
                if count > 0 {
                    stats.insert(format!("{}_average", key), sum / count as f64);
                }
            }
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        assert!(record.validate().is_err());

        record.add_value("temperature", 25.5).unwrap();
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_value() {
        let mut record = DataRecord::new(1, 1234567890);
        let result = record.add_value("invalid", f64::INFINITY);
        assert!(matches!(result, Err(DataError::InvalidInput(_))));
    }

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("pressure", 1013.25).unwrap();
        processor.add_record(record).unwrap();

        processor.add_transformation(|r| {
            let mut transformed = r.clone();
            for (key, value) in &mut transformed.values {
                *value *= 1.1;
            }
            Ok(transformed)
        });

        let processed = processor.process().unwrap();
        assert_eq!(processed[0].values["pressure"], 1114.575);
    }
}