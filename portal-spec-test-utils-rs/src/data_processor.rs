
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn batch_process(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());
        
        for record in records {
            match self.process(record) {
                Ok(processed) => results.push(processed),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    
    processor.add_validation_rule(|record| {
        if record.values.is_empty() {
            Err(ProcessingError::ValidationError("Empty values array".to_string()))
        } else {
            Ok(())
        }
    });
    
    processor.add_validation_rule(|record| {
        if record.timestamp < 0 {
            Err(ProcessingError::ValidationError("Negative timestamp".to_string()))
        } else {
            Ok(())
        }
    });
    
    processor.add_transformation(|mut record| {
        let sum: f64 = record.values.iter().sum();
        let count = record.values.len() as f64;
        let average = sum / count;
        
        record.metadata.insert("average".to_string(), average.to_string());
        record.metadata.insert("count".to_string(), count.to_string());
        
        Ok(record)
    });
    
    processor.add_transformation(|mut record| {
        record.values = record.values.iter().map(|&x| x * 2.0).collect();
        Ok(record)
    });
    
    processor
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_processing() {
        let processor = create_default_processor();
        
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0, 4.0],
            metadata,
        };
        
        let result = processor.process(record).unwrap();
        
        assert_eq!(result.values, vec![2.0, 4.0, 6.0, 8.0]);
        assert_eq!(result.metadata.get("average").unwrap(), "2.5");
        assert_eq!(result.metadata.get("count").unwrap(), "4");
    }
    
    #[test]
    fn test_validation_error() {
        let processor = create_default_processor();
        
        let record = DataRecord {
            id: 2,
            timestamp: -100,
            values: vec![1.0, 2.0],
            metadata: HashMap::new(),
        };
        
        let result = processor.process(record);
        assert!(result.is_err());
    }
}use std::collections::HashMap;
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
    config: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new(config: HashMap<String, String>) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record name cannot be empty".to_string(),
            ));
        }

        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError(
                "Record value must be non-negative".to_string(),
            ));
        }

        if record.tags.len() > 10 {
            return Err(ProcessingError::ValidationError(
                "Too many tags, maximum is 10".to_string(),
            ));
        }

        Ok(())
    }

    pub fn transform_record(
        &self,
        record: DataRecord,
    ) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();

        if let Some(prefix) = self.config.get("name_prefix") {
            transformed.name = format!("{}{}", prefix, transformed.name);
        }

        if let Some(factor_str) = self.config.get("value_multiplier") {
            if let Ok(factor) = factor_str.parse::<f64>() {
                transformed.value *= factor;
            } else {
                return Err(ProcessingError::TransformationFailed(
                    "Invalid multiplier configuration".to_string(),
                ));
            }
        }

        if let Some(default_tag) = self.config.get("default_tag") {
            if transformed.tags.is_empty() {
                transformed.tags.push(default_tag.clone());
            }
        }

        Ok(transformed)
    }

    pub fn process_records(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());

        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(record)?;
            processed.push(transformed);
        }

        Ok(processed)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if records.is_empty() {
            return stats;
        }

        let count = records.len() as f64;
        let sum: f64 = records.iter().map(|r| r.value).sum();
        let avg = sum / count;

        let variance: f64 = records
            .iter()
            .map(|r| (r.value - avg).powi(2))
            .sum::<f64>()
            / count;

        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("average".to_string(), avg);
        stats.insert("variance".to_string(), variance);

        if let Some(max) = records.iter().map(|r| r.value).max_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("maximum".to_string(), max);
        }

        if let Some(min) = records.iter().map(|r| r.value).min_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("minimum".to_string(), min);
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let config = HashMap::new();
        let processor = DataProcessor::new(config);

        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec!["tag1".to_string()],
        };

        assert!(processor.validate_record(&valid_record).is_ok());

        let invalid_record = DataRecord {
            id: 2,
            name: "".to_string(),
            value: -10.0,
            tags: vec![],
        };

        assert!(processor.validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_transformation() {
        let mut config = HashMap::new();
        config.insert("name_prefix".to_string(), "PRE_".to_string());
        config.insert("value_multiplier".to_string(), "2.0".to_string());

        let processor = DataProcessor::new(config);
        let record = DataRecord {
            id: 1,
            name: "data".to_string(),
            value: 50.0,
            tags: vec![],
        };

        let transformed = processor.transform_record(record).unwrap();
        assert_eq!(transformed.name, "PRE_data");
        assert_eq!(transformed.value, 100.0);
    }

    #[test]
    fn test_statistics() {
        let config = HashMap::new();
        let processor = DataProcessor::new(config);

        let records = vec![
            DataRecord {
                id: 1,
                name: "A".to_string(),
                value: 10.0,
                tags: vec![],
            },
            DataRecord {
                id: 2,
                name: "B".to_string(),
                value: 20.0,
                tags: vec![],
            },
            DataRecord {
                id: 3,
                name: "C".to_string(),
                value: 30.0,
                tags: vec![],
            },
        ];

        let stats = processor.calculate_statistics(&records);
        assert_eq!(stats.get("count").unwrap(), &3.0);
        assert_eq!(stats.get("sum").unwrap(), &60.0);
        assert_eq!(stats.get("average").unwrap(), &20.0);
        assert_eq!(stats.get("minimum").unwrap(), &10.0);
        assert_eq!(stats.get("maximum").unwrap(), &30.0);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_threshold: f64,
    transformation_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, transformation_factor: f64) -> Self {
        DataProcessor {
            validation_threshold,
            transformation_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "Empty values array".to_string(),
            ));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationFailed(
                    "Invalid numeric value".to_string(),
                ));
            }

            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::ValidationFailed(format!(
                    "Value {} exceeds threshold {}",
                    value, self.validation_threshold
                )));
            }
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::ValidationFailed(
                "Negative timestamp".to_string(),
            ));
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::TransformationError(
                "Cannot transform empty array".to_string(),
            ));
        }

        for value in &mut record.values {
            *value *= self.transformation_factor;
            
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::TransformationError(
                    "Transformation produced invalid value".to_string(),
                ));
            }
        }

        record.metadata.insert(
            "processed".to_string(),
            "true".to_string(),
        );
        record.metadata.insert(
            "transformation_factor".to_string(),
            self.transformation_factor.to_string(),
        );

        Ok(())
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;
        self.transform_values(&mut record)?;
        
        record.metadata.insert(
            "processing_timestamp".to_string(),
            chrono::Utc::now().timestamp().to_string(),
        );

        Ok(record)
    }
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }

    let total_values: usize = records.iter().map(|r| r.values.len()).sum();
    if total_values == 0 {
        return stats;
    }

    let all_values: Vec<f64> = records
        .iter()
        .flat_map(|r| r.values.clone())
        .collect();

    let sum: f64 = all_values.iter().sum();
    let count = all_values.len() as f64;
    
    let mean = sum / count;
    let variance: f64 = all_values
        .iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>() / count;

    stats.insert("mean".to_string(), mean);
    stats.insert("variance".to_string(), variance);
    stats.insert("total_records".to_string(), records.len() as f64);
    stats.insert("total_values".to_string(), total_values as f64);

    if let Some(min) = all_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()) {
        stats.insert("min".to_string(), *min);
    }
    
    if let Some(max) = all_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
        stats.insert("max".to_string(), *max);
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.5, 20.3, 30.7],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(10.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.5, 20.3, 30.7],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transformation() {
        let processor = DataProcessor::new(1000.0, 2.5);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![2.0, 4.0, 6.0],
            metadata: HashMap::new(),
        };

        assert!(processor.transform_values(&mut record).is_ok());
        assert_eq!(record.values, vec![5.0, 10.0, 15.0]);
        assert_eq!(record.metadata.get("processed"), Some(&"true".to_string()));
    }
}