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
    MetadataKeyTooLong,
}

pub struct DataProcessor {
    max_metadata_key_length: usize,
}

impl DataProcessor {
    pub fn new(max_metadata_key_length: usize) -> Self {
        Self {
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
            for value in &mut record.values {
                *value = (*value - mean) / std_dev;
            }
        }
    }

    pub fn filter_records(
        &self,
        records: Vec<DataRecord>,
        predicate: impl Fn(&DataRecord) -> bool,
    ) -> Vec<DataRecord> {
        records.into_iter().filter(predicate).collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let all_values: Vec<f64> = records
            .iter()
            .flat_map(|r| r.values.clone())
            .collect();

        let count = all_values.len() as f64;
        let sum: f64 = all_values.iter().sum();
        let mean = sum / count;

        let variance: f64 = all_values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let sorted_values = {
            let mut sorted = all_values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted
        };

        let median = if count as usize % 2 == 0 {
            let mid = count as usize / 2;
            (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
        } else {
            sorted_values[count as usize / 2]
        };

        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("median".to_string(), median);
        stats.insert("min".to_string(), *sorted_values.first().unwrap_or(&0.0));
        stats.insert("max".to_string(), *sorted_values.last().unwrap_or(&0.0));

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = DataProcessor::new(50);
        let mut valid_record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::from([("key".to_string(), "value".to_string())]),
        };

        assert!(processor.validate_record(&valid_record).is_ok());

        valid_record.id = 0;
        assert_eq!(
            processor.validate_record(&valid_record),
            Err(ValidationError::InvalidId)
        );
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(50);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        processor.normalize_values(&mut record);
        
        let expected_mean = 0.0;
        let actual_mean: f64 = record.values.iter().sum::<f64>() / record.values.len() as f64;
        assert!(actual_mean.abs() < 1e-10);
        
        let variance: f64 = record.values
            .iter()
            .map(|&x| (x - expected_mean).powi(2))
            .sum::<f64>() / record.values.len() as f64;
        
        assert!((variance - 1.0).abs() < 1e-10);
    }
}