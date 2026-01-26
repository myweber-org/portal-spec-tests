
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u64,
    timestamp: i64,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64, values: Vec<f64>) -> Self {
        Self {
            id,
            timestamp,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        if self.timestamp < 0 {
            return Err("Invalid timestamp".into());
        }
        if self.values.is_empty() {
            return Err("Empty values array".into());
        }
        Ok(())
    }

    pub fn normalize_values(&mut self) {
        if let Some(max) = self.values.iter().copied().reduce(f64::max) {
            if max != 0.0 {
                for value in &mut self.values {
                    *value /= max;
                }
            }
        }
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::new();
    
    for record in records {
        record.validate()?;
        let mut processed_record = DataRecord::new(
            record.id,
            record.timestamp,
            record.values.clone(),
        );
        processed_record.normalize_values();
        processed.push(processed_record);
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }
    
    let all_values: Vec<f64> = records.iter()
        .flat_map(|r| r.values.iter().copied())
        .collect();
    
    let count = all_values.len() as f64;
    let sum: f64 = all_values.iter().sum();
    let mean = sum / count;
    
    let variance: f64 = all_values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / count;
    
    stats.insert("count".to_string(), count);
    stats.insert("mean".to_string(), mean);
    stats.insert("variance".to_string(), variance);
    stats.insert("min".to_string(), all_values.iter().copied().fold(f64::INFINITY, f64::min));
    stats.insert("max".to_string(), all_values.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_normalize_values() {
        let mut record = DataRecord::new(1, 1234567890, vec![2.0, 4.0, 6.0]);
        record.normalize_values();
        assert_eq!(record.values, vec![1.0/3.0, 2.0/3.0, 1.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord::new(1, 1000, vec![1.0, 2.0]),
            DataRecord::new(2, 2000, vec![3.0, 4.0]),
        ];
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats["count"], 4.0);
        assert_eq!(stats["mean"], 2.5);
        assert_eq!(stats["min"], 1.0);
        assert_eq!(stats["max"], 4.0);
    }
}