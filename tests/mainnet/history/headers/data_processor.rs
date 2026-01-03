
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
    MissingMetadata,
    ProcessingFailed(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidValue(v) => write!(f, "Invalid value encountered: {}", v),
            DataError::MissingMetadata => write!(f, "Required metadata is missing"),
            DataError::ProcessingFailed(msg) => write!(f, "Processing failed: {}", msg),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    threshold: f64,
    max_retries: u8,
}

impl DataProcessor {
    pub fn new(threshold: f64, max_retries: u8) -> Self {
        DataProcessor {
            threshold,
            max_retries,
        }
    }

    pub fn validate_value(&self, value: f64) -> Result<f64, DataError> {
        if value.is_nan() || value.is_infinite() {
            return Err(DataError::InvalidValue(value));
        }
        
        if value.abs() > self.threshold {
            return Err(DataError::InvalidValue(value));
        }
        
        Ok(value)
    }

    pub fn process_data(&self, id: u32, raw_value: f64, metadata: Option<&str>) -> Result<ProcessedData, DataError> {
        let validated_value = self.validate_value(raw_value)?;
        
        let metadata_str = metadata
            .ok_or(DataError::MissingMetadata)?
            .to_string();

        if metadata_str.is_empty() {
            return Err(DataError::MissingMetadata);
        }

        let is_valid = validated_value >= 0.0;
        let processed_value = if is_valid {
            validated_value * 2.0
        } else {
            validated_value.abs()
        };

        Ok(ProcessedData {
            id,
            value: processed_value,
            is_valid,
            metadata: metadata_str,
        })
    }

    pub fn batch_process(
        &self,
        items: Vec<(u32, f64, Option<&str>)>,
    ) -> (Vec<ProcessedData>, Vec<DataError>) {
        let mut successes = Vec::new();
        let mut errors = Vec::new();

        for (id, value, metadata) in items {
            match self.process_data(id, value, metadata) {
                Ok(processed) => successes.push(processed),
                Err(err) => errors.push(err),
            }
        }

        (successes, errors)
    }

    pub fn retry_processing<F>(&self, mut operation: F) -> Result<ProcessedData, DataError>
    where
        F: FnMut() -> Result<ProcessedData, DataError>,
    {
        let mut last_error = None;
        
        for attempt in 1..=self.max_retries {
            match operation() {
                Ok(result) => return Ok(result),
                Err(err) => {
                    last_error = Some(err);
                    if attempt < self.max_retries {
                        continue;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| DataError::ProcessingFailed("Max retries exceeded".to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_value_within_threshold() {
        let processor = DataProcessor::new(100.0, 3);
        assert_eq!(processor.validate_value(50.0), Ok(50.0));
    }

    #[test]
    fn test_validate_value_exceeds_threshold() {
        let processor = DataProcessor::new(100.0, 3);
        assert!(processor.validate_value(150.0).is_err());
    }

    #[test]
    fn test_process_data_valid() {
        let processor = DataProcessor::new(100.0, 3);
        let result = processor.process_data(1, 25.0, Some("test_metadata"));
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.id, 1);
        assert_eq!(data.value, 50.0);
        assert!(data.is_valid);
        assert_eq!(data.metadata, "test_metadata");
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(100.0, 3);
        let items = vec![
            (1, 10.0, Some("meta1")),
            (2, -5.0, Some("meta2")),
            (3, 150.0, Some("meta3")),
        ];
        
        let (successes, errors) = processor.batch_process(items);
        
        assert_eq!(successes.len(), 2);
        assert_eq!(errors.len(), 1);
    }
}