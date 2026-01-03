
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: String,
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

        for result in rdr.records() {
            let record = result?;
            if record.len() >= 3 {
                let id: u32 = record[0].parse().unwrap_or(0);
                let value: f64 = record[1].parse().unwrap_or(0.0);
                let timestamp = record[2].to_string();

                if id > 0 && value >= 0.0 {
                    self.records.push(DataRecord {
                        id,
                        value,
                        timestamp,
                    });
                }
            }
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

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,timestamp").unwrap();
        writeln!(temp_file, "1,10.5,2023-01-01").unwrap();
        writeln!(temp_file, "2,20.3,2023-01-02").unwrap();
        writeln!(temp_file, "3,15.7,2023-01-03").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.5).abs() < 0.1);

        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 2);
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
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidId,
    InvalidValue,
    MissingName,
    DuplicateRecord,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidId => write!(f, "ID must be greater than 0"),
            ProcessingError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ProcessingError::MissingName => write!(f, "Name cannot be empty"),
            ProcessingError::DuplicateRecord => write!(f, "Record with this ID already exists"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    processed_count: u32,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            processed_count: 0,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(&record)?;
        
        if self.records.contains_key(&record.id) {
            return Err(ProcessingError::DuplicateRecord);
        }
        
        self.records.insert(record.id, record);
        self.processed_count += 1;
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn process_records(&mut self) -> Vec<DataRecord> {
        let mut processed = Vec::new();
        
        for (_, record) in self.records.drain() {
            let transformed = self.transform_record(record);
            processed.push(transformed);
        }
        
        processed
    }

    pub fn get_statistics(&self) -> (u32, f64, f64) {
        let count = self.records.len() as u32;
        let total_value: f64 = self.records.values().map(|r| r.value).sum();
        let avg_value = if count > 0 { total_value / count as f64 } else { 0.0 };
        
        (count, total_value, avg_value)
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.id == 0 {
            return Err(ProcessingError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(ProcessingError::MissingName);
        }
        
        if record.value < 0.0 || record.value > 1000.0 {
            return Err(ProcessingError::InvalidValue);
        }
        
        Ok(())
    }

    fn transform_record(&self, mut record: DataRecord) -> DataRecord {
        record.value = (record.value * 100.0).round() / 100.0;
        record.metadata.insert("processed".to_string(), "true".to_string());
        record.metadata.insert("processor_version".to_string(), "1.0".to_string());
        record
    }

    pub fn processed_count(&self) -> u32 {
        self.processed_count
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 50.5,
            metadata: HashMap::new(),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.processed_count(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            metadata: HashMap::new(),
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_duplicate_record() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord {
            id: 1,
            name: "Record 1".to_string(),
            value: 10.0,
            metadata: HashMap::new(),
        };
        
        let record2 = DataRecord {
            id: 1,
            name: "Record 2".to_string(),
            value: 20.0,
            metadata: HashMap::new(),
        };
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_err());
    }

    #[test]
    fn test_process_records() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.123,
            metadata: HashMap::new(),
        };
        
        processor.add_record(record).unwrap();
        let processed = processor.process_records();
        
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].value, 10.12);
        assert_eq!(processed[0].metadata.get("processed").unwrap(), "true");
    }
}