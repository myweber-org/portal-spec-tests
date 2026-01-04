
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
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationFailed("ID cannot be zero".into()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed("Timestamp cannot be negative".into()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::ValidationFailed("Values cannot be empty".into()));
        }
        
        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationFailed("Key cannot be empty".into()));
            }
            if !value.is_finite() {
                return Err(DataError::ValidationFailed(format!("Value for {} is not finite", key)));
            }
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), DataError> {
        if !multiplier.is_finite() || multiplier == 0.0 {
            return Err(DataError::ValidationFailed("Invalid multiplier".into()));
        }
        
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
        
        self.timestamp += 1;
        Ok(())
    }
    
    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.values.is_empty() {
            return stats;
        }
        
        let values: Vec<f64> = self.values.values().copied().collect();
        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("count".into(), count);
        stats.insert("sum".into(), sum);
        stats.insert("mean".into(), mean);
        stats.insert("variance".into(), variance);
        stats.insert("std_dev".into(), variance.sqrt());
        
        if let Some(max) = values.iter().copied().reduce(f64::max) {
            stats.insert("max".into(), max);
        }
        
        if let Some(min) = values.iter().copied().reduce(f64::min) {
            stats.insert("min".into(), min);
        }
        
        stats
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for mut record in records {
        record.validate()?;
        record.transform(2.0)?;
        processed.push(record);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: HashMap::from([("temp".into(), 25.5)]),
            tags: vec!["sensor".into()],
        };
        
        assert!(record.validate().is_ok());
        
        record.id = 0;
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: HashMap::from([("temp".into(), 25.5)]),
            tags: vec![],
        };
        
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.values.get("temp"), Some(&51.0));
        assert_eq!(record.timestamp, 1001);
    }
    
    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: HashMap::from([
                ("a".into(), 1.0),
                ("b".into(), 2.0),
                ("c".into(), 3.0),
            ]),
            tags: vec![],
        };
        
        let stats = record.calculate_statistics();
        assert_eq!(stats.get("count"), Some(&3.0));
        assert_eq!(stats.get("sum"), Some(&6.0));
        assert_eq!(stats.get("mean"), Some(&2.0));
    }
}