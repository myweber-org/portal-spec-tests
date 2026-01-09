
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
            if value.is_nan() || value.is_infinite() {
                return Err(DataError::ValidationFailed(format!(
                    "Invalid value for key '{}': {}",
                    key, value
                )));
            }
        }

        Ok(())
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    statistics: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            statistics: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        record.validate()?;
        self.records.push(record);
        self.update_statistics();
        Ok(())
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn get_average(&self, key: &str) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;

        for record in &self.records {
            if let Some(value) = record.values.get(key) {
                sum += value;
                count += 1;
            }
        }

        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }

    fn update_statistics(&mut self) {
        self.statistics.clear();
        let count = self.records.len() as f64;

        if count == 0.0 {
            return;
        }

        let mut value_keys = Vec::new();
        for record in &self.records {
            for key in record.values.keys() {
                if !value_keys.contains(key) {
                    value_keys.push(key.clone());
                }
            }
        }

        for key in value_keys {
            if let Some(avg) = self.get_average(&key) {
                self.statistics.insert(format!("avg_{}", key), avg);
            }
        }

        self.statistics.insert("total_records".to_string(), count);
    }

    pub fn get_statistics(&self) -> &HashMap<String, f64> {
        &self.statistics
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.statistics.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature".to_string(), 25.5);
        record.add_tag("sensor".to_string());

        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_empty_record_validation() {
        let record = DataRecord::new(1, 1234567890);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();

        let mut record1 = DataRecord::new(1, 1000);
        record1.add_value("pressure".to_string(), 1013.25);
        record1.add_tag("weather".to_string());

        let mut record2 = DataRecord::new(2, 2000);
        record2.add_value("pressure".to_string(), 1012.50);
        record2.add_tag("weather".to_string());

        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());

        let weather_records = processor.filter_by_tag("weather");
        assert_eq!(weather_records.len(), 2);

        let avg_pressure = processor.get_average("pressure");
        assert!(avg_pressure.is_some());
        assert!((avg_pressure.unwrap() - 1012.875).abs() < 0.001);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            lines.next();
        }

        for (line_number, line) in lines {
            let line_content = line?;
            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.iter().any(|f| f.is_empty()) {
                return Err(format!("Empty field detected at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records to validate".into());
        }

        let expected_len = records[0].len();
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!("Record {} has {} fields, expected {}", 
                                 idx + 1, record.len(), expected_len).into());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_validation_success() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_records(&records).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_records(&records).is_err());
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: ValidationRules,
}

pub struct ValidationRules {
    min_value: f64,
    max_value: f64,
    required_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(rules: ValidationRules) -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: rules,
        }
    }

    pub fn add_dataset(&mut self, key: String, values: Vec<f64>) -> Result<(), String> {
        if !self.validation_rules.required_keys.contains(&key) {
            return Err(format!("Key '{}' is not in required keys list", key));
        }

        for &value in &values {
            if value < self.validation_rules.min_value || value > self.validation_rules.max_value {
                return Err(format!("Value {} is outside allowed range [{}, {}]", 
                    value, self.validation_rules.min_value, self.validation_rules.max_value));
            }
        }

        self.data.insert(key, values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            let std_dev = variance.sqrt();

            Statistics {
                count,
                sum,
                mean,
                variance,
                std_dev,
            }
        })
    }

    pub fn normalize_data(&mut self, key: &str) -> Result<(), String> {
        if let Some(values) = self.data.get_mut(key) {
            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            if (max - min).abs() < f64::EPSILON {
                return Err("Cannot normalize constant dataset".to_string());
            }

            for value in values.iter_mut() {
                *value = (*value - min) / (max - min);
            }
            Ok(())
        } else {
            Err(format!("Key '{}' not found in dataset", key))
        }
    }

    pub fn merge_datasets(&mut self, target_key: &str, source_key: &str) -> Result<(), String> {
        if let Some(source_values) = self.data.remove(source_key) {
            if let Some(target_values) = self.data.get_mut(target_key) {
                target_values.extend(source_values);
                Ok(())
            } else {
                self.data.insert(source_key.to_string(), source_values);
                Err(format!("Target key '{}' not found", target_key))
            }
        } else {
            Err(format!("Source key '{}' not found", source_key))
        }
    }
}

pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

impl ValidationRules {
    pub fn new(min_value: f64, max_value: f64, required_keys: Vec<String>) -> Self {
        ValidationRules {
            min_value,
            max_value,
            required_keys,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_validation() {
        let rules = ValidationRules::new(0.0, 100.0, vec!["temperature".to_string()]);
        let mut processor = DataProcessor::new(rules);
        
        assert!(processor.add_dataset("temperature".to_string(), vec![25.0, 30.0, 35.0]).is_ok());
        assert!(processor.add_dataset("pressure".to_string(), vec![101.3]).is_err());
        assert!(processor.add_dataset("temperature".to_string(), vec![-5.0]).is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let rules = ValidationRules::new(f64::NEG_INFINITY, f64::INFINITY, 
            vec!["dataset".to_string()]);
        let mut processor = DataProcessor::new(rules);
        
        processor.add_dataset("dataset".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        let stats = processor.calculate_statistics("dataset").unwrap();
        
        assert_eq!(stats.count, 5);
        assert_eq!(stats.sum, 15.0);
        assert_eq!(stats.mean, 3.0);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    OutOfRange(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
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
        if record.values.is_empty() {
            return Err(ProcessingError::InvalidFormat);
        }

        for (i, &value) in record.values.iter().enumerate() {
            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::OutOfRange(
                    format!("Value at index {} exceeds threshold", i)
                ));
            }
        }

        if !record.metadata.contains_key("source") {
            return Err(ProcessingError::MissingField("source".to_string()));
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord) {
        let mean = record.values.iter().sum::<f64>() / record.values.len() as f64;
        
        record.values = record.values
            .iter()
            .map(|&v| (v - mean).abs())
            .collect();
    }

    pub fn process_batch(&self, records: &mut [DataRecord]) -> Vec<Result<DataRecord, ProcessingError>> {
        records.iter_mut()
            .map(|record| {
                self.validate_record(record)
                    .map(|_| {
                        self.transform_values(record);
                        record.clone()
                    })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_passes() {
        let processor = DataProcessor::new(100.0);
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.5, 20.3, 30.7],
            metadata,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_fails_on_threshold() {
        let processor = DataProcessor::new(50.0);
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.5, 60.0, 30.7],
            metadata,
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_values() {
        let processor = DataProcessor::new(100.0);
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 20.0, 30.0],
            metadata,
        };

        processor.transform_values(&mut record);
        let expected = vec![10.0, 0.0, 10.0];
        
        for (i, &value) in record.values.iter().enumerate() {
            assert!((value - expected[i]).abs() < 0.0001);
        }
    }
}