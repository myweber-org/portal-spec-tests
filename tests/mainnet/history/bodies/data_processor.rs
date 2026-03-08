
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord {
            id,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid format at line {}", line_number + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let value = parts[1].parse::<f64>()?;
        let category = parts[2].to_string();

        let record = DataRecord::new(id, value, category);
        if record.is_valid() {
            records.push(record);
        } else {
            eprintln!("Warning: Invalid record at line {}", line_number + 1);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    let std_dev = variance.sqrt();

    (sum, mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record1 = DataRecord::new(0, 42.5, "test".to_string());
        assert!(!record1.is_valid());

        let record2 = DataRecord::new(1, -1.0, "test".to_string());
        assert!(!record2.is_valid());

        let record3 = DataRecord::new(1, 42.5, "".to_string());
        assert!(!record3.is_valid());
    }

    #[test]
    fn test_process_csv() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "1,10.5,category_a")?;
        writeln!(temp_file, "2,20.3,category_b")?;
        writeln!(temp_file, "# Comment line")?;
        writeln!(temp_file, "3,15.7,category_c")?;

        let records = process_csv_file(temp_file.path().to_str().unwrap())?;
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].value, 20.3);
        assert_eq!(records[2].category, "category_c");

        Ok(())
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 10.0, "a".to_string()),
            DataRecord::new(2, 20.0, "b".to_string()),
            DataRecord::new(3, 30.0, "c".to_string()),
        ];

        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
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
    config: ProcessingConfig,
}

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub max_values: usize,
    pub require_timestamp: bool,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub enum ValidationRule {
    ValueRange { min: f64, max: f64 },
    NonEmptyMetadata,
    TimestampRecency { threshold_seconds: i64 },
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn process(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate(record)?;
        self.transform(record)
    }

    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.config.max_values {
            return Err(ProcessingError::ValidationFailed(
                format!("Exceeds maximum values limit: {}", self.config.max_values)
            ));
        }

        if self.config.require_timestamp && record.timestamp <= 0 {
            return Err(ProcessingError::ValidationFailed(
                "Invalid timestamp".to_string()
            ));
        }

        for rule in &self.config.validation_rules {
            match rule {
                ValidationRule::ValueRange { min, max } => {
                    for value in &record.values {
                        if value < min || value > max {
                            return Err(ProcessingError::ValidationFailed(
                                format!("Value {} outside range [{}, {}]", value, min, max)
                            ));
                        }
                    }
                }
                ValidationRule::NonEmptyMetadata => {
                    if record.metadata.is_empty() {
                        return Err(ProcessingError::ValidationFailed(
                            "Metadata cannot be empty".to_string()
                        ));
                    }
                }
                ValidationRule::TimestampRecency { threshold_seconds } => {
                    let current_time = chrono::Utc::now().timestamp();
                    if current_time - record.timestamp > *threshold_seconds {
                        return Err(ProcessingError::ValidationFailed(
                            "Timestamp is too old".to_string()
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn transform(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        if !transformed.values.is_empty() {
            let sum: f64 = transformed.values.iter().sum();
            let mean = sum / transformed.values.len() as f64;
            
            transformed.metadata.insert(
                "mean_value".to_string(),
                format!("{:.4}", mean)
            );
            
            transformed.metadata.insert(
                "processed_at".to_string(),
                chrono::Utc::now().to_rfc3339()
            );
        }
        
        transformed.values = transformed.values
            .iter()
            .map(|&v| v * 1.05)
            .collect();
        
        Ok(transformed)
    }
}

pub fn create_default_config() -> ProcessingConfig {
    ProcessingConfig {
        max_values: 100,
        require_timestamp: true,
        validation_rules: vec![
            ValidationRule::ValueRange { min: 0.0, max: 1000.0 },
            ValidationRule::NonEmptyMetadata,
            ValidationRule::TimestampRecency { threshold_seconds: 86400 },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let config = create_default_config();
        let processor = DataProcessor::new(config);
        
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            timestamp: chrono::Utc::now().timestamp(),
            values: vec![10.0, 20.0, 30.0],
            metadata,
        };
        
        let result = processor.process(&record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.values.len(), 3);
        assert!(processed.metadata.contains_key("mean_value"));
        assert!(processed.metadata.contains_key("processed_at"));
    }
    
    #[test]
    fn test_validation_failure() {
        let config = create_default_config();
        let processor = DataProcessor::new(config);
        
        let record = DataRecord {
            id: 2,
            timestamp: 0,
            values: vec![1500.0],
            metadata: HashMap::new(),
        };
        
        let result = processor.process(&record);
        assert!(result.is_err());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
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

            let category = parts[3].to_string();

            let record = DataRecord::new(id, name, value, category);
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

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap_or(std::cmp::Ordering::Equal)
        })
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
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "B".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.3,CategoryB").unwrap();
        writeln!(temp_file, "3,Item3,15.7,CategoryA").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
    }

    #[test]
    fn test_filter_and_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "A".to_string(), 10.0, "X".to_string()));
        processor.records.push(DataRecord::new(2, "B".to_string(), 20.0, "Y".to_string()));
        processor.records.push(DataRecord::new(3, "C".to_string(), 30.0, "X".to_string()));

        let filtered = processor.filter_by_category("X");
        assert_eq!(filtered.len(), 2);

        let avg = processor.calculate_average();
        assert_eq!(avg, Some(20.0));
    }
}