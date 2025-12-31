use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    SerializationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Value must be positive"),
            ProcessingError::InvalidTimestamp => write!(f, "Timestamp cannot be negative"),
            ProcessingError::SerializationError(msg) => write!(f, "Serialization failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.value <= 0.0 {
        return Err(ProcessingError::InvalidValue);
    }
    if record.timestamp < 0 {
        return Err(ProcessingError::InvalidTimestamp);
    }
    Ok(())
}

pub fn transform_record(record: &DataRecord, multiplier: f64) -> DataRecord {
    DataRecord {
        id: record.id,
        value: record.value * multiplier,
        timestamp: record.timestamp,
    }
}

pub fn serialize_to_json(record: &DataRecord) -> Result<String, ProcessingError> {
    serde_json::to_string(record)
        .map_err(|e| ProcessingError::SerializationError(e.to_string()))
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<String>, ProcessingError> {
    let mut results = Vec::new();
    
    for record in records {
        validate_record(&record)?;
        let transformed = transform_record(&record, 2.0);
        let json = serialize_to_json(&transformed)?;
        results.push(json);
    }
    
    Ok(results)
}