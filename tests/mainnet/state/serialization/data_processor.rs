use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err("File does not exist".into());
        }

        let mut rdr = Reader::from_path(file_path)?;
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&Record> = self.validate_records();
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let test_data = "id,name,value,category\n1,Test1,10.5,A\n2,Test2,-5.0,B\n3,,15.0,A";
        let mut file = File::create("test_data.csv").unwrap();
        file.write_all(test_data.as_bytes()).unwrap();

        let result = processor.load_from_csv("test_data.csv");
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);

        let valid_records = processor.validate_records();
        assert_eq!(valid_records.len(), 2);

        let avg = processor.calculate_average();
        assert_eq!(avg, Some(12.75));

        let category_a = processor.filter_by_category("A");
        assert_eq!(category_a.len(), 2);

        std::fs::remove_file("test_data.csv").unwrap();
    }
}use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    ValidationFailed(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            DataError::EmptyValues => write!(f, "Record contains no values"),
            DataError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64, values: Vec<f64>) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if timestamp < 0 {
            return Err(DataError::InvalidTimestamp);
        }
        if values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        Ok(Self {
            id,
            timestamp,
            values,
            metadata: HashMap::new(),
        })
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.values.iter().any(|&v| v.is_nan() || v.is_infinite()) {
            return Err(DataError::ValidationFailed(
                "Contains invalid numeric values".to_string(),
            ));
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = self.values
            .iter()
            .map(|&value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

pub fn process_records(records: &[DataRecord]) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        
        let mut processed_record = record.clone();
        
        let (mean, _, std_dev) = processed_record.calculate_statistics();
        
        processed_record.add_metadata(
            "processed".to_string(),
            "true".to_string()
        );
        processed_record.add_metadata(
            "mean".to_string(),
            format!("{:.4}", mean)
        );
        processed_record.add_metadata(
            "std_dev".to_string(),
            format!("{:.4}", std_dev)
        );
        
        processed.push(processed_record);
    }
    
    Ok(processed)
}

pub fn filter_records_by_threshold(
    records: &[DataRecord],
    threshold: f64
) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| {
            let (mean, _, _) = record.calculate_statistics();
            mean >= threshold
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 1234567890, vec![1.0, 2.0]);
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0, 4.0]).unwrap();
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert_eq!(mean, 2.5);
        assert_eq!(variance, 1.25);
        assert_eq!(std_dev, variance.sqrt());
    }

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]).unwrap();
        assert!(valid_record.validate().is_ok());
    }
}use csv::{Reader, Writer};
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

pub fn process_data(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_writer(File::create(output_path)?);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

pub fn calculate_statistics(path: &str) -> Result<(f64, f64, usize), Box<dyn Error>> {
    let mut reader = Reader::from_path(path)?;
    let mut sum = 0.0;
    let mut count = 0;
    let mut max_value = f64::MIN;

    for result in reader.deserialize() {
        let record: Record = result?;
        sum += record.value;
        count += 1;
        
        if record.value > max_value {
            max_value = record.value;
        }
    }

    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    Ok((average, max_value, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_data() {
        let input_data = "id,name,value,category\n1,ItemA,15.5,Alpha\n2,ItemB,8.2,Beta\n3,ItemC,22.1,Alpha";
        let input_file = NamedTempFile::new().unwrap();
        std::fs::write(input_file.path(), input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let result = process_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            10.0
        );
        
        assert!(result.is_ok());
        
        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(output_content.contains("ItemA"));
        assert!(!output_content.contains("ItemB"));
        assert!(output_content.contains("ItemC"));
    }
}