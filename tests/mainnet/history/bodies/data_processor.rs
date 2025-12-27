
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn validate_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value.is_finite() && !r.category.is_empty())
            .collect()
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);
        
        (min, max, avg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.3,beta").unwrap();
        writeln!(temp_file, "3,15.7,alpha").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.5).abs() < 0.1);
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
        
        let stats = processor.get_statistics();
        assert!((stats.0 - 10.5).abs() < 0.1);
        assert!((stats.1 - 20.3).abs() < 0.1);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub metadata: Option<HashMap<String, String>>,
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
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            metadata: None,
        }
    }

    pub fn add_value(&mut self, key: &str, value: f64) {
        self.values.insert(key.to_string(), value);
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }
        if let Some(metadata) = &mut self.metadata {
            metadata.insert(key.to_string(), value.to_string());
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
            return Err(ProcessingError::ValidationError("Record must contain at least one value".to_string()));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(ProcessingError::ValidationError("Value key cannot be empty".to_string()));
            }
            if !value.is_finite() {
                return Err(ProcessingError::ValidationError(format!("Value for key '{}' is not finite", key)));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&mut self) -> Result<(), ProcessingError> {
        if self.values.is_empty() {
            return Err(ProcessingError::TransformationFailed("No values to normalize".to_string()));
        }

        let sum: f64 = self.values.values().sum();
        if sum == 0.0 {
            return Err(ProcessingError::TransformationFailed("Sum of values is zero, cannot normalize".to_string()));
        }

        for value in self.values.values_mut() {
            *value /= sum;
        }

        Ok(())
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

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        
        let mut processed_record = record.clone();
        processed_record.normalize_values()?;
        
        processed.push(processed_record);
    }
    
    Ok(processed)
}

pub fn serialize_to_json(records: &[DataRecord]) -> Result<String, ProcessingError> {
    serde_json::to_string(records)
        .map_err(|e| ProcessingError::TransformationFailed(format!("JSON serialization failed: {}", e)))
}

pub fn deserialize_from_json(json_str: &str) -> Result<Vec<DataRecord>, ProcessingError> {
    serde_json::from_str(json_str)
        .map_err(|e| ProcessingError::InvalidData(format!("JSON deserialization failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature", 25.5);
        record.add_metadata("source", "sensor_01");
        
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1234567890);
        assert_eq!(record.values.get("temperature"), Some(&25.5));
        assert_eq!(record.metadata.as_ref().unwrap().get("source"), Some(&"sensor_01".to_string()));
    }

    #[test]
    fn test_validation() {
        let mut valid_record = DataRecord::new(1, 1234567890);
        valid_record.add_value("temp", 25.5);
        assert!(valid_record.validate().is_ok());

        let mut invalid_record = DataRecord::new(0, 1234567890);
        invalid_record.add_value("temp", 25.5);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("a", 1.0);
        record.add_value("b", 2.0);
        record.add_value("c", 3.0);
        
        assert!(record.normalize_values().is_ok());
        
        let sum: f64 = record.values.values().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_statistics() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("x", 1.0);
        record.add_value("y", 2.0);
        record.add_value("z", 3.0);
        
        let stats = record.calculate_statistics();
        assert_eq!(stats.get("count"), Some(&3.0));
        assert_eq!(stats.get("sum"), Some(&6.0));
        assert_eq!(stats.get("mean"), Some(&2.0));
    }
}