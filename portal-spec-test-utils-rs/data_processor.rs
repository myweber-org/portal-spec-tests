
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
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

            let name = parts[1].to_string();
            
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let active = match parts[3].to_lowercase().as_str() {
                "true" | "1" => true,
                "false" | "0" => false,
                _ => continue,
            };

            let record = Record::new(id, name, value, active);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records
            .iter()
            .map(|record| record.value)
            .sum()
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&Record> {
        self.records
            .iter()
            .find(|record| record.id == target_id)
    }

    pub fn record_count(&self) -> usize {
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
        let valid_record = Record::new(1, "Test".to_string(), 100.0, true);
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -50.0, false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut csv_content = "id,name,value,active\n".to_string();
        csv_content.push_str("1,ItemA,100.5,true\n");
        csv_content.push_str("2,ItemB,75.3,false\n");
        csv_content.push_str("3,ItemC,200.0,true\n");

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.filter_active().len(), 2);
        assert_eq!(processor.calculate_total(), 375.8);
        
        let found = processor.find_by_id(2);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "ItemB");
    }
}
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
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
        assert_eq!(result[1], vec!["Bob", "25", "London"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data".to_string(), "value".to_string()];
        let invalid_record = vec!["".to_string(), "value".to_string()];
        
        assert!(processor.validate_record(&valid_record));
        assert!(!processor.validate_record(&invalid_record));
    }

    #[test]
    fn test_extract_column() {
        let processor = DataProcessor::new(',', false);
        let data = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let column = processor.extract_column(&data, 1);
        assert_eq!(column, vec!["b".to_string(), "e".to_string()]);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_threshold: f64,
    max_records: usize,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, max_records: usize) -> Self {
        Self {
            validation_threshold,
            max_records,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "Empty values array".to_string(),
            ));
        }

        for value in &record.values {
            if !value.is_finite() {
                return Err(ProcessingError::ValidationFailed(
                    "Non-finite value detected".to_string(),
                ));
            }
        }

        if record.values.len() > self.max_records {
            return Err(ProcessingError::ValidationFailed(format!(
                "Exceeds maximum record size: {} > {}",
                record.values.len(),
                self.max_records
            )));
        }

        Ok(())
    }

    pub fn transform_data(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut transformed = Vec::with_capacity(records.len());

        for mut record in records {
            self.validate_record(&record)?;

            let mut processed_values = Vec::with_capacity(record.values.len());
            for value in &record.values {
                let transformed_value = if *value > self.validation_threshold {
                    value.ln()
                } else {
                    *value
                };
                processed_values.push(transformed_value);
            }

            record.values = processed_values;
            record
                .metadata
                .insert("processed".to_string(), "true".to_string());
            transformed.push(record);
        }

        Ok(transformed)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);

        let all_values: Vec<f64> = records
            .iter()
            .flat_map(|r| r.values.iter().copied())
            .collect();

        if !all_values.is_empty() {
            let sum: f64 = all_values.iter().sum();
            let mean = sum / all_values.len() as f64;
            let variance: f64 = all_values.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                / all_values.len() as f64;

            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert(
                "min".to_string(),
                all_values
                    .iter()
                    .fold(f64::INFINITY, |a, &b| a.min(b)),
            );
            stats.insert(
                "max".to_string(),
                all_values
                    .iter()
                    .fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            );
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(10.0, 100);
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
        let processor = DataProcessor::new(10.0, 100);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_data() {
        let processor = DataProcessor::new(2.0, 100);
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1234567890,
                values: vec![1.0, 3.0, 2.5],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 1234567891,
                values: vec![0.5, 1.5],
                metadata: HashMap::new(),
            },
        ];

        let result = processor.transform_data(records);
        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed.len(), 2);
        assert_eq!(
            transformed[0].metadata.get("processed"),
            Some(&"true".to_string())
        );
    }
}