
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    config: ProcessingConfig,
}

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub max_values: usize,
    pub min_timestamp: i64,
    pub max_timestamp: i64,
    pub require_metadata: bool,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            max_values: 100,
            min_timestamp: 0,
            max_timestamp: i64::MAX,
            require_metadata: false,
        }
    }
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.config.max_values {
            return Err(ProcessingError::ValidationError(format!(
                "Too many values: {} > {}",
                record.values.len(),
                self.config.max_values
            )));
        }

        if record.timestamp < self.config.min_timestamp {
            return Err(ProcessingError::ValidationError(format!(
                "Timestamp too early: {} < {}",
                record.timestamp, self.config.min_timestamp
            )));
        }

        if record.timestamp > self.config.max_timestamp {
            return Err(ProcessingError::ValidationError(format!(
                "Timestamp too late: {} > {}",
                record.timestamp, self.config.max_timestamp
            )));
        }

        if self.config.require_metadata && record.metadata.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Metadata required but missing".to_string(),
            ));
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::InvalidData(
                "Cannot normalize empty values array".to_string(),
            ));
        }

        let min_value = record
            .values
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let max_value = record
            .values
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        if (max_value - min_value).abs() < f64::EPSILON {
            return Err(ProcessingError::TransformationFailed(
                "All values are identical, cannot normalize".to_string(),
            ));
        }

        for value in &mut record.values {
            *value = (*value - min_value) / (max_value - min_value);
        }

        Ok(())
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;
        self.normalize_values(&mut record)?;
        Ok(record)
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, Vec<(usize, ProcessingError)>> {
        let mut processed = Vec::new();
        let mut errors = Vec::new();

        for (index, record) in records.into_iter().enumerate() {
            match self.process_record(record) {
                Ok(processed_record) => processed.push(processed_record),
                Err(err) => errors.push((index, err)),
            }
        }

        if errors.is_empty() {
            Ok(processed)
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let config = ProcessingConfig::default();
        let processor = DataProcessor::new(config);

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_normalization() {
        let config = ProcessingConfig::default();
        let processor = DataProcessor::new(config);

        let mut record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.normalize_values(&mut record).is_ok());
        assert_eq!(record.values, vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn test_batch_processing() {
        let config = ProcessingConfig {
            max_values: 5,
            ..Default::default()
        };
        let processor = DataProcessor::new(config);

        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: vec![1.0, 2.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                metadata: HashMap::new(),
            },
        ];

        let result = processor.batch_process(records);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, 1);
    }
}