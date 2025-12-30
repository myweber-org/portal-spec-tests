
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Transformation failed")]
    TransformationFailed,
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
}

pub struct DataProcessor {
    validation_threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor {
            validation_threshold: threshold,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(ProcessingError::InvalidInput);
        }

        if record.value.abs() > self.validation_threshold {
            return Err(ProcessingError::ValidationError(
                format!("Value {} exceeds threshold {}", record.value, self.validation_threshold)
            ));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::ValidationError(
                "Negative timestamp not allowed".to_string()
            ));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(record)?;

        let transformed_value = if record.value >= 0.0 {
            record.value.ln()
        } else {
            -record.value.abs().ln()
        };

        if transformed_value.is_nan() || transformed_value.is_infinite() {
            return Err(ProcessingError::TransformationFailed);
        }

        Ok(DataRecord {
            id: record.id,
            value: transformed_value,
            timestamp: record.timestamp,
        })
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records
            .into_iter()
            .map(|record| self.transform_record(&record))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let processor = DataProcessor::new(1000.0);
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1234567890,
        };

        let result = processor.transform_record(&record);
        assert!(result.is_ok());
        
        let transformed = result.unwrap();
        assert_eq!(transformed.id, 1);
        assert!(transformed.value - 3.749504 < 0.001);
    }

    #[test]
    fn test_invalid_record_rejection() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 2,
            value: 150.0,
            timestamp: 1234567890,
        };

        let result = processor.transform_record(&record);
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(1000.0);
        let records = vec![
            DataRecord { id: 1, value: 10.0, timestamp: 1000 },
            DataRecord { id: 2, value: f64::NAN, timestamp: 2000 },
            DataRecord { id: 3, value: -5.0, timestamp: 3000 },
        ];

        let results = processor.process_batch(records);
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        assert!(results[2].is_ok());
    }
}