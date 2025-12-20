
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64, values: Vec<f64>) -> Self {
        Self {
            id,
            timestamp,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        if self.timestamp < 0 {
            return Err("Invalid timestamp".into());
        }
        if self.values.is_empty() {
            return Err("Empty values array".into());
        }
        Ok(())
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|record| record.validate().is_ok())
        .map(|mut record| {
            let normalized_values: Vec<f64> = record
                .values
                .iter()
                .map(|&value| {
                    if value.is_nan() {
                        0.0
                    } else {
                        value.clamp(0.0, 100.0)
                    }
                })
                .collect();
            record.values = normalized_values;
            record
        })
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }

    let total_values: usize = records.iter().map(|r| r.values.len()).sum();
    let all_values: Vec<f64> = records.iter().flat_map(|r| r.values.clone()).collect();

    let sum: f64 = all_values.iter().sum();
    let count = all_values.len() as f64;
    
    if count > 0.0 {
        let mean = sum / count;
        stats.insert("mean".to_string(), mean);
        
        let variance: f64 = all_values.iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());
    }

    stats.insert("total_records".to_string(), records.len() as f64);
    stats.insert("total_values".to_string(), total_values as f64);
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1625097600, vec![10.0, 20.0, 30.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1625097600, vec![10.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 1625097600, vec![150.0, -10.0, f64::NAN]),
            DataRecord::new(2, 1625097600, vec![50.0, 75.0, 25.0]),
        ];
        
        let processed = process_records(records);
        assert_eq!(processed.len(), 2);
        
        for record in processed {
            for &value in &record.values {
                assert!(value >= 0.0 && value <= 100.0);
                assert!(!value.is_nan());
            }
        }
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 1625097600, vec![10.0, 20.0]),
            DataRecord::new(2, 1625097600, vec![30.0, 40.0]),
        ];
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats["total_records"], 2.0);
        assert_eq!(stats["total_values"], 4.0);
        assert_eq!(stats["mean"], 25.0);
    }
}use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
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
                return Err(ProcessingError::ValidationError(format!(
                    "Value for key '{}' must be finite",
                    key
                )));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&mut self) -> Result<(), ProcessingError> {
        let sum: f64 = self.values.values().sum();
        if sum == 0.0 {
            return Err(ProcessingError::TransformationFailed(
                "Cannot normalize zero sum".to_string(),
            ));
        }

        for value in self.values.values_mut() {
            *value /= sum;
        }

        Ok(())
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records.iter_mut() {
        record.validate()?;
        record.normalize_values()?;
        record.add_tag("processed".to_string());
        processed.push(record.clone());
    }

    Ok(processed)
}

pub fn filter_by_tag(records: &[DataRecord], tag: &str) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|r| r.tags.contains(&tag.to_string()))
        .cloned()
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, (f64, f64, f64)> {
    let mut stats = HashMap::new();

    for record in records {
        for (key, value) in &record.values {
            let entry = stats.entry(key.clone()).or_insert((f64::MAX, f64::MIN, 0.0, 0));
            entry.0 = entry.0.min(*value);
            entry.1 = entry.1.max(*value);
            entry.2 += *value;
            entry.3 += 1;
        }
    }

    stats
        .into_iter()
        .map(|(key, (min, max, sum, count))| {
            let avg = sum / count as f64;
            (key, (min, max, avg))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("temp".to_string(), 25.5)]),
            tags: vec!["test".to_string()],
        };

        assert!(record.validate().is_ok());

        record.id = 0;
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_normalize_values() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([
                ("a".to_string(), 10.0),
                ("b".to_string(), 20.0),
                ("c".to_string(), 30.0),
            ]),
            tags: vec![],
        };

        record.normalize_values().unwrap();
        let sum: f64 = record.values.values().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord {
                id: 1,
                timestamp: 1234567890,
                values: HashMap::from([("x".to_string(), 5.0)]),
                tags: vec![],
            },
            DataRecord {
                id: 2,
                timestamp: 1234567891,
                values: HashMap::from([("y".to_string(), 10.0)]),
                tags: vec![],
            },
        ];

        let processed = process_records(&mut records).unwrap();
        assert_eq!(processed.len(), 2);
        assert!(processed[0].tags.contains(&"processed".to_string()));
    }
}