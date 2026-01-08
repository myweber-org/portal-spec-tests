
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

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let processed = Self::normalize_data(data)?;
        self.cache.insert(key.to_string(), processed.clone());
        
        Ok(processed)
    }

    fn normalize_data(data: &[f64]) -> Result<Vec<f64>, String> {
        let max_value = data
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if max_value <= 0.0 {
            return Err("Invalid data range for normalization".to_string());
        }

        let normalized: Vec<f64> = data
            .iter()
            .map(|&x| x / max_value)
            .collect();

        Ok(normalized)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if data.is_empty() {
            return stats;
        }

        let sum: f64 = data.iter().sum();
        let count = data.len() as f64;
        let mean = sum / count;

        let variance: f64 = data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);

        stats
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let result = DataProcessor::normalize_data(&data).unwrap();
        assert_eq!(result, vec![0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("test", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let stats = processor.calculate_statistics(&data);
        
        assert_eq!(stats.get("mean"), Some(&2.5));
        assert_eq!(stats.get("count"), Some(&4.0));
        assert_eq!(stats.get("sum"), Some(&10.0));
    }
}use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidCategory,
    SerializationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Value must be positive"),
            ProcessingError::InvalidCategory => write!(f, "Category cannot be empty"),
            ProcessingError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.value <= 0.0 {
        return Err(ProcessingError::InvalidValue);
    }
    
    if record.category.trim().is_empty() {
        return Err(ProcessingError::InvalidCategory);
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord) -> DataRecord {
    DataRecord {
        id: record.id,
        value: record.value * 2.0,
        category: record.category.to_uppercase(),
    }
}

pub fn process_records(records: &[DataRecord]) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        validate_record(record)?;
        processed.push(transform_record(record));
    }
    
    Ok(processed)
}

pub fn serialize_records(records: &[DataRecord]) -> Result<String, ProcessingError> {
    serde_json::to_string(records)
        .map_err(|e| ProcessingError::SerializationError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            value: 10.5,
            category: "test".to_string(),
        };
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_record_invalid_value() {
        let record = DataRecord {
            id: 1,
            value: -5.0,
            category: "test".to_string(),
        };
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord {
            id: 1,
            value: 10.0,
            category: "example".to_string(),
        };
        let transformed = transform_record(&record);
        assert_eq!(transformed.value, 20.0);
        assert_eq!(transformed.category, "EXAMPLE");
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
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
        let reader = BufReader::new(file);

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();

            self.records.push(DataRecord { id, value, category });
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn validate_records(&self) -> Vec<u32> {
        self.records
            .iter()
            .filter(|record| record.value < 0.0 || record.value > 1000.0)
            .map(|record| record.id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,42.5,alpha").unwrap();
        writeln!(temp_file, "2,1500.0,beta").unwrap();
        writeln!(temp_file, "3,78.9,alpha").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);

        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);

        let invalid_ids = processor.validate_records();
        assert_eq!(invalid_ids, vec![2]);
    }
}