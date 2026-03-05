use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, &'static str> {
        if value < 0.0 {
            return Err("Value cannot be negative");
        }
        if category.is_empty() {
            return Err("Category cannot be empty");
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_tax(&self, rate: f64) -> f64 {
        self.value * rate
    }
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let (id, value, category): (u32, f64, String) = result?;
        match DataRecord::new(id, value, category) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Skipping invalid record: {}", e),
        }
    }

    Ok(records)
}

pub fn process_records(records: &[DataRecord], tax_rate: f64) -> Vec<(u32, f64)> {
    records
        .iter()
        .map(|record| (record.id, record.calculate_tax(tax_rate)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 100.0, "Electronics".to_string());
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_negative_value() {
        let record = DataRecord::new(2, -50.0, "Books".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_tax_calculation() {
        let record = DataRecord::new(3, 200.0, "Furniture".to_string()).unwrap();
        assert_eq!(record.calculate_tax(0.15), 30.0);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    OutOfRange(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub metadata: Option<HashMap<String, String>>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            metadata: None,
        }
    }

    pub fn add_value(&mut self, key: &str, value: f64) -> &mut Self {
        self.values.insert(key.to_string(), value);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        let metadata = self.metadata.get_or_insert_with(HashMap::new);
        metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationFailed("ID cannot be zero".to_string()));
        }

        if self.timestamp < 0 {
            return Err(ProcessingError::OutOfRange("Timestamp cannot be negative".to_string()));
        }

        if self.values.is_empty() {
            return Err(ProcessingError::MissingField("At least one value required".to_string()));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(ProcessingError::InvalidFormat);
            }
            if !value.is_finite() {
                return Err(ProcessingError::ValidationFailed(
                    format!("Value for '{}' must be finite", key)
                ));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&mut self) {
        let sum: f64 = self.values.values().sum();
        if sum != 0.0 {
            for value in self.values.values_mut() {
                *value /= sum;
            }
        }
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.values.is_empty() {
            return stats;
        }

        let values: Vec<f64> = self.values.values().copied().collect();
        let count = values.len() as f64;
        
        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        
        stats
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<HashMap<String, f64>>, ProcessingError> {
    let mut results = Vec::new();
    
    for record in records.iter_mut() {
        record.validate()?;
        record.normalize_values();
        results.push(record.calculate_statistics());
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("temperature", 25.5)
              .add_value("humidity", 60.0)
              .add_metadata("sensor", "A1");
        
        assert_eq!(record.id, 1);
        assert_eq!(record.values.len(), 2);
        assert!(record.metadata.is_some());
    }

    #[test]
    fn test_validation() {
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![("temp".to_string(), 25.0)].into_iter().collect(),
            metadata: None,
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            timestamp: 1625097600,
            values: HashMap::new(),
            metadata: None,
        };
        
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("a", 1.0)
              .add_value("b", 2.0)
              .add_value("c", 3.0);
        
        record.normalize_values();
        
        let sum: f64 = record.values.values().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_numeric_data(&mut self, key: &str, values: &[f64]) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data array provided".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Invalid numeric values detected".to_string());
        }

        let processed: Vec<f64> = values
            .iter()
            .map(|&x| (x * 100.0).round() / 100.0)
            .collect();

        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<(f64, f64, f64)> {
        self.cache.get(key).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();
            
            (mean, variance, std_dev)
        })
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cached_keys(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.234, 2.345, 3.456];
        let result = processor.process_numeric_data("test_data", &data);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1.23, 2.35, 3.46]);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, f64::NAN, 3.0];
        let result = processor.process_numeric_data("invalid", &data);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        processor.process_numeric_data("stats", &data).unwrap();
        let stats = processor.calculate_statistics("stats").unwrap();
        
        assert_eq!(stats.0, 3.0);
        assert_eq!(stats.1, 2.0);
        assert_eq!(stats.2, 2.0_f64.sqrt());
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidValue(f64),
    InvalidTimestamp(i64),
    MissingField(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            DataError::InvalidTimestamp(t) => write!(f, "Invalid timestamp: {}", t),
            DataError::MissingField(field) => write!(f, "Missing field: {}", field),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    validation_threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor {
            validation_threshold: threshold,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.value < 0.0 || record.value > self.validation_threshold {
            return Err(DataError::InvalidValue(record.value));
        }

        if record.timestamp < 0 {
            return Err(DataError::InvalidTimestamp(record.timestamp));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> DataRecord {
        DataRecord {
            id: record.id,
            value: record.value * 2.0,
            timestamp: record.timestamp + 3600,
        }
    }

    pub fn process_records(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, DataError>> {
        records
            .into_iter()
            .map(|record| {
                self.validate_record(&record)?;
                Ok(self.transform_record(&record))
            })
            .collect()
    }
}

pub fn calculate_average(records: &[DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1625097600,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_value() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1625097600,
        };

        assert!(matches!(
            processor.validate_record(&record),
            Err(DataError::InvalidValue(150.0))
        ));
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 25.0,
            timestamp: 1625097600,
        };

        let transformed = processor.transform_record(&record);
        assert_eq!(transformed.value, 50.0);
        assert_eq!(transformed.timestamp, 1625097600 + 3600);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 1625097600,
            },
            DataRecord {
                id: 2,
                value: 20.0,
                timestamp: 1625097600,
            },
            DataRecord {
                id: 3,
                value: 30.0,
                timestamp: 1625097600,
            },
        ];

        assert_eq!(calculate_average(&records), Some(20.0));
    }

    #[test]
    fn test_calculate_average_empty() {
        let records: Vec<DataRecord> = vec![];
        assert_eq!(calculate_average(&records), None);
    }
}