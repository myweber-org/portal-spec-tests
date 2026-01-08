
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ProcessingError {
    message: String,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Processing error: {}", self.message)
    }
}

impl Error for ProcessingError {}

impl ProcessingError {
    pub fn new(msg: &str) -> Self {
        ProcessingError {
            message: msg.to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: u64,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::new("ID cannot be zero"));
        }
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(ProcessingError::new("Value must be a finite number"));
        }
        if self.timestamp == 0 {
            return Err(ProcessingError::new("Timestamp cannot be zero"));
        }
        Ok(())
    }

    pub fn normalize(&mut self, factor: f64) -> Result<(), ProcessingError> {
        if factor == 0.0 {
            return Err(ProcessingError::new("Normalization factor cannot be zero"));
        }
        self.value /= factor;
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord], factor: f64) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        record.normalize(factor)?;
        processed.push(record.clone());
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord { id: 1, value: 42.5, timestamp: 1234567890 };
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord { id: 0, value: 42.5, timestamp: 1234567890 };
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord { id: 1, value: 100.0, timestamp: 1234567890 };
        record.normalize(10.0).unwrap();
        assert_eq!(record.value, 10.0);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord { id: 1, value: 10.0, timestamp: 1000 },
            DataRecord { id: 2, value: 20.0, timestamp: 2000 },
            DataRecord { id: 3, value: 30.0, timestamp: 3000 },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}