
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

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 && self.has_header {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String], expected_fields: usize) -> bool {
        record.len() == expected_fields && 
        record.iter().all(|field| !field.is_empty())
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
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Jane,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data1".to_string(), "data2".to_string()];
        let invalid_record = vec!["".to_string(), "data2".to_string()];

        assert!(processor.validate_record(&valid_record, 2));
        assert!(!processor.validate_record(&invalid_record, 2));
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
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Vec<String> {
        records
            .iter()
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
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&records, 1);

        assert_eq!(column, vec!["b".to_string(), "e".to_string()]);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    config: ProcessingConfig,
}

pub struct ProcessingConfig {
    pub max_value: f64,
    pub min_value: f64,
    pub allowed_tags: Vec<String>,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn process_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(record)?;
        let transformed = self.transform_record(record)?;
        Ok(transformed)
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value > self.config.max_value {
            return Err(ProcessingError::ValidationError(
                format!("Value {} exceeds maximum {}", record.value, self.config.max_value)
            ));
        }

        if record.value < self.config.min_value {
            return Err(ProcessingError::ValidationError(
                format!("Value {} below minimum {}", record.value, self.config.min_value)
            ));
        }

        for tag in &record.tags {
            if !self.config.allowed_tags.contains(tag) {
                return Err(ProcessingError::ValidationError(
                    format!("Tag '{}' is not allowed", tag)
                ));
            }
        }

        Ok(())
    }

    fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let normalized_value = if record.value > 0.0 {
            (record.value / self.config.max_value) * 100.0
        } else {
            0.0
        };

        let mut transformed_tags = record.tags.clone();
        transformed_tags.sort();
        transformed_tags.dedup();

        Ok(DataRecord {
            id: record.id,
            name: record.name.to_uppercase(),
            value: (normalized_value * 100.0).round() / 100.0,
            tags: transformed_tags,
        })
    }

    pub fn batch_process(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());
        
        for record in records {
            match self.process_record(&record) {
                Ok(processed) => results.push(processed),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }

    pub fn create_summary(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut summary = HashMap::new();
        
        if records.is_empty() {
            return summary;
        }

        let total: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len() as f64;
        
        summary.insert("total_records".to_string(), count);
        summary.insert("total_value".to_string(), total);
        summary.insert("average_value".to_string(), total / count);
        
        if let Some(max) = records.iter().map(|r| r.value).reduce(f64::max) {
            summary.insert("max_value".to_string(), max);
        }
        
        if let Some(min) = records.iter().map(|r| r.value).reduce(f64::min) {
            summary.insert("min_value".to_string(), min);
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ProcessingConfig {
        ProcessingConfig {
            max_value: 1000.0,
            min_value: 0.0,
            allowed_tags: vec!["important".to_string(), "normal".to_string(), "test".to_string()],
        }
    }

    #[test]
    fn test_valid_record_processing() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "test record".to_string(),
            value: 500.0,
            tags: vec!["important".to_string(), "test".to_string()],
        };

        let result = processor.process_record(&record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.name, "TEST RECORD");
        assert_eq!(processed.tags, vec!["important".to_string(), "test".to_string()]);
    }

    #[test]
    fn test_invalid_value_validation() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "invalid record".to_string(),
            value: 1500.0,
            tags: vec!["test".to_string()],
        };

        let result = processor.process_record(&record);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_tag_validation() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "invalid tag record".to_string(),
            value: 100.0,
            tags: vec!["invalid".to_string()],
        };

        let result = processor.process_record(&record);
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(create_test_config());
        let records = vec![
            DataRecord {
                id: 1,
                name: "record one".to_string(),
                value: 100.0,
                tags: vec!["normal".to_string()],
            },
            DataRecord {
                id: 2,
                name: "record two".to_string(),
                value: 200.0,
                tags: vec!["important".to_string()],
            },
        ];

        let result = processor.batch_process(records);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_summary_creation() {
        let processor = DataProcessor::new(create_test_config());
        let records = vec![
            DataRecord {
                id: 1,
                name: "record one".to_string(),
                value: 100.0,
                tags: vec![],
            },
            DataRecord {
                id: 2,
                name: "record two".to_string(),
                value: 200.0,
                tags: vec![],
            },
        ];

        let summary = processor.create_summary(&records);
        assert_eq!(summary.get("total_records").unwrap(), &2.0);
        assert_eq!(summary.get("total_value").unwrap(), &300.0);
        assert_eq!(summary.get("average_value").unwrap(), &150.0);
    }
}