use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn save_filtered_to_csv(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let file = File::create(output_path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);
        
        for record in filtered {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn add_record(&mut self, id: u32, name: String, value: f64, category: String) {
        self.records.push(Record {
            id,
            name,
            value,
            category,
        });
    }

    fn remove_record_by_id(&mut self, id: u32) -> bool {
        let initial_len = self.records.len();
        self.records.retain(|record| record.id != id);
        self.records.len() != initial_len
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.add_record(1, "Item A".to_string(), 10.5, "Alpha".to_string());
    processor.add_record(2, "Item B".to_string(), 20.3, "Beta".to_string());
    processor.add_record(3, "Item C".to_string(), 15.7, "Alpha".to_string());
    
    println!("Average value: {:.2}", processor.calculate_average());
    
    let alpha_items = processor.filter_by_category("Alpha");
    println!("Alpha category items: {}", alpha_items.len());
    
    for item in alpha_items {
        println!("ID: {}, Name: {}, Value: {}", item.id, item.name, item.value);
    }
    
    processor.remove_record_by_id(2);
    println!("Records after removal: {}", processor.records.len());
    
    Ok(())
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    EmptyCategory,
    TransformationError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyCategory => write!(f, "Category cannot be empty"),
            DataError::TransformationError(msg) => write!(f, "Transformation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if value < 0.0 || value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        if category.trim().is_empty() {
            return Err(DataError::EmptyCategory);
        }

        Ok(Self {
            id,
            value,
            category: category.trim().to_string(),
        })
    }

    pub fn transform_value(&mut self, multiplier: f64) -> Result<(), DataError> {
        if multiplier <= 0.0 {
            return Err(DataError::TransformationError(
                "Multiplier must be positive".to_string(),
            ));
        }

        self.value *= multiplier;
        if self.value > 1000.0 {
            self.value = 1000.0;
        }

        Ok(())
    }

    pub fn normalize(&self) -> f64 {
        self.value / 1000.0
    }

    pub fn get_summary(&self) -> String {
        format!(
            "Record {}: {} ({:.2})",
            self.id,
            self.category,
            self.normalize()
        )
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<String>, DataError> {
    let mut results = Vec::new();

    for record in records.iter_mut() {
        record.transform_value(1.5)?;
        results.push(record.get_summary());
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 100.0, "Test".to_string());
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 100.0, "Test".to_string());
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, 200.0, "Test".to_string()).unwrap();
        assert!(record.transform_value(2.0).is_ok());
        assert_eq!(record.value, 400.0);
    }

    #[test]
    fn test_normalization() {
        let record = DataRecord::new(1, 500.0, "Test".to_string()).unwrap();
        assert_eq!(record.normalize(), 0.5);
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
    
    (sum, mean, std_dev)
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
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
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
            DataError::EmptyTags => write!(f, "Record must have at least one tag"),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if name.trim().is_empty() {
            return Err(DataError::InvalidName);
        }
        if !(0.0..=1000.0).contains(&value) {
            return Err(DataError::InvalidValue);
        }
        if tags.is_empty() {
            return Err(DataError::EmptyTags);
        }

        Ok(Self {
            id,
            name,
            value,
            tags,
        })
    }

    pub fn transform(&self, multiplier: f64) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            value: self.value * multiplier,
            tags: self.tags.clone(),
        }
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if self.records.contains_key(&record.id) {
            return Err(DataError::InvalidId);
        }
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn process_records<F>(&self, mut processor: F) -> Vec<DataRecord>
    where
        F: FnMut(&DataRecord) -> DataRecord,
    {
        self.records.values().map(&mut processor).collect()
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.has_tag(tag))
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.records.values().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(
            1,
            "Test Record".to_string(),
            100.0,
            vec!["tag1".to_string(), "tag2".to_string()],
        );
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(
            0,
            "Test".to_string(),
            100.0,
            vec!["tag".to_string()],
        );
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        let record = DataRecord::new(
            1,
            "Sample".to_string(),
            50.0,
            vec!["important".to_string()],
        ).unwrap();

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
        assert!(processor.get_record(1).is_some());
    }
}