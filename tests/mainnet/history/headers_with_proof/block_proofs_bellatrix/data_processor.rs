use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines().skip(1) {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    self.data.push(value);
                }
                
                let category = parts[0].to_string();
                *self.frequency_map.entry(category).or_insert(0) += 1;
            }
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_median(&mut self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        self.data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = self.data.len() / 2;
        
        if self.data.len() % 2 == 0 {
            Some((self.data[mid - 1] + self.data[mid]) / 2.0)
        } else {
            Some(self.data[mid])
        }
    }

    pub fn get_frequency_distribution(&self) -> &HashMap<String, u32> {
        &self.frequency_map
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
    }

    pub fn data_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let count = self.data.len();
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        format!(
            "Data Summary:\n  Count: {}\n  Mean: {:.2}\n  Range: [{:.2}, {:.2}]",
            count, mean, min, max
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "category,value").unwrap();
        writeln!(temp_file, "A,10.5").unwrap();
        writeln!(temp_file, "B,20.3").unwrap();
        writeln!(temp_file, "A,15.7").unwrap();
        writeln!(temp_file, "C,8.9").unwrap();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.calculate_mean(), Some(13.85));
        assert_eq!(processor.filter_by_threshold(12.0).len(), 2);
        
        let freq = processor.get_frequency_distribution();
        assert_eq!(freq.get("A"), Some(&2));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let name = parts[1].to_string();
            
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let active = match parts[3].to_lowercase().as_str() {
                "true" | "1" | "yes" => true,
                _ => false,
            };

            let record = Record::new(id, name, value, active);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records
            .iter()
            .map(|r| r.value)
            .sum()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Record> {
        self.records
            .iter()
            .find(|r| r.name.to_lowercase() == name.to_lowercase())
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 10.5, true);
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -5.0, false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Alice,100.5,true").unwrap();
        writeln!(temp_file, "2,Bob,75.2,false").unwrap();
        writeln!(temp_file, "3,Charlie,50.0,true").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.filter_active().len(), 2);
        assert_eq!(processor.calculate_total(), 225.7);
        
        let found = processor.find_by_name("alice");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 1);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Value is invalid"),
            ProcessingError::InvalidTimestamp => write!(f, "Timestamp is invalid"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.value.is_nan() || record.value.is_infinite() {
        return Err(ProcessingError::InvalidValue);
    }
    
    if record.timestamp < 0 {
        return Err(ProcessingError::InvalidTimestamp);
    }
    
    if record.id == 0 {
        return Err(ProcessingError::ValidationFailed("ID cannot be zero".to_string()));
    }
    
    Ok(())
}

pub fn transform_record(record: DataRecord, multiplier: f64) -> Result<DataRecord, ProcessingError> {
    validate_record(&record)?;
    
    let transformed_value = record.value * multiplier;
    if transformed_value.is_nan() || transformed_value.is_infinite() {
        return Err(ProcessingError::InvalidValue);
    }
    
    Ok(DataRecord {
        id: record.id,
        value: transformed_value,
        timestamp: record.timestamp,
    })
}

pub fn process_records(records: Vec<DataRecord>, multiplier: f64) -> Vec<Result<DataRecord, ProcessingError>> {
    records
        .into_iter()
        .map(|record| transform_record(record, multiplier))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1625097600,
        };
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_invalid_value() {
        let record = DataRecord {
            id: 1,
            value: f64::NAN,
            timestamp: 1625097600,
        };
        assert!(matches!(validate_record(&record), Err(ProcessingError::InvalidValue)));
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord {
            id: 1,
            value: 10.0,
            timestamp: 1625097600,
        };
        let result = transform_record(record, 2.5);
        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed.value, 25.0);
    }

    #[test]
    fn test_process_multiple_records() {
        let records = vec![
            DataRecord { id: 1, value: 10.0, timestamp: 1000 },
            DataRecord { id: 2, value: 20.0, timestamp: 2000 },
            DataRecord { id: 3, value: f64::INFINITY, timestamp: 3000 },
        ];
        
        let results = process_records(records, 2.0);
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_err());
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid input data")]
    InvalidData,
    #[error("Transformation failed")]
    TransformationFailed,
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

pub struct DataProcessor {
    validation_rules: Vec<ValidationRule>,
    transformation_pipeline: Vec<Transformation>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformation_pipeline.push(transformation);
    }

    pub fn process(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        for rule in &self.validation_rules {
            rule.validate(record)?;
        }

        for transformation in &self.transformation_pipeline {
            transformation.apply(record)?;
        }

        Ok(())
    }

    pub fn batch_process(
        &self,
        records: &mut [DataRecord],
    ) -> Result<Vec<Result<(), ProcessingError>>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());

        for record in records.iter_mut() {
            results.push(self.process(record));
        }

        Ok(results)
    }
}

pub trait ValidationRule {
    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError>;
}

pub trait Transformation {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError>;
}

pub struct TimestampValidator {
    min_timestamp: i64,
    max_timestamp: i64,
}

impl TimestampValidator {
    pub fn new(min: i64, max: i64) -> Self {
        TimestampValidator {
            min_timestamp: min,
            max_timestamp: max,
        }
    }
}

impl ValidationRule for TimestampValidator {
    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.timestamp < self.min_timestamp || record.timestamp > self.max_timestamp {
            return Err(ProcessingError::ValidationError(format!(
                "Timestamp {} out of range [{}, {}]",
                record.timestamp, self.min_timestamp, self.max_timestamp
            )));
        }
        Ok(())
    }
}

pub struct ValueNormalizer {
    target_range: (f64, f64),
}

impl ValueNormalizer {
    pub fn new(min: f64, max: f64) -> Self {
        ValueNormalizer {
            target_range: (min, max),
        }
    }
}

impl Transformation for ValueNormalizer {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        for value in record.values.values_mut() {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::TransformationFailed);
            }
            
            *value = value.clamp(self.target_range.0, self.target_range.1);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(Box::new(TimestampValidator::new(0, 1000)));
        processor.add_transformation(Box::new(ValueNormalizer::new(0.0, 1.0)));

        let mut record = DataRecord {
            id: 1,
            timestamp: 500,
            values: {
                let mut map = HashMap::new();
                map.insert("temperature".to_string(), 25.5);
                map.insert("humidity".to_string(), 0.85);
                map
            },
            tags: vec!["sensor".to_string(), "room1".to_string()],
        };

        let result = processor.process(&mut record);
        assert!(result.is_ok());
        
        for value in record.values.values() {
            assert!(*value >= 0.0 && *value <= 1.0);
        }
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(Box::new(TimestampValidator::new(0, 100)));

        let record = DataRecord {
            id: 1,
            timestamp: 200,
            values: HashMap::new(),
            tags: Vec::new(),
        };

        let mut temp_record = record.clone();
        let result = processor.process(&mut temp_record);
        assert!(result.is_err());
    }
}