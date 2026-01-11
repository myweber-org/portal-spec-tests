use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    id: u64,
    timestamp: i64,
    values: HashMap<String, f64>,
    tags: Vec<String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn add_value(&mut self, key: String, value: f64) -> Result<(), String> {
        if value.is_nan() || value.is_infinite() {
            return Err(format!("Invalid value for key '{}': {}", key, value));
        }
        self.values.insert(key, value);
        Ok(())
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.id == 0 {
            errors.push("ID cannot be zero".to_string());
        }

        if self.timestamp < 0 {
            errors.push("Timestamp cannot be negative".to_string());
        }

        if self.values.is_empty() {
            errors.push("Record must contain at least one value".to_string());
        }

        for (key, value) in &self.values {
            if value.is_nan() || value.is_infinite() {
                errors.push(format!("Invalid value for key '{}': {}", key, value));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn transform(&self, multiplier: f64) -> Self {
        let mut transformed = self.clone();
        
        for value in transformed.values.values_mut() {
            *value *= multiplier;
        }

        transformed.tags.push("transformed".to_string());
        transformed
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|record| record.validate().is_ok())
        .map(|record| record.transform(2.0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_record_validation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("temperature".to_string(), 25.5).unwrap();
        
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, -1);
        let result = record.validate();
        
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors.len() >= 2);
        }
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("pressure".to_string(), 100.0).unwrap();
        
        let transformed = record.transform(1.5);
        assert_eq!(transformed.values.get("pressure"), Some(&150.0));
        assert!(transformed.tags.contains(&"transformed".to_string()));
    }
}