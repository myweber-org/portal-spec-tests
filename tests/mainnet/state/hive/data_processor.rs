
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
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataSet {
    values: Vec<f64>,
}

impl DataSet {
    pub fn new() -> Self {
        DataSet { values: Vec::new() }
    }

    pub fn from_csv(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut values = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                values.push(value);
            }
        }

        Ok(DataSet { values })
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn variance(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        let mean = self.mean().unwrap();
        let sum_sq_diff: f64 = self.values.iter().map(|&x| (x - mean).powi(2)).sum();
        Some(sum_sq_diff / (self.values.len() - 1) as f64)
    }

    pub fn standard_deviation(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
    }

    pub fn min(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::min)
    }

    pub fn max(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::max)
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }
}

pub fn process_numeric_data(data: &[f64]) -> (f64, f64, f64) {
    let sum: f64 = data.iter().sum();
    let count = data.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = data.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_operations() {
        let mut dataset = DataSet::new();
        dataset.add_value(10.0);
        dataset.add_value(20.0);
        dataset.add_value(30.0);
        
        assert_eq!(dataset.mean(), Some(20.0));
        assert_eq!(dataset.variance(), Some(100.0));
        assert_eq!(dataset.standard_deviation(), Some(10.0));
        assert_eq!(dataset.min(), Some(10.0));
        assert_eq!(dataset.max(), Some(30.0));
        assert_eq!(dataset.count(), 3);
    }

    #[test]
    fn test_empty_dataset() {
        let dataset = DataSet::new();
        assert_eq!(dataset.mean(), None);
        assert_eq!(dataset.variance(), None);
        assert_eq!(dataset.standard_deviation(), None);
        assert_eq!(dataset.min(), None);
        assert_eq!(dataset.max(), None);
        assert_eq!(dataset.count(), 0);
    }

    #[test]
    fn test_process_numeric_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = process_numeric_data(&data);
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert!((std_dev - 1.4142135623730951).abs() < 1e-10);
    }
}