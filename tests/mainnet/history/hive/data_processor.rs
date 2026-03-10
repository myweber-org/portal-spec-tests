
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
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

    pub fn add_value(&mut self, key: String, value: f64) {
        self.values.insert(key, value);
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.values.is_empty() {
            return Err(DataError::ValidationFailed(
                "Record must contain at least one value".to_string(),
            ));
        }

        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationFailed(
                    "Value key cannot be empty".to_string(),
                ));
            }
            if !value.is_finite() {
                return Err(DataError::ValidationFailed(format!(
                    "Value for key '{}' must be finite",
                    key
                )));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&mut self) {
        let sum: f64 = self.values.values().sum();
        if sum != 0.0 {
            for value in self.values.values_mut() {
                *value /= sum;
            }
        }
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }

    pub fn process_records(&mut self) -> HashMap<String, f64> {
        let mut aggregated = HashMap::new();

        for record in &self.records {
            for (key, value) in &record.values {
                *aggregated.entry(key.clone()).or_insert(0.0) += value;
            }
        }

        aggregated
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature".to_string(), 25.5);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_record_validation_failure() {
        let record = DataRecord::new(2, -100);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_normalize_values() {
        let mut record = DataRecord::new(3, 1234567890);
        record.add_value("a".to_string(), 10.0);
        record.add_value("b".to_string(), 20.0);
        record.add_value("c".to_string(), 30.0);
        
        record.normalize_values();
        
        let sum: f64 = record.values.values().sum();
        assert!((sum - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut record1 = DataRecord::new(1, 1234567890);
        record1.add_value("metric1".to_string(), 10.0);
        record1.add_tag("important".to_string());
        
        let mut record2 = DataRecord::new(2, 1234567891);
        record2.add_value("metric1".to_string(), 20.0);
        record2.add_tag("important".to_string());
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        let aggregated = processor.process_records();
        assert_eq!(aggregated.get("metric1"), Some(&30.0));
        
        let filtered = processor.filter_by_tag("important");
        assert_eq!(filtered.len(), 2);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_numeric_data(&mut self, key: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty data provided".to_string());
        }

        if let Some(_) = self.cache.get(key) {
            return Err("Data already processed for this key".to_string());
        }

        let processed: Vec<f64> = data
            .iter()
            .filter(|&&x| x.is_finite())
            .map(|&x| x * 2.0)
            .collect();

        if processed.len() < data.len() {
            return Err("Invalid values filtered out".to_string());
        }

        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn get_statistics(&self, key: &str) -> Option<(f64, f64, f64)> {
        self.cache.get(key).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count;
            let std_dev = variance.sqrt();
            (mean, variance, std_dev)
        })
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0];
        
        let result = processor.process_numeric_data("test", data.clone());
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed, vec![2.0, 4.0, 6.0, 8.0]);
        
        let stats = processor.get_statistics("test").unwrap();
        assert_eq!(stats.0, 5.0);
    }

    #[test]
    fn test_empty_data() {
        let mut processor = DataProcessor::new();
        let result = processor.process_numeric_data("empty", vec![]);
        assert!(result.is_err());
    }
}
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
    validation_rules: HashMap<String, Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: HashMap::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(
        &mut self,
        name: &str,
        rule: Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>,
    ) {
        self.validation_rules.insert(name.to_string(), rule);
    }

    pub fn add_transformation(
        &mut self,
        transform: Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>,
    ) {
        self.transformation_pipeline.push(transform);
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for (rule_name, rule) in &self.validation_rules {
            rule(&record).map_err(|e| {
                ProcessingError::ValidationError(format!("Rule '{}' failed: {}", rule_name, e))
            })?;
        }

        for (index, transform) in self.transformation_pipeline.iter().enumerate() {
            record = transform(record).map_err(|e| {
                ProcessingError::TransformationFailed(format!("Step {} failed: {}", index + 1, e))
            })?;
        }

        Ok(record)
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> (Vec<DataRecord>, Vec<ProcessingError>) {
        let mut successful = Vec::new();
        let mut errors = Vec::new();

        for record in records {
            match self.process_record(record) {
                Ok(processed) => successful.push(processed),
                Err(e) => errors.push(e),
            }
        }

        (successful, errors)
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validation_rule(
        "value_positive",
        Box::new(|record| {
            if record.value >= 0.0 {
                Ok(())
            } else {
                Err(ProcessingError::InvalidData(
                    "Value must be non-negative".to_string(),
                ))
            }
        }),
    );

    processor.add_validation_rule(
        "name_not_empty",
        Box::new(|record| {
            if !record.name.trim().is_empty() {
                Ok(())
            } else {
                Err(ProcessingError::InvalidData(
                    "Name cannot be empty".to_string(),
                ))
            }
        }),
    );

    processor.add_transformation(Box::new(|mut record| {
        record.name = record.name.to_uppercase();
        Ok(record)
    }));

    processor.add_transformation(Box::new(|mut record| {
        record.value = (record.value * 100.0).round() / 100.0;
        Ok(record)
    }));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_passes() {
        let processor = create_default_processor();
        let record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: 42.5,
            tags: vec!["sample".to_string()],
        };

        assert!(processor.process_record(record).is_ok());
    }

    #[test]
    fn test_validation_fails_negative_value() {
        let processor = create_default_processor();
        let record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: -10.0,
            tags: vec![],
        };

        assert!(processor.process_record(record).is_err());
    }

    #[test]
    fn test_transformation_applied() {
        let processor = create_default_processor();
        let record = DataRecord {
            id: 1,
            name: "hello".to_string(),
            value: 42.567,
            tags: vec![],
        };

        let result = processor.process_record(record).unwrap();
        assert_eq!(result.name, "HELLO");
        assert_eq!(result.value, 42.57);
    }

    #[test]
    fn test_batch_processing() {
        let processor = create_default_processor();
        let records = vec![
            DataRecord {
                id: 1,
                name: "first".to_string(),
                value: 10.5,
                tags: vec![],
            },
            DataRecord {
                id: 2,
                name: "".to_string(),
                value: 20.0,
                tags: vec![],
            },
            DataRecord {
                id: 3,
                name: "third".to_string(),
                value: -5.0,
                tags: vec![],
            },
        ];

        let (successful, errors) = processor.process_batch(records);
        assert_eq!(successful.len(), 1);
        assert_eq!(errors.len(), 2);
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(Path::new(output_path))?;
    let mut writer = Writer::from_writer(output_file);
    
    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.is_valid() {
            writer.serialize(&record)?;
        } else {
            eprintln!("Invalid record skipped: {:?}", record);
        }
    }
    
    writer.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let count = records.len();
    if count == 0 {
        return (0.0, 0.0, 0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = sum / count as f64;
    let max = records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);
    
    (avg, max, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 42.5,
            active: true,
        };
        assert!(valid_record.is_valid());
        
        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -10.0,
            active: false,
        };
        assert!(!invalid_record.is_valid());
    }
    
    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];
        
        let (avg, max, count) = calculate_statistics(&records);
        assert_eq!(avg, 20.0);
        assert_eq!(max, 30.0);
        assert_eq!(count, 3);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_numeric_data(&mut self, key: &str, values: &[f64]) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data set provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(values)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        for &value in data {
            if !value.is_finite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }
        Ok(data.to_vec())
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev.abs() < 1e-10 {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.powi(2).ln_1p().tanh())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_entries = self.cache.len();
        let total_values = self.cache.values().map(|v| v.len()).sum();
        (total_entries, total_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_numeric_data("test", &data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), data.len());
        
        let stats = processor.get_cache_stats();
        assert_eq!(stats.0, 1);
        assert_eq!(stats.1, 5);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, f64::NAN, 3.0];
        
        let result = processor.process_numeric_data("invalid", &data);
        assert!(result.is_err());
    }
}