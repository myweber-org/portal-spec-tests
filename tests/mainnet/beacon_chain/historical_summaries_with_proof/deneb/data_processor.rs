use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Transformation failed")]
    TransformationFailed,
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub metadata: Option<HashMap<String, String>>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationError("Timestamp cannot be negative".to_string()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::ValidationError("Values cannot be empty".to_string()));
        }
        
        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationError("Key cannot be empty".to_string()));
            }
            
            if !value.is_finite() {
                return Err(DataError::ValidationError(
                    format!("Value for key '{}' must be finite", key)
                ));
            }
        }
        
        Ok(())
    }
    
    pub fn transform_values<F>(&mut self, transformer: F) -> Result<(), DataError>
    where
        F: Fn(f64) -> Result<f64, DataError>,
    {
        let mut transformed = HashMap::new();
        
        for (key, value) in &self.values {
            match transformer(*value) {
                Ok(transformed_value) => {
                    transformed.insert(key.clone(), transformed_value);
                }
                Err(e) => {
                    return Err(DataError::TransformationFailed);
                }
            }
        }
        
        self.values = transformed;
        Ok(())
    }
    
    pub fn normalize(&mut self) -> Result<(), DataError> {
        if self.values.is_empty() {
            return Ok(());
        }
        
        let sum: f64 = self.values.values().sum();
        if sum.abs() < f64::EPSILON {
            return Err(DataError::TransformationFailed);
        }
        
        let mut normalized = HashMap::new();
        for (key, value) in &self.values {
            normalized.insert(key.clone(), value / sum);
        }
        
        self.values = normalized;
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::new();
    
    for record in records {
        match record.validate() {
            Ok(_) => {
                let mut processed_record = record.clone();
                
                processed_record.transform_values(|x| {
                    if x < 0.0 {
                        Err(DataError::TransformationFailed)
                    } else {
                        Ok(x.ln())
                    }
                })?;
                
                processed_record.normalize()?;
                processed.push(processed_record);
            }
            Err(e) => {
                return Err(e);
            }
        }
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
            metadata: None,
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_record() {
        let mut values = HashMap::new();
        values.insert("".to_string(), f64::NAN);
        
        let record = DataRecord {
            id: 0,
            timestamp: -1,
            values,
            metadata: None,
        };
        
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_normalization() {
        let mut values = HashMap::new();
        values.insert("a".to_string(), 1.0);
        values.insert("b".to_string(), 2.0);
        values.insert("c".to_string(), 3.0);
        
        let mut record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values,
            metadata: None,
        };
        
        assert!(record.normalize().is_ok());
        let sum: f64 = record.values.values().sum();
        assert!((sum - 1.0).abs() < f64::EPSILON);
    }
}