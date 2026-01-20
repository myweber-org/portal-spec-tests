
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Transformation failed: {0}")]
    TransformationFailed(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u64, value: f64, timestamp: i64) -> Self {
        Self {
            id,
            value,
            timestamp,
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(DataError::ValidationError("Value must be finite".to_string()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationError("Timestamp cannot be negative".to_string()));
        }
        
        Ok(())
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        
        let transformed_value = transform_value(record.value)?;
        let processed_record = DataRecord::new(
            record.id,
            transformed_value,
            record.timestamp,
        );
        
        processed.push(processed_record);
    }
    
    Ok(processed)
}

fn transform_value(value: f64) -> Result<f64, DataError> {
    if value.abs() < f64::EPSILON {
        return Err(DataError::TransformationFailed("Value too small".to_string()));
    }
    
    let result = (value * 100.0).ln();
    
    if result.is_nan() || result.is_infinite() {
        Err(DataError::TransformationFailed("Transformation produced invalid result".to_string()))
    } else {
        Ok(result)
    }
}

pub fn calculate_statistics(records: &[DataRecord]) -> Option<(f64, f64, f64)> {
    if records.is_empty() {
        return None;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    Some((mean, variance, std_dev))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 42.5, 1234567890);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 42.5, 1234567890);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 10.0, 1000),
            DataRecord::new(2, 20.0, 2000),
        ];
        
        let result = process_records(records);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 10.0, 1000),
            DataRecord::new(2, 20.0, 2000),
            DataRecord::new(3, 30.0, 3000),
        ];
        
        let stats = calculate_statistics(&records);
        assert!(stats.is_some());
        
        let (mean, variance, std_dev) = stats.unwrap();
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}