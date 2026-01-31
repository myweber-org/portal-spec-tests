
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records to process".into());
        }

        let mut values = Vec::new();
        for record in records {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }

            match record[column_index].parse::<f64>() {
                Ok(value) => values.push(value),
                Err(_) => return Err(format!("Invalid numeric value at column {}", column_index).into()),
            }
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        let variance: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;

        Ok((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "50000"]);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "25.0".to_string()],
            vec!["12.0".to_string(), "30.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let (mean, std_dev) = processor.calculate_statistics(&records, 0).unwrap();
        
        assert!((mean - 12.666).abs() < 0.001);
        assert!((std_dev - 2.054).abs() < 0.001);
    }
}
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        if self.timestamp < 0 {
            return Err("Invalid timestamp".into());
        }
        if self.values.is_empty() {
            return Err("No values provided".into());
        }
        Ok(())
    }
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    let all_values: Vec<f64> = records.iter().flat_map(|r| r.values.clone()).collect();

    if !all_values.is_empty() {
        let sum: f64 = all_values.iter().sum();
        let count = all_values.len() as f64;
        let mean = sum / count;

        let variance: f64 = all_values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
    }

    stats
}

pub fn transform_records(records: Vec<DataRecord>, multiplier: f64) -> Vec<DataRecord> {
    records.into_iter()
        .map(|mut record| {
            record.values = record.values.iter()
                .map(|&v| v * multiplier)
                .collect();
            record
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 1625097600);
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1625097600);
    }

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value(42.5);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record1 = DataRecord::new(1, 1625097600);
        record1.add_value(10.0).add_value(20.0);

        let mut record2 = DataRecord::new(2, 1625097600);
        record2.add_value(30.0);

        let records = vec![record1, record2];
        let stats = calculate_statistics(&records);

        assert_eq!(stats.get("mean"), Some(&20.0));
        assert_eq!(stats.get("count"), Some(&3.0));
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
    category: String,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let max_value = records.iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, |a, b| a.max(b));
    
    (average, max_value, count)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut reader = Reader::from_path(path)?;
        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
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

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn validate_records(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for (index, record) in self.records.iter().enumerate() {
            if record.name.trim().is_empty() {
                errors.push(format!("Record {}: Name is empty", index));
            }
            if record.value < 0.0 {
                errors.push(format!("Record {}: Value is negative", index));
            }
            if record.category.trim().is_empty() {
                errors.push(format!("Record {}: Category is empty", index));
            }
        }
        errors
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
    fn test_data_processing() {
        let csv_data = "id,name,value,category\n\
                        1,ItemA,42.5,Electronics\n\
                        2,ItemB,18.3,Books\n\
                        3,ItemC,75.0,Electronics\n\
                        4,ItemD,33.7,Books";

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();

        let mut processor = DataProcessor::new();
        processor.load_from_csv(temp_file.path()).unwrap();

        assert_eq!(processor.count_records(), 4);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 42.375).abs() < 0.001);
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 3);
        
        let errors = processor.validate_records();
        assert!(errors.is_empty());
    }
}
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Transformation failed: {0}")]
    TransformationFailed(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
    pub metadata: Option<serde_json::Value>,
}

pub struct DataProcessor {
    validation_threshold: f64,
    transformation_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, transformation_factor: f64) -> Self {
        Self {
            validation_threshold,
            transformation_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value.abs() > self.validation_threshold {
            return Err(ProcessingError::ValidationError(
                format!("Value {} exceeds threshold {}", record.value, self.validation_threshold)
            ));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::ValidationError(
                "Timestamp cannot be negative".to_string()
            ));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(record)?;

        let transformed_value = record.value * self.transformation_factor;
        
        if transformed_value.is_nan() || transformed_value.is_infinite() {
            return Err(ProcessingError::TransformationFailed(
                "Resulting value is not finite".to_string()
            ));
        }

        Ok(DataRecord {
            id: record.id,
            value: transformed_value,
            timestamp: record.timestamp,
            metadata: record.metadata.clone(),
        })
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records
            .into_iter()
            .map(|record| self.transform_record(&record))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1234567890,
            metadata: None,
        };

        let result = processor.transform_record(&record);
        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed.value, 85.0);
    }

    #[test]
    fn test_invalid_record_validation() {
        let processor = DataProcessor::new(50.0, 1.0);
        let record = DataRecord {
            id: 2,
            value: 100.0,
            timestamp: 1234567890,
            metadata: None,
        };

        let result = processor.validate_record(&record);
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(100.0, 3.0);
        let records = vec![
            DataRecord { id: 1, value: 10.0, timestamp: 1000, metadata: None },
            DataRecord { id: 2, value: 200.0, timestamp: 2000, metadata: None },
            DataRecord { id: 3, value: 30.0, timestamp: 3000, metadata: None },
        ];

        let results = processor.process_batch(records);
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        assert!(results[2].is_ok());
    }
}