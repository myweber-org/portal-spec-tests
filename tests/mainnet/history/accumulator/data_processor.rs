
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

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value < 0.0 || record.name.is_empty())
            .collect()
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, Option<f64>) {
        let count = self.records.len();
        let min = self.records.iter().map(|r| r.value).reduce(f64::min);
        let max = self.records.iter().map(|r| r.value).reduce(f64::max);
        (count, min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,category").unwrap();
        writeln!(file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(file, "2,ItemB,20.3,Category2").unwrap();
        writeln!(file, "3,ItemC,15.7,Category1").unwrap();
        file
    }

    #[test]
    fn test_load_and_filter() {
        let test_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(test_file.path().to_str().unwrap()).unwrap();
        
        let category1_items = processor.filter_by_category("Category1");
        assert_eq!(category1_items.len(), 2);
        
        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.5).abs() < 0.1);
    }

    #[test]
    fn test_validation() {
        let mut processor = DataProcessor::new();
        processor.records.push(Record {
            id: 1,
            name: "".to_string(),
            value: -5.0,
            category: "Test".to_string(),
        });
        
        let invalid = processor.validate_records();
        assert_eq!(invalid.len(), 1);
    }
}use std::collections::HashMap;
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

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
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

    pub fn transform_record(&self, record: DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        transformed.value = (transformed.value * 100.0).round() / 100.0;
        
        transformed.tags = transformed.tags
            .into_iter()
            .map(|tag| tag.to_lowercase())
            .collect();

        self.validate_record(&transformed)?;
        
        Ok(transformed)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut results = Vec::new();
        let mut error_count = 0;
        
        for record in records {
            match self.transform_record(record) {
                Ok(transformed) => results.push(transformed),
                Err(e) => {
                    error_count += 1;
                    if error_count > 3 {
                        return Err(ProcessingError::TransformationFailed(
                            "Too many errors in batch".to_string()
                        ));
                    }
                }
            }
        }
        
        Ok(results)
    }

    pub fn aggregate_by_tag(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut aggregates = HashMap::new();
        
        for record in records {
            for tag in &record.tags {
                *aggregates.entry(tag.clone()).or_insert(0.0) += record.value;
            }
        }
        
        aggregates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ProcessingConfig {
        ProcessingConfig {
            max_value: 1000.0,
            min_value: 0.0,
            allowed_tags: vec!["active".to_string(), "pending".to_string(), "completed".to_string()],
        }
    }

    #[test]
    fn test_validate_record_valid() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 500.0,
            tags: vec!["active".to_string()],
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_record_invalid_tag() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 500.0,
            tags: vec!["invalid".to_string()],
        };
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 123.456,
            tags: vec!["ACTIVE".to_string()],
        };
        
        let transformed = processor.transform_record(record).unwrap();
        assert_eq!(transformed.value, 123.46);
        assert_eq!(transformed.tags, vec!["active"]);
    }

    #[test]
    fn test_aggregate_by_tag() {
        let processor = DataProcessor::new(create_test_config());
        let records = vec![
            DataRecord {
                id: 1,
                name: "A".to_string(),
                value: 100.0,
                tags: vec!["active".to_string()],
            },
            DataRecord {
                id: 2,
                name: "B".to_string(),
                value: 200.0,
                tags: vec!["active".to_string(), "pending".to_string()],
            },
        ];
        
        let aggregates = processor.aggregate_by_tag(&records);
        assert_eq!(aggregates.get("active"), Some(&300.0));
        assert_eq!(aggregates.get("pending"), Some(&200.0));
    }
}