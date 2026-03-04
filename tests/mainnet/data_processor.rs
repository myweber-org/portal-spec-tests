
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
        record.add_tag("processed".into());
    }
    Ok(())
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, (f64, f64, f64)> {
    let mut stats = HashMap::new();
    
    for record in records {
        for (key, value) in &record.values {
            let entry = stats.entry(key.clone()).or_insert((0.0, 0.0, 0.0));
            entry.0 += value;
            entry.1 = entry.1.max(*value);
            entry.2 = if entry.2 == 0.0 { *value } else { entry.2.min(*value) };
        }
    }
    
    for (_, (sum, max, min)) in stats.iter_mut() {
        *sum = (sum * 100.0).round() / 100.0;
        *max = (max * 100.0).round() / 100.0;
        *min = (min * 100.0).round() / 100.0;
    }
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("temperature".into(), 25.5)]),
            tags: vec![],
        };
        
        assert!(record.validate().is_ok());
        
        record.id = 0;
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("value".into(), 10.0)]),
            tags: vec![],
        };
        
        record.transform(2.0);
        assert_eq!(record.values.get("value"), Some(&20.0));
    }
}