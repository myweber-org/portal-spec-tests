
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid record ID".to_string());
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        if self.values.is_empty() {
            return Err("Record must contain at least one value".to_string());
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> Statistics {
        let count = self.values.len();
        let sum: f64 = self.values.iter().sum();
        let mean = if count > 0 { sum / count as f64 } else { 0.0 };
        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Statistics {
            count,
            sum,
            mean,
            min,
            max,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<Result<DataRecord, String>> {
    records
        .into_iter()
        .map(|mut record| {
            match record.validate() {
                Ok(_) => {
                    if record.values.len() > 10 {
                        record.values.truncate(10);
                    }
                    Ok(record)
                }
                Err(e) => Err(e),
            }
        })
        .collect()
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
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(42.5);
        assert!(record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(10.0);
        record.add_value(20.0);
        record.add_value(30.0);

        let stats = record.calculate_statistics();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.sum, 60.0);
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
    }

    #[test]
    fn test_process_records() {
        let mut valid_record = DataRecord::new(1, 1234567890);
        valid_record.add_value(42.5);

        let invalid_record = DataRecord::new(0, 1234567890);

        let records = vec![valid_record, invalid_record];
        let results = process_records(records);

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
    }
}