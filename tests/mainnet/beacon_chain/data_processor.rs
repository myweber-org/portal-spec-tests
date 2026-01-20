
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
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

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
    required_metadata: Vec<String>,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64, required_metadata: Vec<String>) -> Self {
        DataProcessor {
            min_value,
            max_value,
            required_metadata,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for &value in &record.values {
            if value < self.min_value || value > self.max_value {
                return Err(DataError::ValueOutOfRange(value));
            }
        }

        for key in &self.required_metadata {
            if !record.metadata.contains_key(key) {
                return Err(DataError::MissingMetadata(key.clone()));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) {
        if let Some(max_val) = record.values.iter().cloned().reduce(f64::max) {
            if max_val != 0.0 {
                for value in &mut record.values {
                    *value /= max_val;
                }
            }
        }
    }

    pub fn process_records(&self, records: &mut [DataRecord]) -> Vec<Result<DataRecord, DataError>> {
        let mut results = Vec::new();

        for record in records {
            match self.validate_record(record) {
                Ok(_) => {
                    let mut processed_record = record.clone();
                    self.normalize_values(&mut processed_record);
                    results.push(Ok(processed_record));
                }
                Err(e) => {
                    results.push(Err(e));
                }
            }
        }

        results
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);

        let all_values: Vec<f64> = records
            .iter()
            .flat_map(|r| r.values.iter().cloned())
            .collect();

        if !all_values.is_empty() {
            let sum: f64 = all_values.iter().sum();
            let count = all_values.len() as f64;
            let mean = sum / count;

            let variance: f64 = all_values
                .iter()
                .map(|&value| {
                    let diff = mean - value;
                    diff * diff
                })
                .sum::<f64>()
                / count;

            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("min".to_string(), all_values.iter().cloned().fold(f64::INFINITY, f64::min));
            stats.insert("max".to_string(), all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(0.0, 100.0, vec!["source".to_string()]);
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let record = DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_id() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        let record = DataRecord {
            id: 0,
            values: vec![10.0],
            metadata: HashMap::new(),
        };

        assert!(matches!(processor.validate_record(&record), Err(DataError::InvalidId)));
    }

    #[test]
    fn test_normalize_values() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        let mut record = DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        processor.normalize_values(&mut record);
        assert_eq!(record.values, vec![0.3333333333333333, 0.6666666666666666, 1.0]);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        let records = vec![
            DataRecord {
                id: 1,
                values: vec![10.0, 20.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                values: vec![30.0, 40.0],
                metadata: HashMap::new(),
            },
        ];

        let stats = processor.calculate_statistics(&records);
        assert_eq!(stats.get("total_records"), Some(&2.0));
        assert_eq!(stats.get("total_values"), Some(&4.0));
        assert_eq!(stats.get("mean"), Some(&25.0));
    }
}