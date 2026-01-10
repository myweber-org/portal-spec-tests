
use std::error::Error;
use std::fs::File;
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn validate_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= 0.0 && record.value <= 1000.0)
            .collect()
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
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,42.5,alpha").unwrap();
        writeln!(temp_file, "2,78.9,beta").unwrap();
        writeln!(temp_file, "3,150.2,alpha").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let avg = processor.calculate_average().unwrap();
        assert!((avg - 90.533).abs() < 0.001);
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
        
        let valid_records = processor.validate_records();
        assert_eq!(valid_records.len(), 3);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(i64),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    ValueOutOfRange(f64),
    #[error("Duplicate record ID: {0}")]
    DuplicateId(u64),
}

pub struct DataProcessor {
    validation_enabled: bool,
    seen_ids: std::collections::HashSet<u64>,
}

impl DataProcessor {
    pub fn new(validation_enabled: bool) -> Self {
        Self {
            validation_enabled,
            seen_ids: std::collections::HashSet::new(),
        }
    }

    pub fn process_record(&mut self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        if self.validation_enabled {
            self.validate_record(record)?;
        }

        let processed = self.transform_record(record);
        Ok(processed)
    }

    fn validate_record(&mut self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp(record.timestamp));
        }

        if record.values.is_empty() {
            return Err(ProcessingError::MissingField("values".to_string()));
        }

        for (key, &value) in &record.values {
            if !value.is_finite() {
                return Err(ProcessingError::ValueOutOfRange(value));
            }
            if key.trim().is_empty() {
                return Err(ProcessingError::MissingField("key".to_string()));
            }
        }

        if self.seen_ids.contains(&record.id) {
            return Err(ProcessingError::DuplicateId(record.id));
        }
        self.seen_ids.insert(record.id);

        Ok(())
    }

    fn transform_record(&self, record: &DataRecord) -> DataRecord {
        let mut transformed_values = HashMap::new();
        
        for (key, value) in &record.values {
            let transformed_key = key.to_lowercase().replace(' ', "_");
            let transformed_value = if *value < 0.0 {
                0.0
            } else {
                *value
            };
            transformed_values.insert(transformed_key, transformed_value);
        }

        let mut sorted_tags = record.tags.clone();
        sorted_tags.sort();
        sorted_tags.dedup();

        DataRecord {
            id: record.id,
            timestamp: record.timestamp,
            values: transformed_values,
            tags: sorted_tags,
        }
    }

    pub fn reset(&mut self) {
        self.seen_ids.clear();
    }

    pub fn processed_count(&self) -> usize {
        self.seen_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let mut processor = DataProcessor::new(true);
        let mut values = HashMap::new();
        values.insert("Temperature".to_string(), 25.5);
        values.insert("Pressure".to_string(), 1013.25);

        let record = DataRecord {
            id: 1,
            timestamp: 1672531200,
            values,
            tags: vec!["sensor".to_string(), "room1".to_string()],
        };

        let result = processor.process_record(&record);
        assert!(result.is_ok());
        assert_eq!(processor.processed_count(), 1);
    }

    #[test]
    fn test_invalid_timestamp() {
        let mut processor = DataProcessor::new(true);
        let values = HashMap::from([("test".to_string(), 1.0)]);
        
        let record = DataRecord {
            id: 1,
            timestamp: -1,
            values,
            tags: vec![],
        };

        let result = processor.process_record(&record);
        assert!(matches!(result, Err(ProcessingError::InvalidTimestamp(-1))));
    }

    #[test]
    fn test_duplicate_id() {
        let mut processor = DataProcessor::new(true);
        let values = HashMap::from([("data".to_string(), 42.0)]);

        let record1 = DataRecord {
            id: 100,
            timestamp: 1672531200,
            values: values.clone(),
            tags: vec![],
        };

        let record2 = DataRecord {
            id: 100,
            timestamp: 1672531201,
            values,
            tags: vec![],
        };

        let _ = processor.process_record(&record1);
        let result = processor.process_record(&record2);
        assert!(matches!(result, Err(ProcessingError::DuplicateId(100))));
    }
}