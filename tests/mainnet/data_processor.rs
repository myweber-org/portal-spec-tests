
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidName => write!(f, "Name cannot be empty"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyTags => write!(f, "Record must have at least one tag"),
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

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
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
            .map(|&value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn merge_tags(&mut self) {
        for record in self.records.values_mut() {
            record.tags.sort();
            record.tags.dedup();
        }
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            tags: vec!["tag1".to_string()],
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 50.0,
            tags: vec!["tag1".to_string()],
        };
        
        assert!(matches!(processor.validate_record(&record), Err(DataError::InvalidId)));
    }

    #[test]
    fn test_add_and_retrieve_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 42,
            name: "Sample".to_string(),
            value: 100.0,
            tags: vec!["important".to_string(), "test".to_string()],
        };
        
        assert!(processor.add_record(record.clone()).is_ok());
        assert_eq!(processor.record_count(), 1);
        
        let retrieved = processor.get_record(42);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Sample");
    }

    #[test]
    fn test_transform_values() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            tags: vec!["test".to_string()],
        };
        
        processor.add_record(record).unwrap();
        processor.transform_values(|x| x * 2.0);
        
        let updated = processor.get_record(1).unwrap();
        assert_eq!(updated.value, 20.0);
    }

    #[test]
    fn test_filter_by_tag() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "First".to_string(),
            value: 10.0,
            tags: vec!["alpha".to_string(), "beta".to_string()],
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Second".to_string(),
            value: 20.0,
            tags: vec!["beta".to_string(), "gamma".to_string()],
        };
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        let beta_records = processor.filter_by_tag("beta");
        assert_eq!(beta_records.len(), 2);
        
        let alpha_records = processor.filter_by_tag("alpha");
        assert_eq!(alpha_records.len(), 1);
    }
}