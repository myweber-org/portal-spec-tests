
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: i64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
    TransformationError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid value field"),
            DataError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            DataError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if !value.is_finite() {
            return Err(DataError::InvalidValue);
        }
        if timestamp < 0 {
            return Err(DataError::InvalidTimestamp);
        }

        Ok(DataRecord {
            id,
            value,
            timestamp,
        })
    }

    pub fn transform(&self, factor: f64) -> Result<f64, DataError> {
        if !factor.is_finite() || factor == 0.0 {
            return Err(DataError::TransformationError(
                "Invalid transformation factor".to_string(),
            ));
        }

        let transformed = self.value * factor.ln().exp();
        if transformed.is_nan() || transformed.is_infinite() {
            Err(DataError::TransformationError(
                "Result is not a valid number".to_string(),
            ))
        } else {
            Ok(transformed)
        }
    }

    pub fn normalize(&self, max_value: f64) -> Result<f64, DataError> {
        if max_value <= 0.0 || !max_value.is_finite() {
            return Err(DataError::TransformationError(
                "Invalid normalization parameter".to_string(),
            ));
        }

        let normalized = self.value / max_value;
        if normalized.is_nan() || normalized.is_infinite() {
            Err(DataError::TransformationError(
                "Normalization produced invalid result".to_string(),
            ))
        } else {
            Ok(normalized)
        }
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<f64, DataError>> {
    records.iter().map(|record| record.transform(2.0)).collect()
}

pub fn validate_record_batch(records: &[DataRecord]) -> bool {
    records.iter().all(|record| {
        record.id != 0 && record.value.is_finite() && record.timestamp >= 0
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, 1672531200);
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 42.5, 1672531200);
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_transformation() {
        let record = DataRecord::new(1, 10.0, 1672531200).unwrap();
        let result = record.transform(2.0);
        assert!(result.is_ok());
        assert!((result.unwrap() - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_normalization() {
        let record = DataRecord::new(1, 50.0, 1672531200).unwrap();
        let result = record.normalize(100.0);
        assert!(result.is_ok());
        assert!((result.unwrap() - 0.5).abs() < 0.001);
    }
}