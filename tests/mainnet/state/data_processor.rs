
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
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
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

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        for rule in &self.validation_rules {
            rule(record)?;
        }
        Ok(())
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());

        for record in records {
            match self.process_record(record) {
                Ok(processed) => processed_records.push(processed),
                Err(e) => return Err(e),
            }
        }

        Ok(processed_records)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validation_rule(|record| {
        if record.name.is_empty() {
            Err(ProcessingError::ValidationError("Name cannot be empty".to_string()))
        } else {
            Ok(())
        }
    });

    processor.add_validation_rule(|record| {
        if record.value < 0.0 {
            Err(ProcessingError::ValidationError("Value cannot be negative".to_string()))
        } else {
            Ok(())
        }
    });

    processor.add_transformation(|mut record| {
        record.value = (record.value * 100.0).round() / 100.0;
        Ok(record)
    });

    processor.add_transformation(|mut record| {
        record.name = record.name.to_uppercase();
        Ok(record)
    });

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        let valid_record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: 10.5,
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&valid_record).is_ok());

        let invalid_record = DataRecord {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_processing() {
        let processor = create_default_processor();
        let record = DataRecord {
            id: 1,
            name: "example".to_string(),
            value: 12.3456,
            metadata: HashMap::new(),
        };

        let processed = processor.process_record(record).unwrap();
        assert_eq!(processed.name, "EXAMPLE");
        assert_eq!(processed.value, 12.35);
    }
}