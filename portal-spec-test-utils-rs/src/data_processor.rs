
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ProcessingError {
    details: String,
}

impl ProcessingError {
    fn new(msg: &str) -> ProcessingError {
        ProcessingError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ProcessingError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<DataRecord, ProcessingError> {
        if value < 0.0 || value > 1000.0 {
            return Err(ProcessingError::new("Value out of valid range (0-1000)"));
        }
        if timestamp < 0 {
            return Err(ProcessingError::new("Timestamp cannot be negative"));
        }
        Ok(DataRecord {
            id,
            value,
            timestamp,
        })
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<f64> {
    records
        .iter()
        .map(|r| r.value * 1.1)
        .filter(|&v| v <= 900.0)
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records
        .iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>()
        / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, 1234567890);
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 500.0);
        assert_eq!(record.timestamp, 1234567890);
    }

    #[test]
    fn test_invalid_value_record() {
        let record = DataRecord::new(1, 1500.0, 1234567890);
        assert!(record.is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 100.0, 1000).unwrap(),
            DataRecord::new(2, 200.0, 2000).unwrap(),
            DataRecord::new(3, 900.0, 3000).unwrap(),
        ];
        let processed = process_records(&records);
        assert_eq!(processed.len(), 3);
        assert!((processed[0] - 110.0).abs() < 0.001);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord::new(1, 10.0, 1000).unwrap(),
            DataRecord::new(2, 20.0, 2000).unwrap(),
            DataRecord::new(3, 30.0, 3000).unwrap(),
        ];
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert!((mean - 20.0).abs() < 0.001);
        assert!((variance - 66.666).abs() < 0.1);
        assert!((std_dev - 8.1649).abs() < 0.001);
    }
}