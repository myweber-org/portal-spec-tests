
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    ValueOutOfRange(f64),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::InvalidTimestamp => write!(f, "Timestamp cannot be negative"),
            ValidationError::EmptyValues => write!(f, "Values vector cannot be empty"),
            ValidationError::ValueOutOfRange(val) => write!(f, "Value {} is out of acceptable range", val),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.timestamp < 0 {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        if self.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }
        
        for &value in &self.values {
            if !value.is_finite() || value < 0.0 || value > 1000.0 {
                return Err(ValidationError::ValueOutOfRange(value));
            }
        }
        
        Ok(())
    }
    
    pub fn normalize_values(&mut self) {
        if let Some(max_value) = self.values.iter().copied().reduce(f64::max) {
            if max_value > 0.0 {
                for value in &mut self.values {
                    *value /= max_value;
                }
            }
        }
    }
    
    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<(u32, f64, f64, f64)>, ValidationError> {
    let mut results = Vec::new();
    
    for record in records.iter_mut() {
        record.validate()?;
        record.normalize_values();
        
        let (mean, variance, std_dev) = record.calculate_statistics();
        results.push((record.id, mean, variance, std_dev));
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };
        
        assert!(record.validate().is_ok());
        
        record.normalize_values();
        assert_eq!(record.values, vec![0.3333333333333333, 0.6666666666666666, 1.0]);
        
        let (mean, variance, std_dev) = record.calculate_statistics();
        assert!((mean - 0.6666666666666666).abs() < 1e-10);
        assert!((variance - 0.1111111111111111).abs() < 1e-10);
        assert!((std_dev - 0.3333333333333333).abs() < 1e-10);
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            timestamp: 1625097600,
            values: vec![10.0],
            metadata: HashMap::new(),
        };
        
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_process_multiple_records() {
        let mut records = vec![
            DataRecord {
                id: 1,
                timestamp: 1625097600,
                values: vec![1.0, 2.0, 3.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 1625184000,
                values: vec![4.0, 5.0, 6.0],
                metadata: HashMap::new(),
            },
        ];
        
        let results = process_records(&mut records).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 1);
        assert_eq!(results[1].0, 2);
    }
}