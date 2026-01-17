
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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
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

    pub fn validate_records(&self, records: &[Vec<String>]) -> Vec<usize> {
        let mut invalid_indices = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if record.is_empty() || record.iter().any(|field| field.is_empty()) {
                invalid_indices.push(index);
            }
        }
        
        invalid_indices
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
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["".to_string(), "c".to_string()],
            vec!["d".to_string(), "".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let invalid = processor.validate_records(&records);
        
        assert_eq!(invalid, vec![1, 2]);
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&records, 1);
        
        assert_eq!(column, vec!["b", "e"]);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn register_validator(&mut self, name: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validators.insert(name.to_string(), validator);
    }

    pub fn register_transformer(&mut self, name: &str, transformer: Box<dyn Fn(String) -> String>) {
        self.transformers.insert(name.to_string(), transformer);
    }

    pub fn validate(&self, name: &str, data: &str) -> bool {
        match self.validators.get(name) {
            Some(validator) => validator(data),
            None => false,
        }
    }

    pub fn transform(&self, name: &str, data: String) -> Option<String> {
        self.transformers.get(name).map(|transformer| transformer(data))
    }

    pub fn process_pipeline(&self, data: &str, validators: &[&str], transformers: &[&str]) -> Option<String> {
        for validator_name in validators {
            if !self.validate(validator_name, data) {
                return None;
            }
        }

        let mut result = data.to_string();
        for transformer_name in transformers {
            match self.transform(transformer_name, result) {
                Some(transformed) => result = transformed,
                None => return None,
            }
        }

        Some(result)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.register_validator("non_empty", Box::new(|s| !s.trim().is_empty()));
    processor.register_validator("is_numeric", Box::new(|s| s.chars().all(|c| c.is_ascii_digit())));

    processor.register_transformer("uppercase", Box::new(|s| s.to_uppercase()));
    processor.register_transformer("trim", Box::new(|s| s.trim().to_string()));
    processor.register_transformer("reverse", Box::new(|s| s.chars().rev().collect()));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("non_empty", "hello"));
        assert!(!processor.validate("non_empty", "   "));
        assert!(processor.validate("is_numeric", "12345"));
        assert!(!processor.validate("is_numeric", "123a"));
    }

    #[test]
    fn test_transformation() {
        let processor = create_default_processor();
        assert_eq!(processor.transform("uppercase", "hello".to_string()), Some("HELLO".to_string()));
        assert_eq!(processor.transform("trim", "  hello  ".to_string()), Some("hello".to_string()));
        assert_eq!(processor.transform("reverse", "abc".to_string()), Some("cba".to_string()));
    }

    #[test]
    fn test_pipeline() {
        let processor = create_default_processor();
        let result = processor.process_pipeline("  hello  ", &["non_empty"], &["trim", "uppercase"]);
        assert_eq!(result, Some("HELLO".to_string()));

        let invalid_result = processor.process_pipeline("   ", &["non_empty"], &["trim"]);
        assert_eq!(invalid_result, None);
    }
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
    MissingField,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataError::InvalidValue => write!(f, "Invalid data value"),
            DataError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            DataError::MissingField => write!(f, "Missing required field"),
        }
    }
}

impl Error for DataError {}

pub fn validate_record(record: &DataRecord) -> Result<(), DataError> {
    if record.value.is_nan() || record.value.is_infinite() {
        return Err(DataError::InvalidValue);
    }
    
    if record.timestamp < 0 {
        return Err(DataError::InvalidTimestamp);
    }
    
    Ok(())
}

pub fn transform_record(record: DataRecord, multiplier: f64) -> DataRecord {
    DataRecord {
        value: record.value * multiplier,
        ..record
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<Result<DataRecord, DataError>> {
    records
        .into_iter()
        .map(|record| {
            validate_record(&record)?;
            Ok(transform_record(record, 2.0))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1625097600,
        };
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_record_invalid_value() {
        let record = DataRecord {
            id: 2,
            value: f64::NAN,
            timestamp: 1625097600,
        };
        assert!(matches!(validate_record(&record), Err(DataError::InvalidValue)));
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord {
            id: 3,
            value: 10.0,
            timestamp: 1625097600,
        };
        let transformed = transform_record(record, 3.0);
        assert_eq!(transformed.value, 30.0);
    }
}use std::error::Error;
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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            let _ = lines.next();
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

    pub fn validate_records(&self, records: &[Vec<String>]) -> Vec<usize> {
        let mut invalid_indices = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if record.is_empty() || record.iter().any(|field| field.is_empty()) {
                invalid_indices.push(index);
            }
        }
        
        invalid_indices
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
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["".to_string(), "c".to_string()],
            vec!["d".to_string()],
            vec![],
        ];
        
        let processor = DataProcessor::new(',', false);
        let invalid = processor.validate_records(&records);
        
        assert_eq!(invalid, vec![1, 3]);
    }
}