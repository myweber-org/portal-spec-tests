
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    ValueOutOfRange(f64),
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.timestamp <= 0 {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        if self.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }
        
        for &value in &self.values {
            if !value.is_finite() {
                return Err(ValidationError::ValueOutOfRange(value));
            }
        }
        
        Ok(())
    }
    
    pub fn normalize_values(&mut self) {
        if self.values.is_empty() {
            return;
        }
        
        let sum: f64 = self.values.iter().sum();
        let mean = sum / self.values.len() as f64;
        
        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / self.values.len() as f64;
        
        let std_dev = variance.sqrt();
        
        if std_dev > 0.0 {
            for value in &mut self.values {
                *value = (*value - mean) / std_dev;
            }
        }
    }
    
    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.values.is_empty() {
            return stats;
        }
        
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;
        
        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());
        
        stats
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<HashMap<String, f64>, ValidationError>> {
    records
        .iter_mut()
        .map(|record| {
            record.validate()?;
            record.normalize_values();
            Ok(record.calculate_statistics())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata: HashMap::new(),
        };
        
        assert!(record.validate().is_ok());
        
        let stats = record.calculate_statistics();
        assert_eq!(stats.get("count"), Some(&5.0));
        assert_eq!(stats.get("sum"), Some(&15.0));
        assert_eq!(stats.get("mean"), Some(&3.0));
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            timestamp: 1234567890,
            values: vec![1.0, 2.0],
            metadata: HashMap::new(),
        };
        
        assert_eq!(record.validate(), Err(ValidationError::InvalidId));
    }
    
    #[test]
    fn test_normalization() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata: HashMap::new(),
        };
        
        record.normalize_values();
        
        let stats = record.calculate_statistics();
        assert!((stats.get("mean").unwrap() - 0.0).abs() < 1e-10);
        assert!((stats.get("std_dev").unwrap() - 1.0).abs() < 1e-10);
    }
}