use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data value: {0}")]
    InvalidValue(String),
    #[error("Timestamp out of range")]
    InvalidTimestamp,
    #[error("Data validation failed")]
    ValidationFailed,
}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64) -> Self {
        DataProcessor { min_value, max_value }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < self.min_value || record.value > self.max_value {
            return Err(ProcessingError::InvalidValue(
                format!("Value {} outside range [{}, {}]", record.value, self.min_value, self.max_value)
            ));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }

        Ok(())
    }

    pub fn normalize_value(&self, record: &DataRecord) -> f64 {
        (record.value - self.min_value) / (self.max_value - self.min_value)
    }

    pub fn process_records(&self, records: Vec<DataRecord>) -> Result<Vec<f64>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            let normalized = self.normalize_value(&record);
            results.push(normalized);
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1234567890,
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1234567890,
        };
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 75.0,
            timestamp: 1234567890,
        };
        
        assert_eq!(processor.normalize_value(&record), 0.75);
    }
}