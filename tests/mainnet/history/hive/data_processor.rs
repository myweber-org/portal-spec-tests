
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
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub category: String,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue(f64),
    InvalidCategory(String),
    TimestampOutOfRange(i64),
    SerializationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            ProcessingError::InvalidCategory(c) => write!(f, "Invalid category: {}", c),
            ProcessingError::TimestampOutOfRange(t) => write!(f, "Timestamp out of range: {}", t),
            ProcessingError::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
    allowed_categories: Vec<String>,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64, allowed_categories: Vec<String>) -> Self {
        DataProcessor {
            min_value,
            max_value,
            allowed_categories,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < self.min_value || record.value > self.max_value {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if !self.allowed_categories.contains(&record.category) {
            return Err(ProcessingError::InvalidCategory(record.category.clone()));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::TimestampOutOfRange(record.timestamp));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> DataRecord {
        DataRecord {
            id: record.id,
            value: (record.value * 100.0).round() / 100.0,
            category: record.category.to_uppercase(),
            timestamp: record.timestamp,
        }
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record);
            processed_records.push(transformed);
        }
        
        Ok(processed_records)
    }

    pub fn serialize_records(&self, records: &[DataRecord]) -> Result<String, ProcessingError> {
        serde_json::to_string(records)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }

    pub fn deserialize_records(&self, data: &str) -> Result<Vec<DataRecord>, ProcessingError> {
        serde_json::from_str(data)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(
            0.0,
            100.0,
            vec!["A".to_string(), "B".to_string(), "C".to_string()]
        );
        
        let record = DataRecord {
            id: 1,
            value: 50.0,
            category: "A".to_string(),
            timestamp: 1625097600,
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_value() {
        let processor = DataProcessor::new(
            0.0,
            100.0,
            vec!["A".to_string()]
        );
        
        let record = DataRecord {
            id: 1,
            value: 150.0,
            category: "A".to_string(),
            timestamp: 1625097600,
        };
        
        assert!(matches!(
            processor.validate_record(&record),
            Err(ProcessingError::InvalidValue(150.0))
        ));
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        
        let record = DataRecord {
            id: 1,
            value: 12.345,
            category: "test".to_string(),
            timestamp: 1625097600,
        };
        
        let transformed = processor.transform_record(&record);
        
        assert_eq!(transformed.value, 12.35);
        assert_eq!(transformed.category, "TEST");
    }

    #[test]
    fn test_serialization_deserialization() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.5,
                category: "A".to_string(),
                timestamp: 1625097600,
            },
            DataRecord {
                id: 2,
                value: 20.5,
                category: "B".to_string(),
                timestamp: 1625184000,
            },
        ];
        
        let serialized = processor.serialize_records(&records).unwrap();
        let deserialized = processor.deserialize_records(&serialized).unwrap();
        
        assert_eq!(records.len(), deserialized.len());
        assert_eq!(records[0].id, deserialized[0].id);
        assert_eq!(records[0].value, deserialized[0].value);
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        validate_record(&record)?;
        records.push(record);
    }

    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), Box<dyn Error>> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if record.value < 0.0 {
        return Err("Value cannot be negative".into());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Invalid category".into());
    }
    Ok(())
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

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_valid_data_processing() {
        let data = "id,name,value,category\n1,Test1,10.5,A\n2,Test2,20.0,B\n";
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", data).unwrap();
        
        let result = process_data_file(file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_invalid_category() {
        let data = "id,name,value,category\n1,Test1,10.5,Invalid\n";
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", data).unwrap();
        
        let result = process_data_file(file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "B".to_string() },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 15.0);
        assert_eq!(variance, 25.0);
        assert_eq!(std_dev, 5.0);
    }
}