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
        self.tags.push(tag);
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationFailed("ID cannot be zero".to_string()));
        }

        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed("Timestamp cannot be negative".to_string()));
        }

        if self.values.is_empty() {
            return Err(DataError::ValidationFailed("Values cannot be empty".to_string()));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationFailed("Key cannot be empty".to_string()));
            }
            if !value.is_finite() {
                return Err(DataError::ValidationFailed(format!("Value for {} is not finite", key)));
            }
        }

        Ok(())
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records {
        record.validate()?;
        
        let mut processed_record = record.clone();
        
        processed_record.tags.retain(|tag| !tag.is_empty());
        processed_record.tags.sort();
        processed_record.tags.dedup();

        for value in processed_record.values.values_mut() {
            *value = (*value * 100.0).round() / 100.0;
        }

        processed.push(processed_record);
    }

    processed.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();

    if records.is_empty() {
        return stats;
    }

    for record in records {
        for (key, value) in &record.values {
            let entry = stats.entry(key.clone()).or_insert(Vec::new());
            entry.push(*value);
        }
    }

    let mut result = HashMap::new();
    for (key, values) in stats {
        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        result.insert(format!("{}_mean", key), mean);
        result.insert(format!("{}_variance", key), variance);
        result.insert(format!("{}_count", key), count);
    }

    result
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
    fn test_invalid_record() {
        let record = DataRecord::new(0, 1234567890);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let mut record1 = DataRecord::new(1, 1000);
        record1.add_value("pressure".to_string(), 1013.256);
        record1.add_tag("sensor".to_string());
        record1.add_tag("sensor".to_string());

        let mut record2 = DataRecord::new(2, 900);
        record2.add_value("pressure".to_string(), 1012.789);
        record2.add_tag("".to_string());

        let records = vec![record1, record2];
        let processed = process_records(records).unwrap();

        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].timestamp, 900);
        assert_eq!(processed[0].tags.len(), 0);
        assert_eq!(processed[1].tags.len(), 1);
    }
}use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: String, value: String) -> &mut Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid record ID".to_string());
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        if self.values.is_empty() {
            return Err("Record must contain at least one value".to_string());
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.values.is_empty() {
            return stats;
        }

        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        
        if let Some(&min) = self.values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("min".to_string(), min);
        }
        
        if let Some(&max) = self.values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("max".to_string(), max);
        }

        stats
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<HashMap<String, f64>> {
    records.iter()
        .filter(|record| record.validate().is_ok())
        .map(|record| record.calculate_statistics())
        .collect()
}

pub fn transform_records(records: Vec<DataRecord>, multiplier: f64) -> Vec<DataRecord> {
    records.into_iter()
        .map(|mut record| {
            record.values = record.values.iter()
                .map(|&value| value * multiplier)
                .collect();
            record
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut valid_record = DataRecord::new(1, 1234567890);
        valid_record.add_value(42.0);
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, 1234567890);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(10.0).add_value(20.0).add_value(30.0);
        
        let stats = record.calculate_statistics();
        
        assert_eq!(stats.get("count"), Some(&3.0));
        assert_eq!(stats.get("mean"), Some(&20.0));
        assert_eq!(stats.get("min"), Some(&10.0));
        assert_eq!(stats.get("max"), Some(&30.0));
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(10.0).add_value(20.0);
        
        let transformed = transform_records(vec![record], 2.0);
        
        assert_eq!(transformed[0].values, vec![20.0, 40.0]);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    records: Vec<HashMap<String, f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let header_line = lines.next().ok_or("Empty file")??;
        let headers: Vec<String> = header_line.split(',').map(|s| s.trim().to_string()).collect();
        
        for line_result in lines {
            let line = line_result?;
            let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            
            if values.len() != headers.len() {
                continue;
            }
            
            let mut record = HashMap::new();
            for (i, header) in headers.iter().enumerate() {
                if let Ok(num) = values[i].parse::<f64>() {
                    record.insert(header.clone(), num);
                }
            }
            
            if !record.is_empty() {
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self, column: &str) -> Option<(f64, f64, f64)> {
        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record.get(column).copied())
            .collect();
        
        if values.is_empty() {
            return None;
        }
        
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        Some((mean, variance, std_dev))
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<HashMap<String, f64>>
    where
        F: Fn(&HashMap<String, f64>) -> bool,
    {
        self.records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }

    pub fn get_column_summary(&self, column: &str) -> Option<(f64, f64, f64, f64)> {
        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record.get(column).copied())
            .collect();
        
        if values.is_empty() {
            return None;
        }
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        Some((min, max, mean, count))
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
        writeln!(temp_file, "id,value,temperature").unwrap();
        writeln!(temp_file, "1,25.5,98.6").unwrap();
        writeln!(temp_file, "2,30.2,99.1").unwrap();
        writeln!(temp_file, "3,22.8,97.9").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let stats = processor.calculate_statistics("value");
        assert!(stats.is_some());
        
        let (mean, variance, std_dev) = stats.unwrap();
        assert!((mean - 26.166666666666668).abs() < 0.0001);
        assert!((variance - 13.802222222222223).abs() < 0.0001);
        
        let filtered = processor.filter_records(|record| {
            record.get("temperature").unwrap_or(&0.0) > &98.0
        });
        assert_eq!(filtered.len(), 2);
        
        let summary = processor.get_column_summary("temperature");
        assert!(summary.is_some());
        let (min, max, mean_temp, count) = summary.unwrap();
        assert!((min - 97.9).abs() < 0.0001);
        assert!((max - 99.1).abs() < 0.0001);
        assert_eq!(count, 3.0);
    }
}