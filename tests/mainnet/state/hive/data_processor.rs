
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    ValidationError(String),
    #[error("Transformation failed: {0}")]
    TransformationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
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

pub fn process_records(records: &[DataRecord]) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        
        let mut processed_record = record.clone();
        
        if processed_record.value < 0.0 {
            processed_record.value = processed_record.value.abs();
        }
        
        if processed_record.timestamp == 0 {
            processed_record.timestamp = chrono::Utc::now().timestamp();
        }
        
        processed.push(processed_record);
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> Result<(f64, f64, f64), DataError> {
    if records.is_empty() {
        return Err(DataError::TransformationError("No records provided".to_string()));
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    Ok((mean, variance, std_dev))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1234567890,
        };
        
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord {
            id: 0,
            value: f64::NAN,
            timestamp: -1,
        };
        
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord { id: 1, value: -10.0, timestamp: 0 },
            DataRecord { id: 2, value: 20.0, timestamp: 1000 },
        ];
        
        let processed = process_records(&records).unwrap();
        assert_eq!(processed[0].value, 10.0);
        assert!(processed[0].timestamp > 0);
        assert_eq!(processed[1].value, 20.0);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord { id: 1, value: 10.0, timestamp: 1 },
            DataRecord { id: 2, value: 20.0, timestamp: 2 },
            DataRecord { id: 3, value: 30.0, timestamp: 3 },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records).unwrap();
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}