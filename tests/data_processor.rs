
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
}

fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    let mut valid_count = 0;
    let mut invalid_count = 0;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.is_valid() {
            writer.serialize(&record)?;
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
    }

    println!("Processing complete:");
    println!("  Valid records: {}", valid_count);
    println!("  Invalid records: {}", invalid_count);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "input_data.csv";
    let output_file = "processed_data.csv";
    
    process_csv(input_file, output_file)?;
    
    Ok(())
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
    InvalidValue,
    InvalidTimestamp,
    ValidationFailed(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidValue => write!(f, "Invalid numeric value"),
            DataError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            DataError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<Self, DataError> {
        if value.is_nan() || value.is_infinite() {
            return Err(DataError::InvalidValue);
        }
        
        if timestamp < 0 {
            return Err(DataError::InvalidTimestamp);
        }
        
        Ok(Self {
            id,
            value,
            timestamp,
        })
    }
    
    pub fn validate(&self) -> Result<(), DataError> {
        if self.value < 0.0 || self.value > 1000.0 {
            return Err(DataError::ValidationFailed(
                format!("Value {} out of range [0, 1000]", self.value)
            ));
        }
        
        let current_time = chrono::Utc::now().timestamp();
        if self.timestamp > current_time + 3600 {
            return Err(DataError::ValidationFailed(
                "Timestamp is more than 1 hour in the future".to_string()
            ));
        }
        
        Ok(())
    }
    
    pub fn transform(&self, multiplier: f64) -> Result<Self, DataError> {
        if multiplier <= 0.0 {
            return Err(DataError::InvalidValue);
        }
        
        let transformed_value = self.value * multiplier;
        Self::new(self.id, transformed_value, self.timestamp)
    }
}

pub fn process_records(
    records: Vec<DataRecord>,
    multiplier: f64,
) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        let transformed = record.transform(multiplier)?;
        processed.push(transformed);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, 1672531200);
        assert!(record.is_ok());
        
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.timestamp, 1672531200);
    }
    
    #[test]
    fn test_invalid_value() {
        let record = DataRecord::new(1, f64::NAN, 1672531200);
        assert!(matches!(record, Err(DataError::InvalidValue)));
    }
    
    #[test]
    fn test_validation_success() {
        let record = DataRecord::new(1, 500.0, 1672531200).unwrap();
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_transform_record() {
        let record = DataRecord::new(1, 100.0, 1672531200).unwrap();
        let transformed = record.transform(2.0).unwrap();
        assert_eq!(transformed.value, 200.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
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
            let valid = match parts[3].to_lowercase().as_str() {
                "true" => true,
                "false" => false,
                _ => continue,
            };
            
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
        self.records
            .iter()
            .filter(|record| record.valid)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.records
            .iter()
            .filter(|record| record.valid)
            .collect();
        
        if valid_records.is_empty() {
            return None;
        }
        
        let sum: f64 = valid_records.iter().map(|record| record.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_statistics(&self) -> Statistics {
        let valid_count = self.filter_valid().len();
        let total_count = self.count_records();
        let average = self.calculate_average().unwrap_or(0.0);
        
        Statistics {
            total_records: total_count,
            valid_records: valid_count,
            invalid_records: total_count - valid_count,
            average_value: average,
        }
    }
}

#[derive(Debug)]
pub struct Statistics {
    pub total_records: usize,
    pub valid_records: usize,
    pub invalid_records: usize,
    pub average_value: f64,
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
        writeln!(temp_file, "id,value,category,valid").unwrap();
        writeln!(temp_file, "1,10.5,category_a,true").unwrap();
        writeln!(temp_file, "2,20.3,category_b,false").unwrap();
        writeln!(temp_file, "3,15.7,category_a,true").unwrap();
        
        let count = processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.count_records(), 3);
        
        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 2);
        
        let average = processor.calculate_average().unwrap();
        assert!((average - 13.1).abs() < 0.001);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.total_records, 3);
        assert_eq!(stats.valid_records, 2);
        assert_eq!(stats.invalid_records, 1);
    }
}