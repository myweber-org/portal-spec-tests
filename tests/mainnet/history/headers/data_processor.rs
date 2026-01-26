
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessedData {
    pub id: u32,
    pub value: f64,
    pub is_valid: bool,
    pub metadata: String,
}

#[derive(Debug)]
pub enum DataError {
    InvalidValue(f64),
    EmptyMetadata,
    IdOutOfRange(u32),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidValue(val) => write!(f, "Value {} is outside acceptable range", val),
            DataError::EmptyMetadata => write!(f, "Metadata cannot be empty"),
            DataError::IdOutOfRange(id) => write!(f, "ID {} exceeds maximum allowed value", id),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    max_id: u32,
    min_value: f64,
    max_value: f64,
}

impl DataProcessor {
    pub fn new(max_id: u32, min_value: f64, max_value: f64) -> Self {
        DataProcessor {
            max_id,
            min_value,
            max_value,
        }
    }

    pub fn validate_id(&self, id: u32) -> Result<(), DataError> {
        if id > self.max_id {
            Err(DataError::IdOutOfRange(id))
        } else {
            Ok(())
        }
    }

    pub fn validate_value(&self, value: f64) -> Result<(), DataError> {
        if value < self.min_value || value > self.max_value {
            Err(DataError::InvalidValue(value))
        } else {
            Ok(())
        }
    }

    pub fn validate_metadata(&self, metadata: &str) -> Result<(), DataError> {
        if metadata.trim().is_empty() {
            Err(DataError::EmptyMetadata)
        } else {
            Ok(())
        }
    }

    pub fn process_data(
        &self,
        id: u32,
        value: f64,
        metadata: &str,
    ) -> Result<ProcessedData, DataError> {
        self.validate_id(id)?;
        self.validate_value(value)?;
        self.validate_metadata(metadata)?;

        let normalized_value = (value - self.min_value) / (self.max_value - self.min_value);
        let is_valid = normalized_value >= 0.5;

        Ok(ProcessedData {
            id,
            value: normalized_value,
            is_valid,
            metadata: metadata.trim().to_string(),
        })
    }

    pub fn batch_process(
        &self,
        items: Vec<(u32, f64, String)>,
    ) -> (Vec<ProcessedData>, Vec<DataError>) {
        let mut processed = Vec::new();
        let mut errors = Vec::new();

        for (id, value, metadata) in items {
            match self.process_data(id, value, &metadata) {
                Ok(data) => processed.push(data),
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
        let processor = DataProcessor::new(1000, 0.0, 100.0);
        let result = processor.process_data(42, 75.5, "sample data");

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.id, 42);
        assert!(data.value > 0.7);
        assert!(data.is_valid);
        assert_eq!(data.metadata, "sample data");
    }

    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new(100, 0.0, 100.0);
        let result = processor.process_data(150, 50.0, "test");

        assert!(result.is_err());
        match result.unwrap_err() {
            DataError::IdOutOfRange(id) => assert_eq!(id, 150),
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(500, 0.0, 200.0);
        let items = vec![
            (1, 150.0, "item1".to_string()),
            (600, 50.0, "item2".to_string()),
            (2, 250.0, "item3".to_string()),
            (3, 100.0, "".to_string()),
        ];

        let (processed, errors) = processor.batch_process(items);

        assert_eq!(processed.len(), 1);
        assert_eq!(errors.len(), 3);
    }
}