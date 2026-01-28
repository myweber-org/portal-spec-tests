
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
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
    cache: HashMap<u32, DataRecord>,
}

#[derive(Clone)]
pub struct ProcessingConfig {
    pub max_value: f64,
    pub min_value: f64,
    pub allowed_tags: Vec<String>,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor {
            config,
            cache: HashMap::new(),
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value > self.config.max_value {
            return Err(ProcessingError::ValidationError(
                format!("Value {} exceeds maximum {}", record.value, self.config.max_value)
            ));
        }

        if record.value < self.config.min_value {
            return Err(ProcessingError::ValidationError(
                format!("Value {} below minimum {}", record.value, self.config.min_value)
            ));
        }

        for tag in &record.tags {
            if !self.config.allowed_tags.contains(tag) {
                return Err(ProcessingError::ValidationError(
                    format!("Tag '{}' is not allowed", tag)
                ));
            }
        }

        Ok(())
    }

    pub fn process_record(&mut self, record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;

        let transformed_value = self.transform_value(record.value)?;
        let normalized_name = self.normalize_name(&record.name);

        let processed_record = DataRecord {
            value: transformed_value,
            name: normalized_name,
            ..record
        };

        self.cache.insert(processed_record.id, processed_record.clone());
        Ok(processed_record)
    }

    fn transform_value(&self, value: f64) -> Result<f64, ProcessingError> {
        if value.is_nan() || value.is_infinite() {
            return Err(ProcessingError::TransformationFailed(
                "Cannot transform NaN or infinite values".to_string()
            ));
        }

        let transformed = (value * 100.0).round() / 100.0;
        Ok(transformed)
    }

    fn normalize_name(&self, name: &str) -> String {
        name.trim().to_lowercase()
    }

    pub fn get_cached_record(&self, id: u32) -> Option<&DataRecord> {
        self.cache.get(&id)
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

    fn create_test_config() -> ProcessingConfig {
        ProcessingConfig {
            max_value: 1000.0,
            min_value: 0.0,
            allowed_tags: vec!["important".to_string(), "normal".to_string()],
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 500.0,
            tags: vec!["important".to_string()],
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 1500.0,
            tags: vec!["important".to_string()],
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_process_record() {
        let mut processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "  TEST  ".to_string(),
            value: 123.456,
            tags: vec!["normal".to_string()],
        };

        let result = processor.process_record(record);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.name, "test");
        assert_eq!(processed.value, 123.46);
        assert_eq!(processor.cache_size(), 1);
    }
}