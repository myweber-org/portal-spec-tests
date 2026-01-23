
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data value: {0}")]
    InvalidValue(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.is_empty() {
            return Err(ProcessingError::MissingField("name".to_string()));
        }

        if record.value < 0.0 {
            return Err(ProcessingError::InvalidValue(format!(
                "Negative value: {}",
                record.value
            )));
        }

        if record.value > self.threshold {
            return Err(ProcessingError::ValidationFailed(format!(
                "Value {} exceeds threshold {}",
                record.value, self.threshold
            )));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> DataRecord {
        let mut transformed = DataRecord {
            id: record.id,
            name: record.name.to_uppercase(),
            value: record.value * 2.0,
            metadata: record.metadata.clone(),
        };

        transformed
            .metadata
            .insert("processed".to_string(), "true".to_string());
        transformed
            .metadata
            .insert("original_value".to_string(), record.value.to_string());

        transformed
    }

    pub fn process_records(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());

        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record);
            processed_records.push(transformed);
        }

        Ok(processed_records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: 50.0,
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 150.0,
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(100.0);
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let record = DataRecord {
            id: 1,
            name: "example".to_string(),
            value: 25.5,
            metadata,
        };

        let transformed = processor.transform_record(&record);
        assert_eq!(transformed.name, "EXAMPLE");
        assert_eq!(transformed.value, 51.0);
        assert_eq!(transformed.metadata.get("processed"), Some(&"true".to_string()));
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn process_data(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value && record.active {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
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
    
    (sum, mean, std_dev)
}

fn filter_records(records: Vec<Record>, predicate: impl Fn(&Record) -> bool) -> Vec<Record> {
    records.into_iter()
        .filter(predicate)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, active: true },
            Record { id: 3, name: "Test3".to_string(), value: 30.0, active: false },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 5.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 15.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 25.0, active: true },
        ];
        
        let filtered = filter_records(records, |r| r.active && r.value > 10.0);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 3);
    }
}