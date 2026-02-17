use csv::Reader;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_csv_data(input_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    let std_dev = variance.sqrt();

    (sum, mean, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}
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
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    config: ProcessingConfig,
}

pub struct ProcessingConfig {
    pub max_value_count: usize,
    pub allowed_keys: Vec<String>,
    pub timestamp_range: (i64, i64),
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.config.max_value_count {
            return Err(ProcessingError::ValidationError(
                format!("Too many values: {}", record.values.len())
            ));
        }

        if record.timestamp < self.config.timestamp_range.0 
            || record.timestamp > self.config.timestamp_range.1 {
            return Err(ProcessingError::ValidationError(
                format!("Timestamp out of range: {}", record.timestamp)
            ));
        }

        for key in record.metadata.keys() {
            if !self.config.allowed_keys.contains(key) {
                return Err(ProcessingError::ValidationError(
                    format!("Invalid metadata key: {}", key)
                ));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::InvalidData("Empty values array".to_string()));
        }

        let sum: f64 = record.values.iter().sum();
        if sum == 0.0 {
            return Err(ProcessingError::TransformationError(
                "Cannot normalize zero-sum values".to_string()
            ));
        }

        for value in record.values.iter_mut() {
            *value /= sum;
        }

        Ok(())
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;
        self.normalize_values(&mut record)?;
        
        record.metadata.insert(
            "processed_timestamp".to_string(),
            chrono::Utc::now().timestamp().to_string()
        );

        Ok(record)
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>
    ) -> Result<Vec<DataRecord>, Vec<(usize, ProcessingError)>> {
        let mut processed = Vec::new();
        let mut errors = Vec::new();

        for (index, record) in records.into_iter().enumerate() {
            match self.process_record(record) {
                Ok(processed_record) => processed.push(processed_record),
                Err(err) => errors.push((index, err)),
            }
        }

        if errors.is_empty() {
            Ok(processed)
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ProcessingConfig {
        ProcessingConfig {
            max_value_count: 10,
            allowed_keys: vec!["source".to_string(), "type".to_string()],
            timestamp_range: (0, 1000000000),
        }
    }

    fn create_test_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        metadata.insert("type".to_string(), "sample".to_string());

        DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata,
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(create_test_config());
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_too_many_values() {
        let processor = DataProcessor::new(create_test_config());
        let mut record = create_test_record();
        record.values = vec![1.0; 15];
        
        match processor.validate_record(&record) {
            Err(ProcessingError::ValidationError(msg)) => {
                assert!(msg.contains("Too many values"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(create_test_config());
        let mut record = create_test_record();
        
        assert!(processor.normalize_values(&mut record).is_ok());
        let sum: f64 = record.values.iter().sum();
        assert!((sum - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(create_test_config());
        let records = vec![create_test_record(), create_test_record()];
        
        let result = processor.batch_process(records);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}