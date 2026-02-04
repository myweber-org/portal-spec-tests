use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
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
            if parts.len() != 3 {
                continue;
            }

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].trim();

            let record = DataRecord::new(id, value, category);
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.valid)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.valid {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record);
            }
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "A");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "A");
        assert!(record.valid);
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -5.0, "");
        assert!(!record.valid);
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,20.3,TypeB").unwrap();
        writeln!(temp_file, "3,-5.0,TypeC").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A"));
        processor.records.push(DataRecord::new(2, 20.0, "B"));
        processor.records.push(DataRecord::new(3, 30.0, "C"));

        let average = processor.calculate_average();
        assert_eq!(average, Some(20.0));
    }

    #[test]
    fn test_empty_average() {
        let processor = DataProcessor::new();
        let average = processor.calculate_average();
        assert_eq!(average, None);
    }

    #[test]
    fn test_grouping() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "Group1"));
        processor.records.push(DataRecord::new(2, 20.0, "Group1"));
        processor.records.push(DataRecord::new(3, 30.0, "Group2"));

        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("Group1").unwrap().len(), 2);
        assert_eq!(groups.get("Group2").unwrap().len(), 1);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ProcessingError {
    details: String,
}

impl ProcessingError {
    fn new(msg: &str) -> ProcessingError {
        ProcessingError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ProcessingError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<DataRecord, ProcessingError> {
        if value < 0.0 {
            return Err(ProcessingError::new("Value cannot be negative"));
        }
        if timestamp < 0 {
            return Err(ProcessingError::new("Timestamp cannot be negative"));
        }
        Ok(DataRecord {
            id,
            value,
            timestamp,
        })
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<f64> {
    records
        .into_iter()
        .filter(|r| r.value > 10.0)
        .map(|r| r.value * 1.5)
        .collect()
}

pub fn calculate_statistics(values: &[f64]) -> (f64, f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = values.iter().sum();
    let count = values.len() as f64;
    let mean = sum / count;

    let variance: f64 = values
        .iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>()
        / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 25.5, 1625097600).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 25.5);
        assert_eq!(record.timestamp, 1625097600);
    }

    #[test]
    fn test_invalid_record_negative_value() {
        let result = DataRecord::new(1, -5.0, 1625097600);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 5.0, 1000).unwrap(),
            DataRecord::new(2, 15.0, 2000).unwrap(),
            DataRecord::new(3, 25.0, 3000).unwrap(),
        ];

        let processed = process_records(records);
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0], 22.5);
        assert_eq!(processed[1], 37.5);
    }

    #[test]
    fn test_calculate_statistics() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let (mean, variance, std_dev) = calculate_statistics(&values);
        
        assert!((mean - 5.0).abs() < 0.001);
        assert!((variance - 4.0).abs() < 0.001);
        assert!((std_dev - 2.0).abs() < 0.001);
    }
}
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue(f64),
    InvalidTimestamp(i64),
    EmptyCategory,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            ProcessingError::InvalidTimestamp(t) => write!(f, "Invalid timestamp: {}", t),
            ProcessingError::EmptyCategory => write!(f, "Category cannot be empty"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64) -> Self {
        DataProcessor { min_value, max_value }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < self.min_value || record.value > self.max_value {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp(record.timestamp));
        }

        if record.category.trim().is_empty() {
            return Err(ProcessingError::EmptyCategory);
        }

        Ok(())
    }

    pub fn normalize_value(&self, record: &DataRecord) -> f64 {
        (record.value - self.min_value) / (self.max_value - self.min_value)
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<(u64, f64)>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());

        for record in records {
            self.validate_record(&record)?;
            let normalized = self.normalize_value(&record);
            results.push((record.id, normalized));
        }

        if results.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "No valid records to process".to_string(),
            ));
        }

        Ok(results)
    }

    pub fn filter_by_category(
        records: Vec<DataRecord>,
        category_filter: &str,
    ) -> Vec<DataRecord> {
        records
            .into_iter()
            .filter(|r| r.category == category_filter)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record_valid() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1234567890,
            category: "test".to_string(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_record_invalid_value() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1234567890,
            category: "test".to_string(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_normalize_value() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 75.0,
            timestamp: 1234567890,
            category: "test".to_string(),
        };

        let normalized = processor.normalize_value(&record);
        assert!((normalized - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 1000,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                value: 20.0,
                timestamp: 2000,
                category: "B".to_string(),
            },
            DataRecord {
                id: 3,
                value: 30.0,
                timestamp: 3000,
                category: "A".to_string(),
            },
        ];

        let filtered = DataProcessor::filter_by_category(records, "A");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }
}