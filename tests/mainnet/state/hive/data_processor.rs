
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
    config: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new(config: HashMap<String, String>) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.id == 0 {
            return Err(ProcessingError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError("Name cannot be empty".to_string()));
        }
        
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(ProcessingError::ValidationError("Value must be a finite number".to_string()));
        }
        
        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        if let Some(prefix) = self.config.get("name_prefix") {
            transformed.name = format!("{}{}", prefix, transformed.name);
        }
        
        if let Some(factor_str) = self.config.get("value_multiplier") {
            if let Ok(factor) = factor_str.parse::<f64>() {
                transformed.value *= factor;
            } else {
                return Err(ProcessingError::TransformationError(
                    "Invalid multiplier in config".to_string()
                ));
            }
        }
        
        if let Some(tag_filter) = self.config.get("tag_filter") {
            transformed.tags.retain(|tag| tag.contains(tag_filter));
        }
        
        Ok(transformed)
    }

    pub fn process_records(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record)?;
            processed.push(transformed);
        }
        
        Ok(processed)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }
        
        let count = records.len() as f64;
        let sum: f64 = records.iter().map(|r| r.value).sum();
        let avg = sum / count;
        
        let variance: f64 = records.iter()
            .map(|r| (r.value - avg).powi(2))
            .sum::<f64>() / count;
        
        let max_value = records.iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);
        
        let min_value = records.iter()
            .map(|r| r.value)
            .fold(f64::INFINITY, f64::min);
        
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("average".to_string(), avg);
        stats.insert("variance".to_string(), variance);
        stats.insert("max".to_string(), max_value);
        stats.insert("min".to_string(), min_value);
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let config = HashMap::new();
        let processor = DataProcessor::new(config);
        
        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 42.0,
            tags: vec!["tag1".to_string()],
        };
        
        assert!(processor.validate_record(&valid_record).is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: f64::NAN,
            tags: vec![],
        };
        
        assert!(processor.validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_transformation() {
        let mut config = HashMap::new();
        config.insert("name_prefix".to_string(), "PREFIX_".to_string());
        config.insert("value_multiplier".to_string(), "2.0".to_string());
        
        let processor = DataProcessor::new(config);
        
        let record = DataRecord {
            id: 1,
            name: "Original".to_string(),
            value: 10.0,
            tags: vec!["important".to_string(), "test".to_string()],
        };
        
        let transformed = processor.transform_record(&record).unwrap();
        
        assert_eq!(transformed.name, "PREFIX_Original");
        assert_eq!(transformed.value, 20.0);
    }

    #[test]
    fn test_statistics() {
        let config = HashMap::new();
        let processor = DataProcessor::new(config);
        
        let records = vec![
            DataRecord { id: 1, name: "A".to_string(), value: 10.0, tags: vec![] },
            DataRecord { id: 2, name: "B".to_string(), value: 20.0, tags: vec![] },
            DataRecord { id: 3, name: "C".to_string(), value: 30.0, tags: vec![] },
        ];
        
        let stats = processor.calculate_statistics(&records);
        
        assert_eq!(stats.get("count"), Some(&3.0));
        assert_eq!(stats.get("sum"), Some(&60.0));
        assert_eq!(stats.get("average"), Some(&20.0));
        assert_eq!(stats.get("min"), Some(&10.0));
        assert_eq!(stats.get("max"), Some(&30.0));
    }
}
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

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    fn find_by_id(&self, id: u32) -> Option<&Record> {
        self.records.iter().find(|record| record.id == id)
    }

    fn export_to_json(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &self.records)?;
        Ok(())
    }
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value >= 0.0
}

fn process_data() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("input.csv")?;
    
    let active_records = processor.filter_active();
    println!("Active records: {}", active_records.len());
    
    if let Some(avg) = processor.calculate_average() {
        println!("Average value: {:.2}", avg);
    }
    
    if let Some(record) = processor.find_by_id(42) {
        println!("Found record: {:?}", record);
    }
    
    for record in &processor.records {
        if validate_record(record) {
            println!("Valid record: {}", record.id);
        }
    }
    
    processor.export_to_json("output.json")?;
    
    Ok(())
}