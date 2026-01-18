
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Self {
        DataRecord { id, name, value, tags }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.tags.len() > 10 {
            return Err("Too many tags, maximum is 10".to_string());
        }
        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> HashMap<u32, f64> {
    let mut results = HashMap::new();
    
    for record in records.iter_mut() {
        if let Ok(_) = record.validate() {
            record.transform(multiplier);
            results.insert(record.id, record.value);
        }
    }
    
    results
}

pub fn filter_by_threshold(records: &[DataRecord], threshold: f64) -> Vec<&DataRecord> {
    records.iter()
        .filter(|r| r.value >= threshold)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "test".to_string(), 42.0, vec!["tag1".to_string()]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, vec![]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, "test".to_string(), 10.0, vec![]);
        record.transform(2.5);
        assert_eq!(record.value, 25.0);
        assert_eq!(record.name, "TEST");
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord::new(1, "a".to_string(), 10.0, vec![]),
            DataRecord::new(2, "b".to_string(), 20.0, vec![]),
        ];
        
        let results = process_records(&mut records, 3.0);
        assert_eq!(results.get(&1), Some(&30.0));
        assert_eq!(results.get(&2), Some(&60.0));
    }
}