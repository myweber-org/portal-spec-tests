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

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line.split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String], expected_fields: usize) -> bool {
        record.len() == expected_fields && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_average(&self, records: &[Vec<String>], column_index: usize) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;

        for record in records {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert!(processor.validate_record(&records[0], 3));
    }

    #[test]
    fn test_average_calculation() {
        let records = vec![
            vec!["Alice".to_string(), "30".to_string(), "50000.0".to_string()],
            vec!["Bob".to_string(), "25".to_string(), "45000.0".to_string()],
            vec!["Charlie".to_string(), "35".to_string(), "55000.0".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let average_age = processor.calculate_average(&records, 1);
        let average_salary = processor.calculate_average(&records, 2);

        assert_eq!(average_age, Some(30.0));
        assert_eq!(average_salary, Some(50000.0));
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

            let category = parts[2].trim().to_string();
            if category.is_empty() {
                continue;
            }

            self.records.push(DataRecord {
                id,
                value,
                category,
            });
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

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
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
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.3,beta").unwrap();
        writeln!(temp_file, "3,15.7,alpha").unwrap();
        
        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
        
        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.5).abs() < 0.01);
        
        let (min, max, avg_stat) = processor.get_statistics();
        assert_eq!(min, 10.5);
        assert_eq!(max, 20.3);
        assert!((avg_stat - 15.5).abs() < 0.01);
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process_csv(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let mut records = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|field| field.to_string()).collect();
            
            if Self::validate_row(&row) {
                records.push(row);
            } else {
                eprintln!("Warning: Invalid row skipped: {:?}", row);
            }
        }
        
        Ok(records)
    }

    fn validate_row(row: &[String]) -> bool {
        !row.is_empty() && row.iter().all(|field| !field.trim().is_empty())
    }

    pub fn calculate_statistics(data: &[Vec<String>]) -> Option<(f64, f64)> {
        if data.is_empty() {
            return None;
        }

        let numeric_values: Vec<f64> = data
            .iter()
            .filter_map(|row| row.get(0).and_then(|s| s.parse::<f64>().ok()))
            .collect();

        if numeric_values.is_empty() {
            return None;
        }

        let mean = numeric_values.iter().sum::<f64>() / numeric_values.len() as f64;
        let variance = numeric_values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / numeric_values.len() as f64;

        Some((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "value,description\n10.5,test1\n20.3,test2\ninvalid,test3\n").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process_csv().unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "10.5");
        assert_eq!(result[1][0], "20.3");
        
        let stats = DataProcessor::calculate_statistics(&result).unwrap();
        assert!((stats.0 - 15.4).abs() < 0.01);
        assert!((stats.1 - 4.9).abs() < 0.1);
    }
}
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue(f64),
    InvalidTimestamp(i64),
    EmptyCategory,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            ProcessingError::InvalidTimestamp(t) => write!(f, "Invalid timestamp: {}", t),
            ProcessingError::EmptyCategory => write!(f, "Category cannot be empty"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_enabled: bool,
    max_value_threshold: f64,
}

impl DataProcessor {
    pub fn new(validation_enabled: bool, max_value_threshold: f64) -> Self {
        DataProcessor {
            validation_enabled,
            max_value_threshold,
        }
    }

    pub fn process_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        if self.validation_enabled {
            self.validate_record(record)?;
        }

        let processed_record = self.transform_record(record);
        Ok(processed_record)
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < 0.0 || record.value > self.max_value_threshold {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp(record.timestamp));
        }

        if record.category.trim().is_empty() {
            return Err(ProcessingError::EmptyCategory);
        }

        Ok(())
    }

    fn transform_record(&self, record: &DataRecord) -> DataRecord {
        let normalized_value = if record.value > 100.0 {
            record.value / 10.0
        } else {
            record.value
        };

        let normalized_category = record.category.to_uppercase();

        DataRecord {
            id: record.id,
            value: normalized_value,
            timestamp: record.timestamp,
            category: normalized_category,
        }
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
    ) -> (Vec<DataRecord>, Vec<ProcessingError>) {
        let mut processed = Vec::new();
        let mut errors = Vec::new();

        for record in records {
            match self.process_record(&record) {
                Ok(processed_record) => processed.push(processed_record),
                Err(err) => errors.push(err),
            }
        }

        (processed, errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let processor = DataProcessor::new(true, 1000.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1625097600,
            category: "analytics".to_string(),
        };

        let result = processor.process_record(&record);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.category, "ANALYTICS");
    }

    #[test]
    fn test_invalid_value() {
        let processor = DataProcessor::new(true, 100.0);
        let record = DataRecord {
            id: 2,
            value: 150.0,
            timestamp: 1625097600,
            category: "test".to_string(),
        };

        let result = processor.process_record(&record);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProcessingError::InvalidValue(v) => assert_eq!(v, 150.0),
            _ => panic!("Expected InvalidValue error"),
        }
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(true, 500.0);
        let records = vec![
            DataRecord {
                id: 1,
                value: 100.0,
                timestamp: 1625097600,
                category: "data".to_string(),
            },
            DataRecord {
                id: 2,
                value: 600.0,
                timestamp: 1625097600,
                category: "invalid".to_string(),
            },
        ];

        let (processed, errors) = processor.batch_process(records);
        assert_eq!(processed.len(), 1);
        assert_eq!(errors.len(), 1);
    }
}