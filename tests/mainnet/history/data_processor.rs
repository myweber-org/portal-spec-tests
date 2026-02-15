
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn process(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn batch_process(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records.into_iter().map(|record| self.process(record)).collect()
    }
}

fn validate_timestamp(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.timestamp < 0 {
        return Err(ProcessingError::ValidationError(
            "Timestamp cannot be negative".to_string(),
        ));
    }
    Ok(())
}

fn validate_values(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.values.is_empty() {
        return Err(ProcessingError::ValidationError(
            "Record must contain at least one value".to_string(),
        ));
    }

    for (key, value) in &record.values {
        if value.is_nan() || value.is_infinite() {
            return Err(ProcessingError::ValidationError(format!(
                "Invalid value for key '{}': {}",
                key, value
            )));
        }
    }
    Ok(())
}

fn normalize_values(mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
    let mean: f64 = record.values.values().sum::<f64>() / record.values.len() as f64;
    
    for value in record.values.values_mut() {
        *value = (*value - mean).abs();
    }

    record.tags.push("normalized".to_string());
    Ok(record)
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    
    processor.add_validation_rule(validate_timestamp);
    processor.add_validation_rule(validate_values);
    processor.add_transformation(normalize_values);
    
    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: [("temperature".to_string(), 25.5)].iter().cloned().collect(),
            tags: vec!["sensor".to_string()],
        };

        assert!(processor.process(valid_record).is_ok());

        let invalid_record = DataRecord {
            id: 2,
            timestamp: -1,
            values: [("pressure".to_string(), 1013.25)].iter().cloned().collect(),
            tags: vec![],
        };

        assert!(processor.process(invalid_record).is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = create_default_processor();
        
        let record = DataRecord {
            id: 3,
            timestamp: 1234567890,
            values: [
                ("a".to_string(), 10.0),
                ("b".to_string(), 20.0),
                ("c".to_string(), 30.0),
            ]
            .iter()
            .cloned()
            .collect(),
            tags: vec!["test".to_string()],
        };

        let result = processor.process(record).unwrap();
        assert!(result.tags.contains(&"normalized".to_string()));
        
        let mean = 20.0;
        let expected_values: HashMap<String, f64> = [
            ("a".to_string(), (10.0 - mean).abs()),
            ("b".to_string(), (20.0 - mean).abs()),
            ("c".to_string(), (30.0 - mean).abs()),
        ]
        .iter()
        .cloned()
        .collect();

        assert_eq!(result.values, expected_values);
    }
}