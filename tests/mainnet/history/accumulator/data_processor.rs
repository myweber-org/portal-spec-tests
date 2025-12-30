
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: String) -> Self {
        Self {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.category.is_empty() 
            && self.value.is_finite() 
            && self.id > 0
            && !self.timestamp.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
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

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].to_string();
            let timestamp = parts[3].to_string();

            let record = DataRecord::new(id, value, category, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
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

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A".to_string(), "2024-01-01".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, f64::NAN, "".to_string(), "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);
        assert_eq!(processor.calculate_average(), None);
    }

    #[test]
    fn test_filtering() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A".to_string(), "time1".to_string()));
        processor.records.push(DataRecord::new(2, 20.0, "B".to_string(), "time2".to_string()));
        processor.records.push(DataRecord::new(3, 30.0, "A".to_string(), "time3".to_string()));

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
    }
}use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
    max_values: usize,
}

impl DataProcessor {
    pub fn new(threshold: f64, max_values: usize) -> Self {
        DataProcessor {
            threshold,
            max_values,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationError("Empty values array".to_string()));
        }

        if record.values.len() > self.max_values {
            return Err(ProcessingError::ValidationError(
                format!("Too many values: {} > {}", record.values.len(), self.max_values)
            ));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::InvalidData(
                    format!("Invalid numeric value: {}", value)
                ));
            }
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(record)?;

        let mut transformed_values = Vec::with_capacity(record.values.len());
        for value in &record.values {
            let transformed = if *value > self.threshold {
                value.ln()
            } else {
                *value
            };
            
            if transformed.is_nan() || transformed.is_infinite() {
                return Err(ProcessingError::TransformationFailed(
                    format!("Transformation produced invalid value: {}", transformed)
                ));
            }
            
            transformed_values.push(transformed);
        }

        let mut transformed_metadata = record.metadata.clone();
        transformed_metadata.insert("processed".to_string(), "true".to_string());
        transformed_metadata.insert("transformation_threshold".to_string(), self.threshold.to_string());

        Ok(DataRecord {
            id: record.id,
            timestamp: record.timestamp,
            values: transformed_values,
            metadata: transformed_metadata,
        })
    }

    pub fn batch_process(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());
        
        for record in records {
            match self.transform_record(&record) {
                Ok(transformed) => results.push(transformed),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(10.0, 5);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_empty_values() {
        let processor = DataProcessor::new(10.0, 5);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![],
            metadata: HashMap::new(),
        };
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transformation() {
        let processor = DataProcessor::new(2.0, 5);
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 3.0, 5.0],
            metadata,
        };
        
        let result = processor.transform_record(&record).unwrap();
        assert_eq!(result.values[0], 1.0);
        assert_eq!(result.values[1], 3.0_f64.ln());
        assert_eq!(result.values[2], 5.0_f64.ln());
        assert_eq!(result.metadata.get("processed").unwrap(), "true");
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
    active: bool,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let count = records.len();
    if count == 0 {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = sum / count as f64;
    let max = records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);

    (avg, max, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,-3.2,false").unwrap();
        writeln!(temp_file, "3,ItemC,7.8,true").unwrap();

        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].name, "ItemC");
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 5.0, active: true },
            Record { id: 2, name: "Test2".to_string(), value: 15.0, active: false },
            Record { id: 3, name: "Test3".to_string(), value: 10.0, active: true },
        ];

        let (avg, max, count) = calculate_statistics(&records);
        assert_eq!(avg, 10.0);
        assert_eq!(max, 15.0);
        assert_eq!(count, 3);
    }
}