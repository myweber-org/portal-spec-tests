use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn process(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn batch_process(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records.into_iter().map(|record| self.process(record)).collect()
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validation_rule(|record| {
        if record.values.is_empty() {
            Err(ProcessingError::InvalidData("Record has no values".to_string()))
        } else {
            Ok(())
        }
    });

    processor.add_validation_rule(|record| {
        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::InvalidData(
                    "Record contains invalid numeric values".to_string(),
                ));
            }
        }
        Ok(())
    });

    processor.add_transformation(|mut record| {
        let sum: f64 = record.values.iter().sum();
        let count = record.values.len() as f64;
        let average = sum / count;

        record.metadata.insert("average".to_string(), average.to_string());
        record.metadata.insert("count".to_string(), count.to_string());

        Ok(record)
    });

    processor.add_transformation(|mut record| {
        record.values = record.values.into_iter().map(|v| v * 100.0).collect();
        Ok(record)
    });

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let processor = create_default_processor();

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let record = DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata,
        };

        let result = processor.process(record);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.values, vec![1000.0, 2000.0, 3000.0]);
        assert_eq!(processed.metadata.get("average").unwrap(), "20");
        assert_eq!(processed.metadata.get("count").unwrap(), "3");
    }

    #[test]
    fn test_invalid_data() {
        let processor = create_default_processor();

        let record = DataRecord {
            id: 2,
            values: vec![f64::NAN, 20.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(record);
        assert!(result.is_err());
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
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() && !record.iter().all(|field| field.is_empty()) {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>], expected_columns: usize) -> Vec<usize> {
        let mut invalid_rows = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if record.len() != expected_columns {
                invalid_rows.push(index);
            }
        }
        
        invalid_rows
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
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();
        writeln!(temp_file, "Bob,35,Tokyo").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_record_validation() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["x".to_string(), "y".to_string()],
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let invalid = processor.validate_records(&records, 3);
        
        assert_eq!(invalid, vec![1]);
    }

    #[test]
    fn test_column_extraction() {
        let records = vec![
            vec!["John".to_string(), "30".to_string()],
            vec!["Alice".to_string(), "25".to_string()],
            vec!["Bob".to_string(), "35".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let names = processor.extract_column(&records, 0);
        
        assert_eq!(names, vec!["John", "Alice", "Bob"]);
    }
}