use csv::Reader;
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
        DataProcessor { records: Vec::new() }
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

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn max_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    pub fn min_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).min_by(|a, b| a.partial_cmp(b).unwrap())
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let csv_data = "id,value,category\n1,10.5,A\n2,20.3,B\n3,15.7,A\n4,25.1,B\n5,18.9,A";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let mut processor = DataProcessor::new();
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.record_count(), 5);
        assert_eq!(processor.calculate_mean().unwrap(), 18.1);
        assert_eq!(processor.filter_by_category("A").len(), 3);
        assert_eq!(processor.max_value().unwrap(), 25.1);
        assert_eq!(processor.min_value().unwrap(), 10.5);
    }
}use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid input data")]
    InvalidData,
    #[error("Transformation failed: {0}")]
    TransformationFailed(String),
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

impl DataRecord {
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationError(
                "ID cannot be zero".to_string(),
            ));
        }

        if self.timestamp < 0 {
            return Err(ProcessingError::ValidationError(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        if self.values.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Values cannot be empty".to_string(),
            ));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(ProcessingError::ValidationError(
                    "Key cannot be empty".to_string(),
                ));
            }
            if !value.is_finite() {
                return Err(ProcessingError::ValidationError(
                    format!("Value for key '{}' is not finite", key),
                ));
            }
        }

        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        if multiplier <= 0.0 || !multiplier.is_finite() {
            return Err(ProcessingError::TransformationFailed(
                "Invalid multiplier".to_string(),
            ));
        }

        for value in self.values.values_mut() {
            *value *= multiplier;
        }

        self.tags.push(format!("transformed_by_{}", multiplier));
        Ok(())
    }
}

pub fn process_records(
    records: Vec<DataRecord>,
    multiplier: f64,
) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());

    for mut record in records {
        record.validate()?;
        record.transform(multiplier)?;
        processed.push(record);
    }

    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), 25.5);
        values.insert("humidity".to_string(), 60.0);

        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values,
            tags: vec!["sensor_a".to_string()],
        };

        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let values = HashMap::new();
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values,
            tags: vec![],
        };

        assert!(record.validate().is_err());
    }

    #[test]
    fn test_transform_record() {
        let mut values = HashMap::new();
        values.insert("value".to_string(), 10.0);

        let mut record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values,
            tags: vec![],
        };

        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.values.get("value").unwrap(), &20.0);
        assert!(record.tags.contains(&"transformed_by_2".to_string()));
    }
}