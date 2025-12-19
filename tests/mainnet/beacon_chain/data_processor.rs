
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64, values: Vec<f64>) -> Self {
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
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|record| record.validate().is_ok())
        .map(|mut record| {
            let normalized_values: Vec<f64> = record
                .values
                .iter()
                .map(|&value| {
                    if value.is_nan() {
                        0.0
                    } else {
                        value.clamp(0.0, 100.0)
                    }
                })
                .collect();
            record.values = normalized_values;
            record
        })
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }

    let total_values: usize = records.iter().map(|r| r.values.len()).sum();
    let all_values: Vec<f64> = records.iter().flat_map(|r| r.values.clone()).collect();

    let sum: f64 = all_values.iter().sum();
    let count = all_values.len() as f64;
    
    if count > 0.0 {
        let mean = sum / count;
        stats.insert("mean".to_string(), mean);
        
        let variance: f64 = all_values.iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());
    }

    stats.insert("total_records".to_string(), records.len() as f64);
    stats.insert("total_values".to_string(), total_values as f64);
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1625097600, vec![10.0, 20.0, 30.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1625097600, vec![10.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 1625097600, vec![150.0, -10.0, f64::NAN]),
            DataRecord::new(2, 1625097600, vec![50.0, 75.0, 25.0]),
        ];
        
        let processed = process_records(records);
        assert_eq!(processed.len(), 2);
        
        for record in processed {
            for &value in &record.values {
                assert!(value >= 0.0 && value <= 100.0);
                assert!(!value.is_nan());
            }
        }
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 1625097600, vec![10.0, 20.0]),
            DataRecord::new(2, 1625097600, vec![30.0, 40.0]),
        ];
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats["total_records"], 2.0);
        assert_eq!(stats["total_values"], 4.0);
        assert_eq!(stats["mean"], 25.0);
    }
}