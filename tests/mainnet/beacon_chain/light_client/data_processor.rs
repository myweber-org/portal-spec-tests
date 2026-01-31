
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
    cache: HashMap<u32, DataRecord>,
}

#[derive(Clone)]
pub struct ProcessingConfig {
    pub max_value: f64,
    pub min_value: f64,
    pub allowed_tags: Vec<String>,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor {
            config,
            cache: HashMap::new(),
        }
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

    pub fn process_record(&mut self, record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;

        let transformed_value = self.transform_value(record.value)?;
        let normalized_name = self.normalize_name(&record.name);

        let processed_record = DataRecord {
            value: transformed_value,
            name: normalized_name,
            ..record
        };

        self.cache.insert(processed_record.id, processed_record.clone());
        Ok(processed_record)
    }

    fn transform_value(&self, value: f64) -> Result<f64, ProcessingError> {
        if value.is_nan() || value.is_infinite() {
            return Err(ProcessingError::TransformationFailed(
                "Cannot transform NaN or infinite values".to_string()
            ));
        }

        let transformed = (value * 100.0).round() / 100.0;
        Ok(transformed)
    }

    fn normalize_name(&self, name: &str) -> String {
        name.trim().to_lowercase()
    }

    pub fn get_cached_record(&self, id: u32) -> Option<&DataRecord> {
        self.cache.get(&id)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ProcessingConfig {
        ProcessingConfig {
            max_value: 1000.0,
            min_value: 0.0,
            allowed_tags: vec!["important".to_string(), "normal".to_string()],
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 500.0,
            tags: vec!["important".to_string()],
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 1500.0,
            tags: vec!["important".to_string()],
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_process_record() {
        let mut processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "  TEST  ".to_string(),
            value: 123.456,
            tags: vec!["normal".to_string()],
        };

        let result = processor.process_record(record);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.name, "test");
        assert_eq!(processed.value, 123.46);
        assert_eq!(processor.cache_size(), 1);
    }
}use csv::{Reader, Writer};
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

pub fn process_data(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value && record.active {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

pub fn calculate_statistics(path: &str) -> Result<(f64, f64, usize), Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);
    
    let mut sum = 0.0;
    let mut count = 0;
    let mut max_value = f64::MIN;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.active {
            sum += record.value;
            count += 1;
            if record.value > max_value {
                max_value = record.value;
            }
        }
    }

    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    Ok((average, max_value, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let input_data = "id,name,value,active\n1,Test1,10.5,true\n2,Test2,5.0,false\n3,Test3,15.0,true\n";
        
        let input_file = NamedTempFile::new().unwrap();
        std::fs::write(input_file.path(), input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        process_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            10.0
        ).unwrap();
        
        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(output_content.contains("Test1"));
        assert!(!output_content.contains("Test2"));
        assert!(output_content.contains("Test3"));
    }
}