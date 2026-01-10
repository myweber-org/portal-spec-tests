
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
        validate_record(&record)?;
        records.push(record);
    }

    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.is_empty() {
        return Err(format!("Empty name for record ID {}", record.id));
    }
    
    if record.value < 0.0 {
        return Err(format!("Negative value for record ID {}", record.id));
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
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,20.0,false").unwrap();
        
        let result = process_data_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_invalid_data_handling() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,,10.5,true").unwrap();
        
        let result = process_data_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessedData {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub is_valid: bool,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    EmptyCategory,
    ValidationFailed(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyCategory => write!(f, "Category cannot be empty"),
            DataError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    pub fn process_raw_data(
        &self,
        id: u32,
        value: f64,
        category: &str,
    ) -> Result<ProcessedData, DataError> {
        self.validate_input(id, value, category)?;

        let processed_value = self.transform_value(value);
        let is_valid = processed_value >= self.threshold;

        Ok(ProcessedData {
            id,
            value: processed_value,
            category: category.to_string(),
            is_valid,
        })
    }

    fn validate_input(&self, id: u32, value: f64, category: &str) -> Result<(), DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }

        if value < 0.0 || value > 1000.0 {
            return Err(DataError::InvalidValue);
        }

        if category.trim().is_empty() {
            return Err(DataError::EmptyCategory);
        }

        Ok(())
    }

    fn transform_value(&self, value: f64) -> f64 {
        (value * 1.1).round() / 1.0
    }

    pub fn batch_process(
        &self,
        data_points: &[(u32, f64, &str)],
    ) -> Vec<Result<ProcessedData, DataError>> {
        data_points
            .iter()
            .map(|&(id, value, category)| self.process_raw_data(id, value, category))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_data_processing() {
        let processor = DataProcessor::new(50.0);
        let result = processor.process_raw_data(1, 45.5, "analytics");

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.id, 1);
        assert!(data.value > 45.5);
        assert_eq!(data.category, "analytics");
    }

    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new(50.0);
        let result = processor.process_raw_data(0, 45.5, "analytics");

        assert!(result.is_err());
        match result.unwrap_err() {
            DataError::InvalidId => (),
            _ => panic!("Expected InvalidId error"),
        }
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(50.0);
        let data_points = vec![
            (1, 45.5, "analytics"),
            (2, 60.0, "metrics"),
            (3, 30.0, "logs"),
        ];

        let results = processor.batch_process(&data_points);
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_ok());
    }
}