
use csv;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
    
    fn process(&mut self) {
        self.name = self.name.to_uppercase();
        self.value = (self.value * 100.0).round() / 100.0;
    }
}

pub fn load_and_process_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let mut records: Vec<Record> = Vec::new();
    
    for result in rdr.deserialize() {
        let mut record: Record = result?;
        
        if record.is_valid() {
            record.process();
            records.push(record);
        }
    }
    
    Ok(records)
}

pub fn save_processed_data(records: &[Record], output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut wtr = csv::Writer::from_writer(file);
    
    for record in records {
        wtr.serialize(record)?;
    }
    
    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "test".to_string(),
            value: 42.5,
            active: true,
        };
        
        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -10.0,
            active: false,
        };
        
        assert!(valid_record.is_valid());
        assert!(!invalid_record.is_valid());
    }
    
    #[test]
    fn test_record_processing() {
        let mut record = Record {
            id: 1,
            name: "hello".to_string(),
            value: 123.456,
            active: true,
        };
        
        record.process();
        
        assert_eq!(record.name, "HELLO");
        assert_eq!(record.value, 123.46);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
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

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Result<Self, ProcessingError> {
        if values.is_empty() {
            return Err(ProcessingError::InvalidData("Values cannot be empty".to_string()));
        }
        
        if values.iter().any(|&v| v.is_nan() || v.is_infinite()) {
            return Err(ProcessingError::InvalidData("Values contain NaN or infinite numbers".to_string()));
        }
        
        Ok(Self {
            id,
            values,
            metadata: HashMap::new(),
        })
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if self.values.len() > 1000 {
            return Err(ProcessingError::ValidationError("Too many values".to_string()));
        }
        
        Ok(())
    }
    
    pub fn normalize(&mut self) -> Result<(), ProcessingError> {
        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max - min).abs() < f64::EPSILON {
            return Err(ProcessingError::TransformationError("Cannot normalize constant values".to_string()));
        }
        
        for value in &mut self.values {
            *value = (*value - min) / (max - min);
        }
        
        Ok(())
    }
    
    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let sorted_values = {
            let mut sorted = self.values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted
        };
        
        let median = if count as usize % 2 == 0 {
            let mid = count as usize / 2;
            (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
        } else {
            sorted_values[count as usize / 2]
        };
        
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("median".to_string(), median);
        stats.insert("min".to_string(), *sorted_values.first().unwrap_or(&0.0));
        stats.insert("max".to_string(), *sorted_values.last().unwrap_or(&0.0));
        stats.insert("sum".to_string(), sum);
        stats.insert("count".to_string(), count);
        
        stats
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<HashMap<String, f64>>, ProcessingError> {
    let mut results = Vec::new();
    
    for record in records {
        record.validate()?;
        record.normalize()?;
        results.push(record.calculate_statistics());
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.values, vec![1.0, 2.0, 3.0]);
    }
    
    #[test]
    fn test_invalid_record_creation() {
        let result = DataRecord::new(1, vec![]);
        assert!(result.is_err());
        
        let result = DataRecord::new(1, vec![f64::NAN, 2.0]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        record.normalize().unwrap();
        
        let expected = vec![0.0, 0.5, 1.0];
        for (actual, expected) in record.values.iter().zip(expected.iter()) {
            assert!((actual - expected).abs() < 1e-10);
        }
    }
    
    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        let stats = record.calculate_statistics();
        
        assert!((stats["mean"] - 3.0).abs() < 1e-10);
        assert!((stats["median"] - 3.0).abs() < 1e-10);
        assert_eq!(stats["count"], 5.0);
    }
}