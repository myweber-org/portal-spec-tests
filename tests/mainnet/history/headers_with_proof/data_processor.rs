
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    OutOfRange(String),
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

    pub fn add_value(&mut self, key: &str, value: f64) -> Result<(), DataError> {
        if !value.is_finite() {
            return Err(DataError::InvalidFormat);
        }
        self.values.insert(key.to_string(), value);
        Ok(())
    }

    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidFormat);
        }

        if self.timestamp < 0 {
            return Err(DataError::OutOfRange("timestamp".to_string()));
        }

        if self.values.is_empty() {
            return Err(DataError::MissingField("values".to_string()));
        }

        Ok(())
    }

    pub fn transform(&self, multiplier: f64) -> HashMap<String, f64> {
        self.values
            .iter()
            .map(|(k, v)| (k.clone(), v * multiplier))
            .collect()
    }
}

pub fn process_records(records: &[DataRecord]) -> Result<Vec<HashMap<String, f64>>, DataError> {
    let mut results = Vec::new();

    for record in records {
        record.validate()?;
        let transformed = record.transform(2.0);
        results.push(transformed);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let mut record = DataRecord::new(1, 1625097600);
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1625097600);
    }

    #[test]
    fn test_add_value() {
        let mut record = DataRecord::new(1, 1625097600);
        assert!(record.add_value("temperature", 25.5).is_ok());
        assert_eq!(record.values.get("temperature"), Some(&25.5));
    }

    #[test]
    fn test_validate_record() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("pressure", 1013.25).unwrap();
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_transform_values() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("value", 10.0).unwrap();
        let transformed = record.transform(3.0);
        assert_eq!(transformed.get("value"), Some(&30.0));
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: String,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }

    pub fn validate_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.id != 0 && !record.timestamp.is_empty())
            .collect()
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
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,timestamp").unwrap();
        writeln!(temp_file, "1,42.5,2023-10-01T12:00:00").unwrap();
        writeln!(temp_file, "2,37.8,2023-10-01T12:05:00").unwrap();
        writeln!(temp_file, "3,45.2,2023-10-01T12:10:00").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 41.83333).abs() < 0.0001);

        let filtered = processor.filter_by_threshold(40.0);
        assert_eq!(filtered.len(), 2);

        let valid = processor.validate_records();
        assert_eq!(valid.len(), 3);
    }
}