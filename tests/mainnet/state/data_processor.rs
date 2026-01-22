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

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_number == 0 && self.has_header {
                continue;
            }

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();

            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String], expected_fields: usize) -> bool {
        record.len() == expected_fields && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        let values: Vec<f64> = records
            .iter()
            .filter_map(|record| record.get(column_index).and_then(|s| s.parse::<f64>().ok()))
            .collect();

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
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
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "50000"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["test".to_string(), "123".to_string()];
        let invalid_record = vec!["".to_string(), "data".to_string()];

        assert!(processor.validate_record(&valid_record, 2));
        assert!(!processor.validate_record(&invalid_record, 2));
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(',', false);
        let records = vec![
            vec!["10.5".to_string()],
            vec!["20.0".to_string()],
            vec!["15.5".to_string()],
        ];

        let stats = processor.calculate_statistics(&records, 0).unwrap();
        assert!((stats.0 - 15.333).abs() < 0.001);
        assert!((stats.2 - 4.041).abs() < 0.001);
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.3,category_b").unwrap();
        writeln!(temp_file, "3,15.7,category_a").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert!(stats.0 > 0.0);
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
    }
}use serde::{Deserialize, Serialize};
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
pub enum DataError {
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(i64),
    #[error("Empty values array")]
    EmptyValues,
    #[error("Value out of range: {0}")]
    ValueOutOfRange(f64),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
    max_values_count: usize,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64, max_values_count: usize) -> Self {
        Self {
            min_value,
            max_value,
            max_values_count,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.timestamp < 0 {
            return Err(DataError::InvalidTimestamp(record.timestamp));
        }

        if record.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for &value in &record.values {
            if value < self.min_value || value > self.max_value {
                return Err(DataError::ValueOutOfRange(value));
            }
        }

        if record.values.len() > self.max_values_count {
            return Err(DataError::ValueOutOfRange(record.values.len() as f64));
        }

        Ok(())
    }

    pub fn normalize_values(&self, values: &[f64]) -> Vec<f64> {
        if values.is_empty() {
            return Vec::new();
        }

        let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max_val - min_val).abs() < f64::EPSILON {
            return vec![0.5; values.len()];
        }

        values
            .iter()
            .map(|&v| (v - min_val) / (max_val - min_val))
            .collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

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
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>() / count;
            
            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("min".to_string(), all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
            stats.insert("max".to_string(), all_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
            stats.insert("count".to_string(), count);
        }

        stats
    }

    pub fn filter_records<F>(&self, records: Vec<DataRecord>, predicate: F) -> Vec<DataRecord>
    where
        F: Fn(&DataRecord) -> bool,
    {
        records.into_iter().filter(predicate).collect()
    }

    pub fn merge_metadata(&self, records: &mut [DataRecord], additional_metadata: HashMap<String, String>) {
        for record in records {
            for (key, value) in &additional_metadata {
                record.metadata.insert(key.clone(), value.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record_valid() {
        let processor = DataProcessor::new(0.0, 100.0, 10);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_normalize_values() {
        let processor = DataProcessor::new(0.0, 100.0, 10);
        let values = vec![0.0, 50.0, 100.0];
        let normalized = processor.normalize_values(&values);
        
        assert_eq!(normalized, vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(0.0, 100.0, 10);
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: vec![10.0, 20.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: vec![30.0, 40.0],
                metadata: HashMap::new(),
            },
        ];
        
        let stats = processor.calculate_statistics(&records);
        
        assert_eq!(stats.get("mean"), Some(&25.0));
        assert_eq!(stats.get("count"), Some(&4.0));
    }
}