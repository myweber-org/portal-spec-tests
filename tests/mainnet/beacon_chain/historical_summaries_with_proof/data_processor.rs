
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    InvalidMetadata,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.timestamp < 0 {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        if self.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }
        
        for (key, value) in &self.metadata {
            if key.trim().is_empty() || value.trim().is_empty() {
                return Err(ValidationError::InvalidMetadata);
            }
        }
        
        Ok(())
    }
    
    pub fn transform_values(&mut self, transformer: fn(f64) -> f64) {
        self.values = self.values.iter().map(|&v| transformer(v)).collect();
    }
    
    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = self.values
            .iter()
            .map(|&value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

pub fn normalize_data(records: &mut [DataRecord]) {
    for record in records {
        if let Ok(()) = record.validate() {
            record.transform_values(|x| (x - x.min()) / (x.max() - x.min()));
        }
    }
}

pub fn filter_valid_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|record| record.validate().is_ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation_success() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "sensor_a".to_string());
        
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0],
            metadata,
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_validation_failure() {
        let record = DataRecord {
            id: 0,
            timestamp: -1,
            values: vec![],
            metadata: HashMap::new(),
        };
        
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata: HashMap::new(),
        };
        
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
    
    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        record.transform_values(|x| x * 2.0);
        
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }
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

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut rdr = csv::Reader::from_reader(reader);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.add_record(record);
        }

        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let mut wtr = csv::Writer::from_writer(writer);

        for record in &self.records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.is_valid() {
                *counts.entry(record.category.clone()).or_insert(0) += 1;
            }
        }
        
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A");
        assert!(valid_record.is_valid());

        let invalid_value = DataRecord::new(2, -5.0, "B");
        assert!(!invalid_value.is_valid());

        let invalid_category = DataRecord::new(3, 15.0, "");
        assert!(!invalid_category.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord::new(1, 10.0, "CategoryA"));
        processor.add_record(DataRecord::new(2, 20.0, "CategoryB"));
        processor.add_record(DataRecord::new(3, -5.0, "CategoryA"));

        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 2);

        let average = processor.calculate_average();
        assert_eq!(average, Some(15.0));

        let counts = processor.count_by_category();
        assert_eq!(counts.get("CategoryA"), Some(&1));
        assert_eq!(counts.get("CategoryB"), Some(&1));
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, 10.5, "Test"));
        processor.add_record(DataRecord::new(2, 20.3, "Test"));

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        processor.save_to_csv(path)?;

        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(path)?;

        assert_eq!(new_processor.records.len(), 2);
        Ok(())
    }
}