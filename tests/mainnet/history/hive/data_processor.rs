
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationFailed("ID cannot be zero".to_string()));
        }
        
        if self.timestamp <= 0 {
            return Err(DataError::ValidationFailed("Timestamp must be positive".to_string()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::ValidationFailed("Values cannot be empty".to_string()));
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) {
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        record.transform(multiplier);
        record.add_tag("processed".to_string());
        processed.push(record.clone());
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, (f64, f64, f64)> {
    let mut stats = HashMap::new();
    
    for record in records {
        for (key, value) in &record.values {
            let entry = stats.entry(key.clone()).or_insert((f64::MAX, f64::MIN, 0.0, 0));
            let (min, max, sum, count) = *entry;
            
            let new_min = min.min(*value);
            let new_max = max.max(*value);
            let new_sum = sum + *value;
            let new_count = count + 1;
            
            stats.insert(key.clone(), (new_min, new_max, new_sum, new_count));
        }
    }
    
    stats
        .into_iter()
        .map(|(key, (min, max, sum, count))| {
            let avg = if count > 0 { sum / count as f64 } else { 0.0 };
            (key, (min, max, avg))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("temperature".to_string(), 25.5)]),
            tags: vec![],
        };
        
        assert!(record.validate().is_ok());
        
        record.id = 0;
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("value".to_string(), 10.0)]),
            tags: vec![],
        };
        
        record.transform(2.0);
        assert_eq!(record.values.get("value"), Some(&20.0));
    }
    
    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: HashMap::from([("metric".to_string(), 10.0)]),
                tags: vec![],
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: HashMap::from([("metric".to_string(), 20.0)]),
                tags: vec![],
            },
        ];
        
        let stats = calculate_statistics(&records);
        let (min, max, avg) = stats.get("metric").unwrap();
        
        assert_eq!(*min, 10.0);
        assert_eq!(*max, 20.0);
        assert_eq!(*avg, 15.0);
    }
}