
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
        let data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&data, 1);
        
        assert_eq!(column, vec!["b".to_string(), "d".to_string()]);
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
    pub category: String,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidValue(String),
    MissingField(String),
    ParseError(String),
    ValidationError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            DataError::MissingField(field) => write!(f, "Missing required field: {}", field),
            DataError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            DataError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    validation_rules: HashMap<String, Box<dyn Fn(&DataRecord) -> Result<(), DataError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, DataError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: HashMap::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, name: &str, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), DataError> + 'static,
    {
        self.validation_rules.insert(name.to_string(), Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, DataError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), Vec<DataError>> {
        let mut errors = Vec::new();

        for (rule_name, rule) in &self.validation_rules {
            if let Err(err) = rule(record) {
                errors.push(err);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, DataError> {
        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }
        Ok(record)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> (Vec<DataRecord>, Vec<DataError>) {
        let mut processed = Vec::new();
        let mut errors = Vec::new();

        for record in records {
            match self.validate_record(&record) {
                Ok(_) => match self.process_record(record) {
                    Ok(processed_record) => processed.push(processed_record),
                    Err(err) => errors.push(err),
                },
                Err(validation_errors) => {
                    errors.extend(validation_errors);
                }
            }
        }

        (processed, errors)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validation_rule("value_positive", |record| {
        if record.value < 0.0 {
            Err(DataError::InvalidValue(format!(
                "Value must be positive, got {}",
                record.value
            )))
        } else {
            Ok(())
        }
    });

    processor.add_validation_rule("name_not_empty", |record| {
        if record.name.trim().is_empty() {
            Err(DataError::InvalidValue("Name cannot be empty".to_string()))
        } else {
            Ok(())
        }
    });

    processor.add_transformation(|mut record| {
        record.name = record.name.trim().to_uppercase();
        Ok(record)
    });

    processor.add_transformation(|mut record| {
        if record.category.is_empty() {
            record.category = "DEFAULT".to_string();
        }
        Ok(record)
    });

    processor
}

pub fn parse_record_from_csv(line: &str) -> Result<DataRecord, DataError> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 5 {
        return Err(DataError::MissingField("Insufficient fields in CSV".to_string()));
    }

    let id = parts[0]
        .parse::<u32>()
        .map_err(|_| DataError::ParseError("Invalid ID format".to_string()))?;

    let name = parts[1].to_string();

    let value = parts[2]
        .parse::<f64>()
        .map_err(|_| DataError::ParseError("Invalid value format".to_string()))?;

    let category = parts[3].to_string();

    let timestamp = parts[4]
        .parse::<i64>()
        .map_err(|_| DataError::ParseError("Invalid timestamp format".to_string()))?;

    Ok(DataRecord {
        id,
        name,
        value,
        category,
        timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let processor = create_default_processor();

        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
            timestamp: 1234567890,
        };

        assert!(processor.validate_record(&valid_record).is_ok());

        let invalid_record = DataRecord {
            id: 2,
            name: "".to_string(),
            value: -50.0,
            category: "B".to_string(),
            timestamp: 1234567890,
        };

        let result = processor.validate_record(&invalid_record);
        assert!(result.is_err());
    }

    #[test]
    fn test_record_processing() {
        let processor = create_default_processor();

        let record = DataRecord {
            id: 1,
            name: "  test  ".to_string(),
            value: 50.0,
            category: "".to_string(),
            timestamp: 1234567890,
        };

        let processed = processor.process_record(record).unwrap();
        assert_eq!(processed.name, "TEST");
        assert_eq!(processed.category, "DEFAULT");
    }

    #[test]
    fn test_csv_parsing() {
        let csv_line = "123,Product Name,45.67,Electronics,1672531200";
        let record = parse_record_from_csv(csv_line).unwrap();

        assert_eq!(record.id, 123);
        assert_eq!(record.name, "Product Name");
        assert_eq!(record.value, 45.67);
        assert_eq!(record.category, "Electronics");
        assert_eq!(record.timestamp, 1672531200);
    }
}