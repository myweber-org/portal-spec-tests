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
pub enum DataError {
    InvalidId,
    TimestampOutOfRange,
    EmptyValues,
    ValidationFailed(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::TimestampOutOfRange => write!(f, "Timestamp out of valid range"),
            DataError::EmptyValues => write!(f, "Record contains no values"),
            DataError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    max_values: usize,
    min_timestamp: i64,
    max_timestamp: i64,
}

impl DataProcessor {
    pub fn new(max_values: usize, min_timestamp: i64, max_timestamp: i64) -> Self {
        DataProcessor {
            max_values,
            min_timestamp,
            max_timestamp,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.timestamp < self.min_timestamp || record.timestamp > self.max_timestamp {
            return Err(DataError::TimestampOutOfRange);
        }

        if record.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        if record.values.len() > self.max_values {
            return Err(DataError::ValidationFailed(
                format!("Too many values: {} > {}", record.values.len(), self.max_values)
            ));
        }

        Ok(())
    }

    pub fn process_records(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, DataError> {
        let mut processed = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            
            let processed_record = DataRecord {
                id: record.id,
                timestamp: record.timestamp,
                values: self.normalize_values(&record.values),
                metadata: record.metadata,
            };
            
            processed.push(processed_record);
        }
        
        Ok(processed)
    }

    fn normalize_values(&self, values: &[f64]) -> Vec<f64> {
        if values.is_empty() {
            return Vec::new();
        }
        
        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        values.iter()
            .map(|&v| if v > 0.0 { v / mean } else { v })
            .collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }
        
        let all_values: Vec<f64> = records.iter()
            .flat_map(|r| r.values.iter())
            .copied()
            .collect();
        
        if !all_values.is_empty() {
            let count = all_values.len() as f64;
            let sum: f64 = all_values.iter().sum();
            let mean = sum / count;
            
            let variance: f64 = all_values.iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>() / count;
            
            stats.insert("total_records".to_string(), records.len() as f64);
            stats.insert("total_values".to_string(), count);
            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("std_dev".to_string(), variance.sqrt());
        }
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation_valid_record() {
        let processor = DataProcessor::new(10, 0, 1000);
        let record = DataRecord {
            id: 1,
            timestamp: 500,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validation_invalid_id() {
        let processor = DataProcessor::new(10, 0, 1000);
        let record = DataRecord {
            id: 0,
            timestamp: 500,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(matches!(processor.validate_record(&record), Err(DataError::InvalidId)));
    }
    
    #[test]
    fn test_normalize_values() {
        let processor = DataProcessor::new(10, 0, 1000);
        let values = vec![1.0, 2.0, 3.0];
        let normalized = processor.normalize_values(&values);
        
        assert_eq!(normalized.len(), 3);
        assert!((normalized[0] - 0.5).abs() < 0.001);
        assert!((normalized[1] - 1.0).abs() < 0.001);
        assert!((normalized[2] - 1.5).abs() < 0.001);
    }
}