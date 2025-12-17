
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: u64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    TimestampOutOfRange,
    TransformationError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid data value"),
            DataError::TimestampOutOfRange => write!(f, "Timestamp out of valid range"),
            DataError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    validation_enabled: bool,
    max_value_threshold: f64,
}

impl DataProcessor {
    pub fn new(validation_enabled: bool, max_value_threshold: f64) -> Self {
        DataProcessor {
            validation_enabled,
            max_value_threshold,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if !self.validation_enabled {
            return Ok(());
        }

        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.value.is_nan() || record.value.is_infinite() {
            return Err(DataError::InvalidValue);
        }

        if record.value.abs() > self.max_value_threshold {
            return Err(DataError::InvalidValue);
        }

        if record.timestamp > 1_000_000_000_000 {
            return Err(DataError::TimestampOutOfRange);
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, DataError> {
        self.validate_record(record)?;

        let transformed_value = if record.value >= 0.0 {
            record.value.ln()
        } else {
            -record.value.abs().ln()
        };

        if transformed_value.is_nan() || transformed_value.is_infinite() {
            return Err(DataError::TransformationError(
                "Failed to compute logarithm".to_string(),
            ));
        }

        Ok(DataRecord {
            id: record.id,
            value: transformed_value,
            timestamp: record.timestamp,
        })
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> (Vec<DataRecord>, Vec<DataError>) {
        let mut processed = Vec::new();
        let mut errors = Vec::new();

        for record in records {
            match self.transform_record(&record) {
                Ok(transformed) => processed.push(transformed),
                Err(err) => errors.push(err),
            }
        }

        (processed, errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_validation() {
        let processor = DataProcessor::new(true, 1000.0);
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1_000_000,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id_validation() {
        let processor = DataProcessor::new(true, 1000.0);
        let record = DataRecord {
            id: 0,
            value: 42.5,
            timestamp: 1_000_000,
        };

        assert!(matches!(
            processor.validate_record(&record),
            Err(DataError::InvalidId)
        ));
    }

    #[test]
    fn test_record_transformation() {
        let processor = DataProcessor::new(true, 1000.0);
        let record = DataRecord {
            id: 1,
            value: std::f64::consts::E,
            timestamp: 1_000_000,
        };

        let transformed = processor.transform_record(&record).unwrap();
        assert!((transformed.value - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(true, 1000.0);
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 1_000_000,
            },
            DataRecord {
                id: 0,
                value: 20.0,
                timestamp: 1_000_000,
            },
            DataRecord {
                id: 3,
                value: -1.0,
                timestamp: 1_000_000,
            },
        ];

        let (processed, errors) = processor.process_batch(records);
        assert_eq!(processed.len(), 2);
        assert_eq!(errors.len(), 1);
    }
}