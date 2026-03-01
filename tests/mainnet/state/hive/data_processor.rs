
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, timestamp: String) -> Self {
        DataRecord {
            id,
            name,
            value,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.timestamp.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
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

            let timestamp = parts[3].to_string();

            let record = DataRecord::new(id, name, value, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_value(&self, min_value: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= min_value)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "test".to_string(), 10.5, "2023-01-01".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,item1,10.5,2023-01-01").unwrap();
        writeln!(temp_file, "2,item2,20.0,2023-01-02").unwrap();
        writeln!(temp_file, "3,item3,invalid,2023-01-03").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 2);
    }

    #[test]
    fn test_filter_and_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "a".to_string(), 10.0, "t1".to_string()));
        processor.records.push(DataRecord::new(2, "b".to_string(), 20.0, "t2".to_string()));
        processor.records.push(DataRecord::new(3, "c".to_string(), 30.0, "t3".to_string()));

        let filtered = processor.filter_by_value(15.0);
        assert_eq!(filtered.len(), 2);

        let avg = processor.calculate_average();
        assert_eq!(avg, Some(20.0));
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    filters: Vec<Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            filters: Vec::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn add_transformer<N, T>(&mut self, name: N, transformer: T)
    where
        N: Into<String>,
        T: Fn(String) -> String + 'static,
    {
        self.transformers.insert(name.into(), Box::new(transformer));
    }

    pub fn process_data(&self, input: &str) -> Option<String> {
        if !self.filters.iter().all(|f| f(input)) {
            return None;
        }

        let mut result = input.to_string();
        for transformer in self.transformers.values() {
            result = transformer(result);
        }

        Some(result)
    }

    pub fn batch_process(&self, inputs: Vec<&str>) -> Vec<String> {
        inputs
            .iter()
            .filter_map(|&input| self.process_data(input))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_filter(|s| s.len() > 3);
        processor.add_transformer("uppercase", |s| s.to_uppercase());
        processor.add_transformer("trim", |s| s.trim().to_string());

        let result = processor.process_data("  test data  ");
        assert_eq!(result, Some("TEST DATA".to_string()));

        let filtered = processor.process_data("abc");
        assert_eq!(filtered, None);

        let batch = processor.batch_process(vec!["  one  ", "two", "  three  "]);
        assert_eq!(batch, vec!["ONE", "THREE"]);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
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
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,A").unwrap();
        writeln!(temp_file, "2,20.3,B").unwrap();
        writeln!(temp_file, "3,15.7,A").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert!((stats.0 - 15.5).abs() < 0.01);
        
        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }
        
        Ok(Self { threshold })
    }
    
    pub fn process_values(&self, values: &[f64]) -> Vec<f64> {
        values
            .iter()
            .filter(|&&v| v >= self.threshold)
            .map(|&v| v * 2.0)
            .collect()
    }
    
    pub fn calculate_statistics(&self, values: &[f64]) -> (f64, f64, f64) {
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());
    }
    
    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(1.5);
        assert!(processor.is_err());
    }
    
    #[test]
    fn test_process_values() {
        let processor = DataProcessor::new(0.3).unwrap();
        let values = vec![0.1, 0.4, 0.2, 0.5, 0.6];
        let result = processor.process_values(&values);
        assert_eq!(result, vec![0.8, 1.0, 1.2]);
    }
    
    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(0.0).unwrap();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = processor.calculate_statistics(&values);
        
        assert!((mean - 3.0).abs() < 0.0001);
        assert!((variance - 2.0).abs() < 0.0001);
        assert!((std_dev - 1.41421356237).abs() < 0.0001);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Error)]
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
        Self {
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

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if record.values.iter().any(|v| v.is_nan()) {
            return Err(ProcessingError::TransformationError(
                "Cannot transform NaN values".to_string(),
            ));
        }

        for value in &mut record.values {
            *value *= self.transformation_factor;
        }

        record.metadata.insert(
            "transformed".to_string(),
            format!("factor_{}", self.transformation_factor),
        );

        Ok(())
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());

        for mut record in records {
            self.validate_record(&record)?;
            self.transform_values(&mut record)?;
            processed_records.push(record);
        }

        Ok(processed_records)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        let all_values: Vec<f64> = records
            .iter()
            .flat_map(|r| r.values.iter().copied())
            .collect();

        if !all_values.is_empty() {
            let sum: f64 = all_values.iter().sum();
            let count = all_values.len() as f64;
            let mean = sum / count;

            let variance: f64 = all_values
                .iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>()
                / count;

            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("count".to_string(), count);
            stats.insert("sum".to_string(), sum);

            if let Some(min) = all_values.iter().copied().reduce(f64::min) {
                stats.insert("min".to_string(), min);
            }
            if let Some(max) = all_values.iter().copied().reduce(f64::max) {
                stats.insert("max".to_string(), max);
            }
        }

        stats
    }
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
    fn test_validation_threshold_exceeded() {
        let processor = DataProcessor::new(10.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![5.0, 15.0, 8.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_values() {
        let processor = DataProcessor::new(1000.0, 3.0);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![2.0, 4.0, 6.0],
            metadata: HashMap::new(),
        };

        assert!(processor.transform_values(&mut record).is_ok());
        assert_eq!(record.values, vec![6.0, 12.0, 18.0]);
        assert_eq!(
            record.metadata.get("transformed"),
            Some(&"factor_3".to_string())
        );
    }

    #[test]
    fn test_process_batch() {
        let processor = DataProcessor::new(100.0, 2.0);
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1234567890,
                values: vec![10.0, 20.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 1234567891,
                values: vec![30.0, 40.0],
                metadata: HashMap::new(),
            },
        ];

        let result = processor.process_batch(records);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].values, vec![20.0, 40.0]);
        assert_eq!(processed[1].values, vec![60.0, 80.0]);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        let mut processor = DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        };
        
        processor.register_default_validators();
        processor.register_default_transformers();
        
        processor
    }
    
    fn register_default_validators(&mut self) {
        self.validators.insert(
            "email".to_string(),
            Box::new(|input: &str| {
                input.contains('@') && input.contains('.') && input.len() > 5
            })
        );
        
        self.validators.insert(
            "numeric".to_string(),
            Box::new(|input: &str| {
                input.parse::<f64>().is_ok()
            })
        );
    }
    
    fn register_default_transformers(&mut self) {
        self.transformers.insert(
            "uppercase".to_string(),
            Box::new(|input: String| {
                input.to_uppercase()
            })
        );
        
        self.transformers.insert(
            "trim".to_string(),
            Box::new(|input: String| {
                input.trim().to_string()
            })
        );
    }
    
    pub fn validate(&self, validator_name: &str, input: &str) -> bool {
        match self.validators.get(validator_name) {
            Some(validator) => validator(input),
            None => false,
        }
    }
    
    pub fn transform(&self, transformer_name: &str, input: String) -> Option<String> {
        self.transformers.get(transformer_name)
            .map(|transformer| transformer(input))
    }
    
    pub fn process_data(&self, input: &str) -> Result<String, String> {
        if !self.validate("email", input) {
            return Err("Invalid email format".to_string());
        }
        
        let trimmed = self.transform("trim", input.to_string())
            .ok_or("Transformation failed")?;
        
        let result = self.transform("uppercase", trimmed)
            .ok_or("Transformation failed")?;
        
        Ok(result)
    }
}

pub fn create_processor() -> DataProcessor {
    DataProcessor::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("email", "test@example.com"));
        assert!(!processor.validate("email", "invalid-email"));
    }
    
    #[test]
    fn test_numeric_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("numeric", "123.45"));
        assert!(!processor.validate("numeric", "abc"));
    }
    
    #[test]
    fn test_uppercase_transformation() {
        let processor = DataProcessor::new();
        let result = processor.transform("uppercase", "hello".to_string());
        assert_eq!(result, Some("HELLO".to_string()));
    }
    
    #[test]
    fn test_trim_transformation() {
        let processor = DataProcessor::new();
        let result = processor.transform("trim", "  hello  ".to_string());
        assert_eq!(result, Some("hello".to_string()));
    }
    
    #[test]
    fn test_process_data_success() {
        let processor = DataProcessor::new();
        let result = processor.process_data("  user@domain.com  ");
        assert_eq!(result, Ok("USER@DOMAIN.COM".to_string()));
    }
    
    #[test]
    fn test_process_data_failure() {
        let processor = DataProcessor::new();
        let result = processor.process_data("invalid-email");
        assert!(result.is_err());
    }
}