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

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if values.is_empty() {
            return Err(DataError::EmptyValues);
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
    
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    pub fn validate(&self) -> Result<(), DataError> {
        for &value in &self.values {
            if !value.is_finite() || value < 0.0 || value > 1000.0 {
                return Err(DataError::ValueOutOfRange(value));
            }
        }
        
        if !self.metadata.contains_key("source") {
            return Err(DataError::MissingMetadata("source".to_string()));
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, factor: f64) {
        for value in &mut self.values {
            *value *= factor;
        }
    }
    
    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.values.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

pub fn process_records(records: &mut [DataRecord], factor: f64) -> Result<Vec<(u32, f64)>, DataError> {
    let mut results = Vec::new();
    
    for record in records {
        record.validate()?;
        record.transform(factor);
        
        let (mean, _, _) = record.calculate_statistics();
        results.push((record.id, mean));
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, vec![10.0, 20.0, 30.0]).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.values, vec![10.0, 20.0, 30.0]);
    }
    
    #[test]
    fn test_invalid_id() {
        let result = DataRecord::new(0, vec![10.0]);
        assert!(matches!(result, Err(DataError::InvalidId)));
    }
    
    #[test]
    fn test_empty_values() {
        let result = DataRecord::new(1, vec![]);
        assert!(matches!(result, Err(DataError::EmptyValues)));
    }
    
    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, vec![2.0, 4.0, 6.0]).unwrap();
        record.add_metadata("source".to_string(), "test".to_string());
        
        let (mean, variance, std_dev) = record.calculate_statistics();
        assert_eq!(mean, 4.0);
        assert_eq!(variance, 8.0 / 3.0);
        assert!((std_dev - (8.0 / 3.0).sqrt()).abs() < 1e-10);
    }
    
    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, vec![100.0, 200.0]).unwrap();
        assert!(record.validate().is_err());
        
        record.add_metadata("source".to_string(), "test".to_string());
        record.values = vec![100.0, 200.0];
        assert!(record.validate().is_ok());
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: String, value: f64, category: String) -> Self {
        DataRecord {
            id,
            timestamp,
            value,
            category,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if self.value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = parts[0].parse::<u32>()?;
            let timestamp = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();

            let record = DataRecord::new(id, timestamp, value, category);
            if let Err(e) = record.validate() {
                eprintln!("Validation error on line {}: {}", line_num + 1, e);
                continue;
            }

            self.records.push(record);
            count += 1;
        }

        Ok(count)
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

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|record| record.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "2024-01-01".to_string(), 100.0, "A".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, "2024-01-01".to_string(), 100.0, "A".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,timestamp,value,category").unwrap();
        writeln!(temp_file, "1,2024-01-01,100.0,A").unwrap();
        writeln!(temp_file, "2,2024-01-02,200.0,B").unwrap();
        writeln!(temp_file, "3,2024-01-03,300.0,A").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.count_records(), 3);

        let category_a = processor.filter_by_category("A");
        assert_eq!(category_a.len(), 2);

        let stats = processor.get_statistics();
        assert_eq!(stats.0, 100.0);
        assert_eq!(stats.1, 300.0);
        assert_eq!(stats.2, 200.0);
    }
}