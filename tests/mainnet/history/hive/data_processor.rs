
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
    MetadataKeyTooLong,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidTimestamp => write!(f, "Timestamp cannot be negative"),
            ValidationError::EmptyValues => write!(f, "Values vector cannot be empty"),
            ValidationError::MetadataKeyTooLong => write!(f, "Metadata key exceeds maximum length"),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    max_metadata_key_length: usize,
}

impl DataProcessor {
    pub fn new(max_metadata_key_length: usize) -> Self {
        DataProcessor {
            max_metadata_key_length,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }

        if record.timestamp < 0 {
            return Err(ValidationError::InvalidTimestamp);
        }

        if record.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }

        for key in record.metadata.keys() {
            if key.len() > self.max_metadata_key_length {
                return Err(ValidationError::MetadataKeyTooLong);
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) {
        if record.values.is_empty() {
            return;
        }

        let sum: f64 = record.values.iter().sum();
        let mean = sum / record.values.len() as f64;

        let variance: f64 = record.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / record.values.len() as f64;
        
        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            for value in record.values.iter_mut() {
                *value = (*value - mean) / std_dev;
            }
        }
    }

    pub fn filter_records(
        &self,
        records: Vec<DataRecord>,
        min_timestamp: i64,
        max_timestamp: i64,
    ) -> Vec<DataRecord> {
        records
            .into_iter()
            .filter(|record| {
                record.timestamp >= min_timestamp && record.timestamp <= max_timestamp
            })
            .collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let all_values: Vec<f64> = records
            .iter()
            .flat_map(|record| record.values.clone())
            .collect();

        let count = all_values.len() as f64;
        let sum: f64 = all_values.iter().sum();
        let mean = sum / count;

        let variance: f64 = all_values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let min = all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = all_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record_valid() {
        let processor = DataProcessor::new(50);
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0],
            metadata,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_normalize_values() {
        let processor = DataProcessor::new(50);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        processor.normalize_values(&mut record);
        
        let sum: f64 = record.values.iter().sum();
        let mean = sum / record.values.len() as f64;
        
        assert!(mean.abs() < 1e-10);
    }

    #[test]
    fn test_filter_records() {
        let processor = DataProcessor::new(50);
        
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1625097600,
                values: vec![1.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 1625184000,
                values: vec![2.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 3,
                timestamp: 1625270400,
                values: vec![3.0],
                metadata: HashMap::new(),
            },
        ];

        let filtered = processor.filter_records(records, 1625097600, 1625184000);
        
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 2);
    }
}