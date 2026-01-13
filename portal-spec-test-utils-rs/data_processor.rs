
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64, values: Vec<f64>) -> Self {
        Self {
            id,
            timestamp,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationError("ID cannot be zero".to_string()));
        }

        if self.timestamp < 0 {
            return Err(ProcessingError::ValidationError("Timestamp cannot be negative".to_string()));
        }

        if self.values.is_empty() {
            return Err(ProcessingError::ValidationError("Values cannot be empty".to_string()));
        }

        for (i, &value) in self.values.iter().enumerate() {
            if !value.is_finite() {
                return Err(ProcessingError::ValidationError(
                    format!("Value at index {} is not finite", i)
                ));
            }
        }

        Ok(())
    }

    pub fn normalize(&mut self) -> Result<(), ProcessingError> {
        if self.values.is_empty() {
            return Err(ProcessingError::TransformationFailed("Cannot normalize empty values".to_string()));
        }

        let mean = self.values.iter().sum::<f64>() / self.values.len() as f64;
        let variance = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / self.values.len() as f64;
        
        if variance.abs() < 1e-10 {
            return Err(ProcessingError::TransformationFailed("Variance too small for normalization".to_string()));
        }

        let std_dev = variance.sqrt();
        
        for value in &mut self.values {
            *value = (*value - mean) / std_dev;
        }

        self.metadata.insert("normalized".to_string(), "true".to_string());
        self.metadata.insert("original_mean".to_string(), mean.to_string());
        self.metadata.insert("original_std_dev".to_string(), std_dev.to_string());

        Ok(())
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.values.is_empty() {
            return stats;
        }

        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;
        
        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        let variance = self.values.iter()
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

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::new();
    
    for record in records {
        record.validate()?;
        
        let mut processed_record = record.clone();
        processed_record.normalize()?;
        
        processed.push(processed_record);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(record.normalize().is_ok());
        
        let stats = record.get_statistics();
        let mean = stats.get("mean").unwrap();
        let std_dev = stats.get("std_dev").unwrap();
        
        assert!(mean.abs() < 1e-10);
        assert!((std_dev - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_statistics() {
        let record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        let stats = record.get_statistics();
        
        assert_eq!(stats.get("count"), Some(&3.0));
        assert_eq!(stats.get("sum"), Some(&6.0));
        assert_eq!(stats.get("mean"), Some(&2.0));
        assert_eq!(stats.get("min"), Some(&1.0));
        assert_eq!(stats.get("max"), Some(&3.0));
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, key: &str, values: Vec<f64>) -> Result<(), ValidationError> {
        if values.is_empty() {
            return Err(ValidationError {
                message: "Dataset cannot be empty".to_string(),
            });
        }

        for value in &values {
            if value.is_nan() || value.is_infinite() {
                return Err(ValidationError {
                    message: "Dataset contains invalid numeric values".to_string(),
                });
            }
        }

        self.data.insert(key.to_string(), values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            
            let std_dev = variance.sqrt();

            Statistics {
                count,
                sum,
                mean,
                variance,
                std_dev,
            }
        })
    }

    pub fn normalize_data(&self, key: &str) -> Option<Vec<f64>> {
        self.calculate_statistics(key).map(|stats| {
            self.data[key].iter()
                .map(|&x| (x - stats.mean) / stats.std_dev)
                .collect()
        })
    }

    pub fn merge_datasets(&self, keys: &[&str]) -> Option<Vec<f64>> {
        let mut result = Vec::new();
        
        for key in keys {
            if let Some(values) = self.data.get(*key) {
                result.extend(values);
            } else {
                return None;
            }
        }
        
        Some(result)
    }

    pub fn dataset_count(&self) -> usize {
        self.data.len()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("test", vec![1.0, 2.0, 3.0]);
        assert!(result.is_ok());
        assert_eq!(processor.dataset_count(), 1);
    }

    #[test]
    fn test_add_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("empty", vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("numbers", vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("numbers").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.sum, 15.0);
        assert_eq!(stats.mean, 3.0);
    }

    #[test]
    fn test_normalize_data() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("values", vec![1.0, 2.0, 3.0]).unwrap();
        
        let normalized = processor.normalize_data("values").unwrap();
        assert_eq!(normalized.len(), 3);
    }

    #[test]
    fn test_merge_datasets() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("set1", vec![1.0, 2.0]).unwrap();
        processor.add_dataset("set2", vec![3.0, 4.0]).unwrap();
        
        let merged = processor.merge_datasets(&["set1", "set2"]).unwrap();
        assert_eq!(merged, vec![1.0, 2.0, 3.0, 4.0]);
    }
}