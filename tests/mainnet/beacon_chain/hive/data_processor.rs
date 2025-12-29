
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
        
        if self.timestamp <= 0 {
            return Err(DataError::ValidationFailed("Timestamp must be positive".into()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::ValidationFailed("Values cannot be empty".into()));
        }
        
        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationFailed("Key cannot be empty".into()));
            }
            if !value.is_finite() {
                return Err(DataError::ValidationFailed(
                    format!("Value for key '{}' must be finite", key)
                ));
            }
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), DataError> {
        if !multiplier.is_finite() || multiplier == 0.0 {
            return Err(DataError::ValidationFailed(
                "Multiplier must be non-zero finite value".into()
            ));
        }
        
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
        
        self.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| DataError::ValidationFailed("System time error".into()))?
            .as_secs() as i64;
            
        Ok(())
    }
}

pub fn process_records(
    records: Vec<DataRecord>,
    multiplier: f64,
) -> Result<Vec<DataRecord>, DataError> {
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
        values.insert("temperature".into(), 23.5);
        
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values,
            tags: vec!["sensor".into(), "room1".into()],
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_record() {
        let values = HashMap::new();
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values,
            tags: vec![],
        };
        
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_transform() {
        let mut values = HashMap::new();
        values.insert("value".into(), 10.0);
        
        let mut record = DataRecord {
            id: 1,
            timestamp: 1000,
            values,
            tags: vec![],
        };
        
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.values.get("value").unwrap(), &20.0);
    }
}