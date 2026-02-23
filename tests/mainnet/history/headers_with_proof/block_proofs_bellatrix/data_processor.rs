
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

pub fn process_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(file_path)?;
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
        return Err("Value must be non-negative".into());
    }
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = if count > 0.0 { sum / count } else { 0.0 };
    
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
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }
            
            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let category = parts[2].to_string();
            let valid = value >= 0.0 && value <= 1000.0;
            
            self.records.push(DataRecord {
                id,
                value,
                category,
                valid,
            });
            
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }
        
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn get_records_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.valid)
            .collect()
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }

    pub fn valid_records_count(&self) -> usize {
        self.filter_valid().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,TypeA").unwrap();
        writeln!(temp_file, "2,200.0,TypeB").unwrap();
        writeln!(temp_file, "3,150.75,TypeA").unwrap();
        writeln!(temp_file, "4,-10.0,TypeC").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4);
        assert_eq!(processor.total_records(), 4);
        assert_eq!(processor.valid_records_count(), 3);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 150.41666666666666).abs() < 0.0001);
        
        let type_a_records = processor.get_records_by_category("TypeA");
        assert_eq!(type_a_records.len(), 2);
    }
}
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    CategoryTooLong,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Value must be between 0 and 1000"),
            ProcessingError::InvalidTimestamp => write!(f, "Timestamp cannot be negative"),
            ProcessingError::CategoryTooLong => write!(f, "Category exceeds maximum length"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    max_category_length: usize,
}

impl DataProcessor {
    pub fn new(max_category_length: usize) -> Self {
        Self {
            max_category_length,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < 0.0 || record.value > 1000.0 {
            return Err(ProcessingError::InvalidValue);
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }

        if record.category.len() > self.max_category_length {
            return Err(ProcessingError::CategoryTooLong);
        }

        Ok(())
    }

    pub fn transform_value(&self, record: &DataRecord) -> f64 {
        match record.category.as_str() {
            "temperature" => (record.value - 32.0) * 5.0 / 9.0,
            "pressure" => record.value * 1000.0,
            "humidity" => record.value / 100.0,
            _ => record.value,
        }
    }

    pub fn process_records(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());

        for record in records {
            self.validate_record(&record)?;

            let transformed_value = self.transform_value(&record);
            let processed_record = DataRecord {
                value: transformed_value,
                ..record
            };

            processed.push(processed_record);
        }

        Ok(processed)
    }

    pub fn filter_by_category(&self, records: Vec<DataRecord>, category: &str) -> Vec<DataRecord> {
        records
            .into_iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> (f64, f64, f64) {
        if records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len() as f64;
        let mean = sum / count;

        let variance: f64 = records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(50);
        let record = DataRecord {
            id: 1,
            value: 500.0,
            timestamp: 1234567890,
            category: "temperature".to_string(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_value() {
        let processor = DataProcessor::new(50);
        let record = DataRecord {
            id: 1,
            value: 1500.0,
            timestamp: 1234567890,
            category: "temperature".to_string(),
        };

        assert!(matches!(
            processor.validate_record(&record),
            Err(ProcessingError::InvalidValue)
        ));
    }

    #[test]
    fn test_transform_temperature() {
        let processor = DataProcessor::new(50);
        let record = DataRecord {
            id: 1,
            value: 212.0,
            timestamp: 1234567890,
            category: "temperature".to_string(),
        };

        let transformed = processor.transform_value(&record);
        assert!((transformed - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(50);
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 1,
                category: "test".to_string(),
            },
            DataRecord {
                id: 2,
                value: 20.0,
                timestamp: 2,
                category: "test".to_string(),
            },
            DataRecord {
                id: 3,
                value: 30.0,
                timestamp: 3,
                category: "test".to_string(),
            },
        ];

        let (mean, variance, std_dev) = processor.calculate_statistics(&records);
        assert!((mean - 20.0).abs() < 0.001);
        assert!((variance - 66.666).abs() < 0.1);
        assert!((std_dev - 8.1649).abs() < 0.001);
    }
}