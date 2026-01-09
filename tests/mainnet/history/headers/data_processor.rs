
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue(f64),
    MissingTimestamp,
    RecordTooOld(i64),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            ProcessingError::MissingTimestamp => write!(f, "Missing timestamp"),
            ProcessingError::RecordTooOld(ts) => write!(f, "Record too old: {}", ts),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    max_age: i64,
    min_value: f64,
    max_value: f64,
}

impl DataProcessor {
    pub fn new(max_age: i64, min_value: f64, max_value: f64) -> Self {
        DataProcessor {
            max_age,
            min_value,
            max_value,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < self.min_value || record.value > self.max_value {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        let current_time = chrono::Utc::now().timestamp();
        if current_time - record.timestamp > self.max_age {
            return Err(ProcessingError::RecordTooOld(record.timestamp));
        }

        Ok(())
    }

    pub fn transform_value(&self, record: &DataRecord) -> f64 {
        (record.value - self.min_value) / (self.max_value - self.min_value)
    }

    pub fn process_records(&self, records: Vec<DataRecord>) -> Vec<Result<f64, ProcessingError>> {
        records
            .into_iter()
            .map(|record| {
                self.validate_record(&record)
                    .map(|_| self.transform_value(&record))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(3600, 0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: chrono::Utc::now().timestamp(),
        };
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_value() {
        let processor = DataProcessor::new(3600, 0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: chrono::Utc::now().timestamp(),
        };
        match processor.validate_record(&record) {
            Err(ProcessingError::InvalidValue(v)) => assert_eq!(v, 150.0),
            _ => panic!("Expected InvalidValue error"),
        }
    }

    #[test]
    fn test_transform_value() {
        let processor = DataProcessor::new(3600, 0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 75.0,
            timestamp: chrono::Utc::now().timestamp(),
        };
        assert_eq!(processor.transform_value(&record), 0.75);
    }
}