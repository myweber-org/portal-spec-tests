use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: String) -> Self {
        DataRecord {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.category.is_empty() 
            && self.value >= 0.0 
            && self.value <= 1000.0
            && self.timestamp.len() == 19
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
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            let record = DataRecord::new(
                id,
                value,
                parts[2].to_string(),
                parts[3].to_string(),
            );

            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(
            1,
            500.0,
            "electronics".to_string(),
            "2024-01-15 10:30:00".to_string(),
        );
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(
            2,
            -10.0,
            "".to_string(),
            "2024-01-15".to_string(),
        );
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);
        
        processor.records.push(DataRecord::new(
            1,
            100.0,
            "test".to_string(),
            "2024-01-15 10:30:00".to_string(),
        ));
        
        assert_eq!(processor.count_records(), 1);
        assert_eq!(processor.calculate_average(), Some(100.0));
        
        processor.clear();
        assert_eq!(processor.count_records(), 0);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
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

    pub fn add_value(&mut self, key: &str, value: f64) {
        self.values.insert(key.to_string(), value);
    }

    pub fn add_tag(&mut self, tag: &str) {
        self.tags.push(tag.to_string());
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
                    "Value for '{}' must be finite",
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

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records.iter_mut() {
        record.validate()?;
        record.normalize_values();
        processed.push(record.clone());
    }

    Ok(processed)
}

pub fn filter_records_by_tag(
    records: &[DataRecord],
    required_tag: &str,
) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|record| record.tags.iter().any(|tag| tag == required_tag))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature", 25.5);
        record.add_tag("sensor");

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
        record.add_value("a", 1.0);
        record.add_value("b", 2.0);
        record.add_value("c", 3.0);

        record.normalize_values();

        let expected_sum: f64 = record.values.values().sum();
        assert!((expected_sum - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_filter_by_tag() {
        let mut record1 = DataRecord::new(1, 100);
        record1.add_tag("important");

        let mut record2 = DataRecord::new(2, 200);
        record2.add_tag("normal");

        let records = vec![record1, record2];
        let filtered = filter_records_by_tag(&records, "important");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
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
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    MissingMetadata(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.name.trim().is_empty() {
        return Err(ValidationError::EmptyName);
    }
    
    if record.value < 0.0 {
        return Err(ValidationError::NegativeValue);
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord) -> DataRecord {
    let mut transformed = record.clone();
    transformed.name = record.name.to_uppercase();
    transformed.value = (record.value * 100.0).round() / 100.0;
    
    let mut new_metadata = record.metadata.clone();
    new_metadata.insert("processed_timestamp".to_string(), 
                       chrono::Utc::now().to_rfc3339());
    new_metadata.insert("original_value".to_string(), 
                       record.value.to_string());
    
    transformed.metadata = new_metadata;
    transformed
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ValidationError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        validate_record(&record)?;
        let transformed = transform_record(&record);
        processed.push(transformed);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 42.5,
            metadata: HashMap::new(),
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_record_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 42.5,
            metadata: HashMap::new(),
        };
        
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_transform_record() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            name: "example".to_string(),
            value: 42.567,
            metadata,
        };
        
        let transformed = transform_record(&record);
        
        assert_eq!(transformed.name, "EXAMPLE");
        assert_eq!(transformed.value, 42.57);
        assert!(transformed.metadata.contains_key("processed_timestamp"));
        assert!(transformed.metadata.contains_key("original_value"));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err(format!("Invalid value {} for record {}", value, id));
        }
        if category.is_empty() {
            return Err(format!("Empty category for record {}", id));
        }
        Ok(DataRecord { id, value, category })
    }

    pub fn calculate_score(&self) -> f64 {
        self.value * self.category.len() as f64
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;

        for line in reader.lines() {
            line_count += 1;
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() != 3 {
                return Err(format!("Invalid CSV format at line {}", line_count).into());
            }

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();

            match DataRecord::new(id, value, category) {
                Ok(record) => self.records.push(record),
                Err(e) => eprintln!("Warning: {} at line {}", e, line_count),
            }
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total_score(&self) -> f64 {
        self.records.iter().map(|record| record.calculate_score()).sum()
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let count = self.records.len() as f64;
        if count == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let avg = sum / count;
        let max = self.records.iter().map(|r| r.value).fold(0.0, f64::max);
        (sum, avg, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string()).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_record() {
        assert!(DataRecord::new(2, -1.0, "test".to_string()).is_err());
        assert!(DataRecord::new(3, 10.0, "".to_string()).is_err());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,100.0,alpha\n2,200.0,beta\n3,300.0,alpha").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
    }
}