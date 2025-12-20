use csv::{Reader, Writer};
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

struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    fn process_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let mut reader = Reader::from_path(input_path)?;
        let mut writer = Writer::from_writer(File::create(output_path)?);

        for result in reader.deserialize() {
            let record: Record = result?;
            
            if self.filter_record(&record) {
                let processed = self.transform_record(record);
                writer.serialize(processed)?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    fn filter_record(&self, record: &Record) -> bool {
        record.active && record.value >= self.threshold
    }

    fn transform_record(&self, mut record: Record) -> Record {
        record.value = (record.value * 100.0).round() / 100.0;
        record
    }
}

fn validate_data(records: &[Record]) -> bool {
    records.iter().all(|r| !r.name.is_empty() && r.id > 0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let processor = DataProcessor::new(50.0);
    processor.process_file("input.csv", "output.csv")?;
    
    println!("Data processing completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_record() {
        let processor = DataProcessor::new(50.0);
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 75.5,
            active: true,
        };
        
        assert!(processor.filter_record(&record));
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(0.0);
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 123.456,
            active: true,
        };
        
        let transformed = processor.transform_record(record);
        assert_eq!(transformed.value, 123.46);
    }
}
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

pub fn process_csv_data(input_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
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
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Category must be A, B, or C".into());
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
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_valid_csv_processing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,category").unwrap();
        writeln!(file, "1,Test1,10.5,A").unwrap();
        writeln!(file, "2,Test2,20.0,B").unwrap();
        
        let result = process_csv_data(file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "C".to_string() },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Transformation failed: {0}")]
    TransformationFailed(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationError("Timestamp cannot be negative".to_string()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::ValidationError("Values cannot be empty".to_string()));
        }
        
        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationError("Key cannot be empty".to_string()));
            }
            if !value.is_finite() {
                return Err(DataError::ValidationError(format!("Value for key '{}' is not finite", key)));
            }
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), DataError> {
        if !multiplier.is_finite() || multiplier == 0.0 {
            return Err(DataError::TransformationFailed("Invalid multiplier".to_string()));
        }
        
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
        
        self.timestamp += 1;
        Ok(())
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    pub fn calculate_sum(&self) -> f64 {
        self.values.values().sum()
    }
    
    pub fn calculate_average(&self) -> Option<f64> {
        let count = self.values.len() as f64;
        if count > 0.0 {
            Some(self.calculate_sum() / count)
        } else {
            None
        }
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        let mut processed_record = record.clone();
        processed_record.transform(multiplier)?;
        processed_record.add_tag("processed".to_string());
        processed.push(processed_record);
    }
    
    Ok(processed)
}

pub fn filter_records_by_threshold(records: &[DataRecord], threshold: f64) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| record.calculate_sum() > threshold)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    fn create_test_record() -> DataRecord {
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), 25.5);
        values.insert("humidity".to_string(), 60.0);
        
        DataRecord {
            id: 1,
            timestamp: 1625097600,
            values,
            tags: vec!["sensor".to_string()],
        }
    }
    
    #[test]
    fn test_record_validation() {
        let record = create_test_record();
        assert!(record.validate().is_ok());
        
        let mut invalid_record = record.clone();
        invalid_record.id = 0;
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = create_test_record();
        let original_sum = record.calculate_sum();
        
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.calculate_sum(), original_sum * 2.0);
        assert_eq!(record.timestamp, 1625097601);
        assert!(record.tags.contains(&"processed".to_string()));
    }
    
    #[test]
    fn test_calculate_average() {
        let record = create_test_record();
        let average = record.calculate_average();
        assert!(average.is_some());
        assert_eq!(average.unwrap(), (25.5 + 60.0) / 2.0);
    }
    
    #[test]
    fn test_filter_records() {
        let record1 = create_test_record();
        let mut record2 = create_test_record();
        record2.values.insert("pressure".to_string(), 100.0);
        
        let records = vec![record1, record2];
        let filtered = filter_records_by_threshold(&records, 150.0);
        assert_eq!(filtered.len(), 1);
    }
}