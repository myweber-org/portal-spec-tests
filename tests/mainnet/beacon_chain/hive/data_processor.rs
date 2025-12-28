
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
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::DuplicateTag => write!(f, "Tags must be unique"),
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
        
        let mut seen_tags = HashMap::new();
        for tag in &self.tags {
            if seen_tags.insert(tag, true).is_some() {
                return Err(ValidationError::DuplicateTag);
            }
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, ValidationError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        let mut transformed = record.clone();
        transformed.transform(multiplier);
        processed.push(transformed);
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec![],
        };
        
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_transform() {
        let mut record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: 50.0,
            tags: vec![],
        };
        
        record.transform(2.0);
        assert_eq!(record.value, 100.0);
        assert_eq!(record.name, "TEST");
    }
    
    #[test]
    fn test_statistics() {
        let records = vec![
            DataRecord { id: 1, name: "A".to_string(), value: 10.0, tags: vec![] },
            DataRecord { id: 2, name: "B".to_string(), value: 20.0, tags: vec![] },
            DataRecord { id: 3, name: "C".to_string(), value: 30.0, tags: vec![] },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }
}use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataProcessor {
    data: Vec<Vec<String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        for result in rdr.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            self.data.push(row);
        }
        
        Ok(())
    }

    pub fn filter_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        self.data
            .iter()
            .filter(|row| predicate(row))
            .cloned()
            .collect()
    }

    pub fn get_column(&self, column_index: usize) -> Vec<String> {
        self.data
            .iter()
            .filter_map(|row| row.get(column_index).cloned())
            .collect()
    }

    pub fn row_count(&self) -> usize {
        self.data.len()
    }

    pub fn column_count(&self) -> usize {
        self.data.first().map_or(0, |row| row.len())
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.row_count(), 3);
        assert_eq!(processor.column_count(), 3);
        
        let filtered = processor.filter_rows(|row| {
            row.get(1).and_then(|age| age.parse::<i32>().ok()).map_or(false, |age| age > 30)
        });
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0][0], "Charlie");
        
        let ages = processor.get_column(1);
        assert_eq!(ages, vec!["30", "25", "35"]);
    }
}
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

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let max_value = records.iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, |a, b| a.max(b));
    
    (average, max_value, count)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    total_value: f64,
    valid_count: usize,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            total_value: 0.0,
            valid_count: 0,
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }
            
            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let category = parts[2].to_string();
            
            let record = DataRecord::new(id, value, category);
            self.add_record(record);
        }
        
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        if record.is_valid() {
            self.total_value += record.get_value();
            self.valid_count += 1;
        }
        self.records.push(record);
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.valid_count > 0 {
            Some(self.total_value / self.valid_count as f64)
        } else {
            None
        }
    }

    pub fn get_valid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn get_invalid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| !r.is_valid()).collect()
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert!(record.is_valid());
        assert_eq!(record.get_value(), 42.5);
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -10.0, "".to_string());
        assert!(!record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(1, 10.0, "A".to_string());
        let record2 = DataRecord::new(2, 20.0, "B".to_string());
        
        processor.add_record(record1);
        processor.add_record(record2);
        
        assert_eq!(processor.total_records(), 2);
        assert_eq!(processor.calculate_average(), Some(15.0));
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,20.3,TypeB").unwrap();
        writeln!(temp_file, "3,-5.0,Invalid").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.total_records(), 2);
        assert_eq!(processor.get_valid_records().len(), 2);
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
pub enum DataError {
    InvalidId,
    InvalidName,
    InvalidValue,
    EmptyTags,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidName => write!(f, "Name cannot be empty"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyTags => write!(f, "At least one tag is required"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(DataError::InvalidName);
        }
        
        if !(0.0..=1000.0).contains(&record.value) {
            return Err(DataError::InvalidValue);
        }
        
        if record.tags.is_empty() {
            return Err(DataError::EmptyTags);
        }
        
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        self.validate_record(&record)?;
        
        if self.records.contains_key(&record.id) {
            return Err(format!("Record with ID {} already exists", record.id).into());
        }
        
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn transform_values(&mut self, transform_fn: fn(f64) -> f64) {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.tags.iter().any(|t| t == tag))
            .collect()
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.values().map(|r| r.value).collect();
        
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn merge_tags(&mut self) -> HashMap<String, Vec<u32>> {
        let mut tag_map: HashMap<String, Vec<u32>> = HashMap::new();
        
        for record in self.records.values() {
            for tag in &record.tags {
                tag_map
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(record.id);
            }
        }
        
        tag_map
    }
}

pub fn normalize_value(value: f64) -> f64 {
    (value / 1000.0).clamp(0.0, 1.0)
}

pub fn scale_value(value: f64, factor: f64) -> f64 {
    value * factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = DataProcessor::new();
        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 500.0,
            tags: vec!["tag1".to_string()],
        };
        
