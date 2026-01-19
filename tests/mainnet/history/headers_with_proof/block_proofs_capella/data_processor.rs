
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Self {
        DataRecord { id, value, timestamp }
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::InvalidData("ID cannot be zero".to_string()));
        }
        if !self.value.is_finite() {
            return Err(ProcessingError::InvalidData("Value must be finite".to_string()));
        }
        if self.timestamp < 0 {
            return Err(ProcessingError::InvalidData("Timestamp cannot be negative".to_string()));
        }
        Ok(())
    }

    pub fn transform(&mut self, factor: f64) -> Result<(), ProcessingError> {
        if factor <= 0.0 {
            return Err(ProcessingError::TransformationFailed(
                "Transformation factor must be positive".to_string(),
            ));
        }
        self.value *= factor;
        self.timestamp += 3600;
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord], factor: f64) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        record.transform(factor)?;
        processed.push(DataRecord::new(record.id, record.value, record.timestamp));
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 42.5, 1672531200);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 42.5, 1672531200);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_transformation() {
        let mut record = DataRecord::new(1, 10.0, 1672531200);
        assert!(record.transform(2.5).is_ok());
        assert_eq!(record.value, 25.0);
        assert_eq!(record.timestamp, 1672534800);
    }

    #[test]
    fn test_batch_processing() {
        let mut records = vec![
            DataRecord::new(1, 10.0, 1672531200),
            DataRecord::new(2, 20.0, 1672531200),
        ];
        
        let result = process_records(&mut records, 3.0);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed[0].value, 30.0);
        assert_eq!(processed[1].value, 60.0);
    }
}