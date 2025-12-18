
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: u64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    TimestampOutOfRange,
    TransformationError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid data value"),
            DataError::TimestampOutOfRange => write!(f, "Timestamp out of valid range"),
            DataError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    validation_enabled: bool,
    max_value_threshold: f64,
}

impl DataProcessor {
    pub fn new(validation_enabled: bool, max_value_threshold: f64) -> Self {
        DataProcessor {
            validation_enabled,
            max_value_threshold,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if !self.validation_enabled {
            return Ok(());
        }

        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.value.is_nan() || record.value.is_infinite() {
            return Err(DataError::InvalidValue);
        }

        if record.value.abs() > self.max_value_threshold {
            return Err(DataError::InvalidValue);
        }

        if record.timestamp > 1_000_000_000_000 {
            return Err(DataError::TimestampOutOfRange);
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, DataError> {
        self.validate_record(record)?;

        let transformed_value = if record.value >= 0.0 {
            record.value.ln()
        } else {
            -record.value.abs().ln()
        };

        if transformed_value.is_nan() || transformed_value.is_infinite() {
            return Err(DataError::TransformationError(
                "Failed to compute logarithm".to_string(),
            ));
        }

        Ok(DataRecord {
            id: record.id,
            value: transformed_value,
            timestamp: record.timestamp,
        })
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> (Vec<DataRecord>, Vec<DataError>) {
        let mut processed = Vec::new();
        let mut errors = Vec::new();

        for record in records {
            match self.transform_record(&record) {
                Ok(transformed) => processed.push(transformed),
                Err(err) => errors.push(err),
            }
        }

        (processed, errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_validation() {
        let processor = DataProcessor::new(true, 1000.0);
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1_000_000,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id_validation() {
        let processor = DataProcessor::new(true, 1000.0);
        let record = DataRecord {
            id: 0,
            value: 42.5,
            timestamp: 1_000_000,
        };

        assert!(matches!(
            processor.validate_record(&record),
            Err(DataError::InvalidId)
        ));
    }

    #[test]
    fn test_record_transformation() {
        let processor = DataProcessor::new(true, 1000.0);
        let record = DataRecord {
            id: 1,
            value: std::f64::consts::E,
            timestamp: 1_000_000,
        };

        let transformed = processor.transform_record(&record).unwrap();
        assert!((transformed.value - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(true, 1000.0);
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 1_000_000,
            },
            DataRecord {
                id: 0,
                value: 20.0,
                timestamp: 1_000_000,
            },
            DataRecord {
                id: 3,
                value: -1.0,
                timestamp: 1_000_000,
            },
        ];

        let (processed, errors) = processor.process_batch(records);
        assert_eq!(processed.len(), 2);
        assert_eq!(errors.len(), 1);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = Reader::from_reader(file);

        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = if count > 0.0 { sum / count } else { 0.0 };
        
        let variance: f64 = if count > 0.0 {
            values.iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>() / count
        } else {
            0.0
        };
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
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
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.3,category_b").unwrap();
        writeln!(temp_file, "3,15.7,category_a").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        
        let (mean, variance, std_dev) = processor.calculate_statistics();
        assert!((mean - 15.5).abs() < 0.1);
        
        let filtered = processor.filter_by_category("category_a");
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
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::DuplicateTag => write!(f, "Tags contain duplicates"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let mut seen_tags = std::collections::HashSet::new();
        for tag in &self.tags {
            if !seen_tags.insert(tag) {
                return Err(ValidationError::DuplicateTag);
            }
        }
        
        Ok(())
    }
    
    pub fn normalize(&mut self) {
        self.name = self.name.trim().to_string();
        self.tags.sort();
        self.tags.dedup();
    }
}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
        }
    }
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        record.validate()?;
        
        if self.records.contains_key(&record.id) {
            return Err(format!("Record with ID {} already exists", record.id).into());
        }
        
        let mut normalized_record = record.clone();
        normalized_record.normalize();
        
        self.records.insert(normalized_record.id, normalized_record);
        Ok(())
    }
    
    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }
    
    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.values().map(|r| r.value).collect();
        
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
    
    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records.values()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }
    
    pub fn total_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -5.0,
            tags: vec!["duplicate".to_string(), "duplicate".to_string()],
        };
        
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: "Sample Data".to_string(),
            value: 100.0,
            tags: vec!["important".to_string(), "processed".to_string()],
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.total_records(), 1);
        
        let stats = processor.calculate_statistics();
        assert_eq!(stats.0, 100.0);
    }
}use csv::Reader;
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
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value < 0.0 {
            return Err(format!("Invalid negative value in record ID {}", record.id).into());
        }
        records.push(record);
    }

    if records.is_empty() {
        return Err("No valid records found in file".into());
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = sum / records.len() as f64;
    let max = records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);
    let category_count = records.iter().map(|r| &r.category).collect::<std::collections::HashSet<_>>().len();

    (avg, max, category_count)
}