        assert!(processor.validate_record(&valid_record).is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let processor = DataProcessor::new();
        let invalid_record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            tags: vec![],
        };
        
        assert!(processor.validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_transform_values() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec!["test".to_string()],
        };
        
        processor.add_record(record).unwrap();
        processor.transform_values(|v| v * 2.0);
        
        let updated = processor.get_record(1).unwrap();
        assert_eq!(updated.value, 200.0);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    ValueOutOfRange(f64),
    MetadataKeyTooLong(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "Record ID must be greater than zero"),
            ValidationError::InvalidTimestamp => write!(f, "Timestamp cannot be negative"),
            ValidationError::EmptyValues => write!(f, "Values vector cannot be empty"),
            ValidationError::ValueOutOfRange(val) => write!(f, "Value {} is out of acceptable range", val),
            ValidationError::MetadataKeyTooLong(key) => write!(f, "Metadata key '{}' exceeds maximum length", key),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    max_value: f64,
    min_value: f64,
    max_key_length: usize,
}

impl DataProcessor {
    pub fn new(max_value: f64, min_value: f64, max_key_length: usize) -> Self {
        DataProcessor {
            max_value,
            min_value,
            max_key_length,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }

        if record.timestamp < 0 {
            return Err(ValidationError::InvalidTimestamp);
        }

        if record.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }

        for &value in &record.values {
            if value < self.min_value || value > self.max_value {
                return Err(ValidationError::ValueOutOfRange(value));
            }
        }

        for key in record.metadata.keys() {
            if key.len() > self.max_key_length {
                return Err(ValidationError::MetadataKeyTooLong(key.clone()));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &DataRecord) -> Vec<f64> {
        if record.values.is_empty() {
            return Vec::new();
        }

        let min_val = record.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = record.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max_val - min_val).abs() < f64::EPSILON {
            return vec![0.0; record.values.len()];
        }

        record.values.iter()
            .map(|&v| (v - min_val) / (max_val - min_val))
            .collect()
    }

    pub fn filter_records(&self, records: Vec<DataRecord>, predicate: impl Fn(&DataRecord) -> bool) -> Vec<DataRecord> {
        records.into_iter()
            .filter(predicate)
            .collect()
    }

    pub fn aggregate_values(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut result = HashMap::new();
        
        if records.is_empty() {
            return result;
        }

        let value_count = records[0].values.len();
        
        for i in 0..value_count {
            let sum: f64 = records.iter()
                .map(|r| r.values.get(i).copied().unwrap_or(0.0))
                .sum();
            
            let avg = sum / records.len() as f64;
            result.insert(format!("avg_value_{}", i), avg);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![10.0, 20.0, 30.0],
            metadata,
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(100.0, 0.0, 50);
        let record = create_test_record();
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_id() {
        let processor = DataProcessor::new(100.0, 0.0, 50);
        let mut record = create_test_record();
        record.id = 0;
        
        assert!(matches!(processor.validate_record(&record), Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_normalize_values() {
        let processor = DataProcessor::new(100.0, 0.0, 50);
        let record = create_test_record();
        let normalized = processor.normalize_values(&record);
        
        assert_eq!(normalized.len(), 3);
        assert!((normalized[0] - 0.0).abs() < 0.001);
        assert!((normalized[1] - 0.5).abs() < 0.001);
        assert!((normalized[2] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_filter_records() {
        let processor = DataProcessor::new(100.0, 0.0, 50);
        let records = vec![
            create_test_record(),
            DataRecord { id: 2, timestamp: 1625097601, values: vec![5.0], metadata: HashMap::new() },
        ];
        
        let filtered = processor.filter_records(records, |r| r.values.len() > 1);
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_aggregate_values() {
        let processor = DataProcessor::new(100.0, 0.0, 50);
        let records = vec![
            DataRecord { id: 1, timestamp: 1625097600, values: vec![10.0, 20.0], metadata: HashMap::new() },
            DataRecord { id: 2, timestamp: 1625097601, values: vec![30.0, 40.0], metadata: HashMap::new() },
        ];
        
        let aggregates = processor.aggregate_values(&records);
        
        assert_eq!(aggregates.len(), 2);
        assert!((aggregates["avg_value_0"] - 20.0).abs() < 0.001);
        assert!((aggregates["avg_value_1"] - 30.0).abs() < 0.001);
    }
}