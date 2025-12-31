
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Processing timeout")]
    Timeout,
    #[error("Serialization error")]
    Serialization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidInput("ID cannot be zero".to_string()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::InvalidInput("Timestamp cannot be negative".to_string()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::InvalidInput("Values cannot be empty".to_string()));
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) {
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<(), DataError> {
    for record in records.iter_mut() {
        record.validate()?;
        record.transform(multiplier);
        record.add_tag("processed".to_string());
    }
    Ok(())
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }
    
    for key in records[0].values.keys() {
        let values: Vec<f64> = records
            .iter()
            .filter_map(|r| r.values.get(key))
            .copied()
            .collect();
        
        if !values.is_empty() {
            let count = values.len() as f64;
            let sum: f64 = values.iter().sum();
            let avg = sum / count;
            let variance: f64 = values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / count;
            
            stats.insert(format!("{}_avg", key), avg);
            stats.insert(format!("{}_variance", key), variance);
        }
    }
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 0,
            timestamp: 1234567890,
            values: HashMap::from([("temp".to_string(), 25.5)]),
            tags: vec![],
        };
        
        assert!(record.validate().is_err());
        
        record.id = 1;
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("value".to_string(), 10.0)]),
            tags: vec![],
        };
        
        record.transform(2.5);
        assert_eq!(record.values.get("value"), Some(&25.0));
    }
    
    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord {
                id: 1,
                timestamp: 1234567890,
                values: HashMap::from([("data".to_string(), 2.0)]),
                tags: vec![],
            },
            DataRecord {
                id: 2,
                timestamp: 1234567891,
                values: HashMap::from([("data".to_string(), 4.0)]),
                tags: vec![],
            },
        ];
        
        let result = process_records(&mut records, 3.0);
        assert!(result.is_ok());
        assert!(records[0].tags.contains(&"processed".to_string()));
        assert_eq!(records[0].values.get("data"), Some(&6.0));
    }
}