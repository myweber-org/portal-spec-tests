use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    EmptyValues,
    ValueOutOfRange(f64),
    MissingMetadata(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::EmptyValues => write!(f, "Record contains no values"),
            DataError::ValueOutOfRange(val) => write!(f, "Value {} is out of acceptable range", val),
            DataError::MissingMetadata(key) => write!(f, "Missing required metadata: {}", key),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if values.is_empty() {
            return Err(DataError::EmptyValues);
        }
        
        Ok(Self {
            id,
            values,
            metadata: HashMap::new(),
        })
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    pub fn validate(&self) -> Result<(), DataError> {
        for &value in &self.values {
            if !value.is_finite() || value < 0.0 || value > 1000.0 {
                return Err(DataError::ValueOutOfRange(value));
            }
        }
        
        if !self.metadata.contains_key("source") {
            return Err(DataError::MissingMetadata("source".to_string()));
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, factor: f64) {
        for value in &mut self.values {
            *value *= factor;
        }
    }
    
    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.values.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

pub fn process_records(records: &mut [DataRecord], factor: f64) -> Result<Vec<(u32, f64)>, DataError> {
    let mut results = Vec::new();
    
    for record in records {
        record.validate()?;
        record.transform(factor);
        
        let (mean, _, _) = record.calculate_statistics();
        results.push((record.id, mean));
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, vec![10.0, 20.0, 30.0]).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.values, vec![10.0, 20.0, 30.0]);
    }
    
    #[test]
    fn test_invalid_id() {
        let result = DataRecord::new(0, vec![10.0]);
        assert!(matches!(result, Err(DataError::InvalidId)));
    }
    
    #[test]
    fn test_empty_values() {
        let result = DataRecord::new(1, vec![]);
        assert!(matches!(result, Err(DataError::EmptyValues)));
    }
    
    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, vec![2.0, 4.0, 6.0]).unwrap();
        record.add_metadata("source".to_string(), "test".to_string());
        
        let (mean, variance, std_dev) = record.calculate_statistics();
        assert_eq!(mean, 4.0);
        assert_eq!(variance, 8.0 / 3.0);
        assert!((std_dev - (8.0 / 3.0).sqrt()).abs() < 1e-10);
    }
    
    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, vec![100.0, 200.0]).unwrap();
        assert!(record.validate().is_err());
        
        record.add_metadata("source".to_string(), "test".to_string());
        record.values = vec![100.0, 200.0];
        assert!(record.validate().is_ok());
    }
}