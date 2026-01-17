use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "Invalid record ID"),
            ValidationError::InvalidValue => write!(f, "Invalid value field"),
            ValidationError::InvalidTimestamp => write!(f, "Invalid timestamp"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.value.is_nan() || record.value.is_infinite() {
        return Err(ValidationError::InvalidValue);
    }
    
    if record.timestamp < 0 {
        return Err(ValidationError::InvalidTimestamp);
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord, multiplier: f64) -> Option<DataRecord> {
    if multiplier.is_nan() || multiplier.is_infinite() || multiplier == 0.0 {
        return None;
    }
    
    let transformed_value = record.value * multiplier;
    
    Some(DataRecord {
        id: record.id,
        value: transformed_value,
        timestamp: record.timestamp,
    })
}

pub fn process_records(records: Vec<DataRecord>, multiplier: f64) -> Vec<Result<DataRecord, ValidationError>> {
    records
        .into_iter()
        .map(|record| {
            validate_record(&record)
                .map(|_| transform_record(&record, multiplier))
                .and_then(|opt| opt.ok_or(ValidationError::InvalidValue))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1625097600,
        };
        
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 42.5,
            timestamp: 1625097600,
        };
        
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord {
            id: 1,
            value: 10.0,
            timestamp: 1625097600,
        };
        
        let transformed = transform_record(&record, 2.5).unwrap();
        assert_eq!(transformed.value, 25.0);
    }
}