
use csv;
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

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
    
    fn process(&mut self) {
        self.name = self.name.to_uppercase();
        self.value = (self.value * 100.0).round() / 100.0;
    }
}

pub fn load_and_process_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let mut records: Vec<Record> = Vec::new();
    
    for result in rdr.deserialize() {
        let mut record: Record = result?;
        
        if record.is_valid() {
            record.process();
            records.push(record);
        }
    }
    
    Ok(records)
}

pub fn save_processed_data(records: &[Record], output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut wtr = csv::Writer::from_writer(file);
    
    for record in records {
        wtr.serialize(record)?;
    }
    
    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "test".to_string(),
            value: 10.5,
            active: true,
        };
        
        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            active: false,
        };
        
        assert!(valid_record.is_valid());
        assert!(!invalid_record.is_valid());
    }
    
    #[test]
    fn test_record_processing() {
        let mut record = Record {
            id: 1,
            name: "hello".to_string(),
            value: 12.3456,
            active: true,
        };
        
        record.process();
        
        assert_eq!(record.name, "HELLO");
        assert_eq!(record.value, 12.35);
    }
}use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
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
}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(DataError::ValidationError(
                "Value must be a finite number".to_string(),
            ));
        }

        if record.timestamp < 0 {
            return Err(DataError::ValidationError(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        Ok(())
    }

    pub fn process_record(&self, record: DataRecord) -> Result<DataRecord, DataError> {
        self.validate_record(&record)?;

        let processed_value = if record.value > self.threshold {
            record.value * 0.9
        } else {
            record.value * 1.1
        };

        let processed_record = DataRecord {
            value: processed_value,
            ..record
        };

        Ok(processed_record)
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, Vec<DataError>> {
        let mut results = Vec::new();
        let mut errors = Vec::new();

        for record in records {
            match self.process_record(record) {
                Ok(processed) => results.push(processed),
                Err(e) => errors.push(e),
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(results)
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
            value: 50.0,
            timestamp: 1234567890,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: f64::NAN,
            timestamp: 1234567890,
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_process_record_above_threshold() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1234567890,
        };

        let result = processor.process_record(record).unwrap();
        assert_eq!(result.value, 135.0);
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(100.0);
        let records = vec![
            DataRecord {
                id: 1,
                value: 50.0,
                timestamp: 1234567890,
            },
            DataRecord {
                id: 2,
                value: 150.0,
                timestamp: 1234567891,
            },
        ];

        let results = processor.batch_process(records).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].value, 55.0);
        assert_eq!(results[1].value, 135.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if !fields.is_empty() && !fields[0].is_empty() {
                records.push(fields);
            }
        }

        if records.is_empty() {
            return Err("No valid records found".into());
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process().unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new("dummy.csv");
        
        assert!(processor.validate_record(&["test".to_string(), "data".to_string()]));
        assert!(!processor.validate_record(&[]));
        assert!(!processor.validate_record(&["".to_string(), "value".to_string()]));
    }
}