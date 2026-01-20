
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error")]
    TransformationError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawData {
    pub id: u64,
    pub value: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedData {
    pub id: u64,
    pub normalized_value: f64,
    pub is_valid: bool,
    pub processed_at: i64,
}

pub struct DataProcessor {
    validation_threshold: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64) -> Self {
        Self {
            validation_threshold,
        }
    }

    pub fn validate_raw_data(&self, data: &RawData) -> Result<(), DataError> {
        if data.value.is_empty() {
            return Err(DataError::ValidationFailed("Empty value".to_string()));
        }

        if data.timestamp <= 0 {
            return Err(DataError::ValidationFailed("Invalid timestamp".to_string()));
        }

        Ok(())
    }

    pub fn parse_numeric_value(value: &str) -> Result<f64, DataError> {
        value
            .parse::<f64>()
            .map_err(|_| DataError::InvalidFormat)
    }

    pub fn process_data(&self, raw_data: RawData) -> Result<ProcessedData, DataError> {
        self.validate_raw_data(&raw_data)?;
        
        let numeric_value = Self::parse_numeric_value(&raw_data.value)?;
        
        let normalized_value = if numeric_value.abs() > self.validation_threshold {
            numeric_value / self.validation_threshold
        } else {
            numeric_value
        };

        let is_valid = normalized_value >= 0.0 && normalized_value <= 1.0;

        Ok(ProcessedData {
            id: raw_data.id,
            normalized_value,
            is_valid,
            processed_at: chrono::Utc::now().timestamp(),
        })
    }

    pub fn batch_process(
        &self,
        data_vec: Vec<RawData>,
    ) -> (Vec<ProcessedData>, Vec<DataError>) {
        let mut processed = Vec::new();
        let mut errors = Vec::new();

        for data in data_vec {
            match self.process_data(data) {
                Ok(processed_data) => processed.push(processed_data),
                Err(err) => errors.push(err),
            }
        }

        (processed, errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_data_processing() {
        let processor = DataProcessor::new(100.0);
        let raw_data = RawData {
            id: 1,
            value: "42.5".to_string(),
            timestamp: 1234567890,
        };

        let result = processor.process_data(raw_data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.id, 1);
        assert!(processed.normalized_value > 0.0);
        assert!(processed.processed_at > 0);
    }

    #[test]
    fn test_invalid_data_format() {
        let processor = DataProcessor::new(100.0);
        let raw_data = RawData {
            id: 2,
            value: "not_a_number".to_string(),
            timestamp: 1234567890,
        };

        let result = processor.process_data(raw_data);
        assert!(result.is_err());
        
        if let Err(DataError::InvalidFormat) = result {
            // Expected error
        } else {
            panic!("Expected InvalidFormat error");
        }
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(100.0);
        
        let test_data = vec![
            RawData {
                id: 1,
                value: "50.0".to_string(),
                timestamp: 1234567890,
            },
            RawData {
                id: 2,
                value: "invalid".to_string(),
                timestamp: 1234567890,
            },
            RawData {
                id: 3,
                value: "150.0".to_string(),
                timestamp: 1234567890,
            },
        ];

        let (processed, errors) = processor.batch_process(test_data);
        
        assert_eq!(processed.len(), 2);
        assert_eq!(errors.len(), 1);
        
        for data in &processed {
            assert!(data.id == 1 || data.id == 3);
        }
    }
}