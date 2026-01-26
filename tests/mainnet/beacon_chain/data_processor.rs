
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data value: {0}")]
    InvalidValue(f64),
    #[error("Timestamp out of range: {0}")]
    InvalidTimestamp(i64),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64) -> Self {
        DataProcessor { min_value, max_value }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < self.min_value || record.value > self.max_value {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp(record.timestamp));
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

    pub fn process_records(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());

        for record in records {
            self.validate_record(&record)?;
            processed.push(self.transform_record(&record));
        }

        Ok(processed)
    }

    pub fn serialize_records(&self, records: &[DataRecord]) -> Result<String, ProcessingError> {
        serde_json::to_string(records)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1234567890,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1234567890,
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1000,
        };

        let transformed = processor.transform_record(&record);
        assert_eq!(transformed.value, 100.0);
        assert_eq!(transformed.timestamp, 4600);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    MissingField,
    ValidationFailed(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid numeric value"),
            DataError::MissingField => write!(f, "Required field is missing"),
            DataError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if !value.is_finite() || value < 0.0 {
            return Err(DataError::InvalidValue);
        }
        
        if name.trim().is_empty() {
            return Err(DataError::MissingField);
        }
        
        Ok(DataRecord {
            id,
            name: name.trim().to_string(),
            value,
            metadata: HashMap::new(),
        })
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    pub fn transform_value<F>(&mut self, transformer: F) 
    where
        F: Fn(f64) -> f64,
    {
        self.value = transformer(self.value);
    }
    
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if !self.value.is_finite() {
            return Err(DataError::InvalidValue);
        }
        
        if self.name.is_empty() {
            return Err(DataError::MissingField);
        }
        
        Ok(())
    }
    
    pub fn to_json(&self) -> String {
        let metadata_json: Vec<String> = self.metadata
            .iter()
            .map(|(k, v)| format!("\"{}\": \"{}\"", k, v))
            .collect();
        
        format!(
            "{{\"id\": {}, \"name\": \"{}\", \"value\": {}, \"metadata\": {{{}}}}}",
            self.id,
            self.name,
            self.value,
            metadata_json.join(", ")
        )
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<String>, DataError> {
    let mut results = Vec::new();
    
    for record in records.iter_mut() {
        record.validate()?;
        
        record.transform_value(|v| v * 1.1);
        
        if let Some(metadata) = record.get_metadata("category") {
            if metadata == "premium" {
                record.transform_value(|v| v * 1.2);
            }
        }
        
        results.push(record.to_json());
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, "Test Record".to_string(), 100.0);
        assert!(record.is_ok());
        
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test Record");
        assert_eq!(record.value, 100.0);
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, "Test".to_string(), 100.0);
        assert!(matches!(record, Err(DataError::InvalidId)));
    }
    
    #[test]
    fn test_metadata_operations() {
        let mut record = DataRecord::new(1, "Test".to_string(), 50.0).unwrap();
        record.add_metadata("category".to_string(), "standard".to_string());
        
        assert_eq!(record.get_metadata("category"), Some(&"standard".to_string()));
        assert_eq!(record.get_metadata("nonexistent"), None);
    }
    
    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 100.0).unwrap();
        record.transform_value(|v| v * 2.0);
        
        assert_eq!(record.value, 200.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    file_path: String,
    filter_column: usize,
    filter_value: String,
}

impl DataProcessor {
    pub fn new(file_path: &str, filter_column: usize, filter_value: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
            filter_column,
            filter_value: filter_value.to_string(),
        }
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_data = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if columns.len() > self.filter_column {
                if columns[self.filter_column] == self.filter_value {
                    filtered_data.push(columns);
                }
            }
        }

        Ok(filtered_data)
    }

    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let data = self.process()?;
        Ok(data.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,status").unwrap();
        writeln!(temp_file, "1,alice,active").unwrap();
        writeln!(temp_file, "2,bob,inactive").unwrap();
        writeln!(temp_file, "3,charlie,active").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), 2, "active");
        let result = processor.process().unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][1], "alice");
        assert_eq!(result[1][1], "charlie");
    }
}