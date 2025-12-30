
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum DataError {
    InvalidFormat,
    OutOfRange,
    ConversionFailed,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidFormat => write!(f, "Data format is invalid"),
            DataError::OutOfRange => write!(f, "Value is out of acceptable range"),
            DataError::ConversionFailed => write!(f, "Failed to convert data type"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, DataError> {
        if threshold < 0.0 || threshold > 100.0 {
            return Err(DataError::OutOfRange);
        }
        Ok(Self { threshold })
    }

    pub fn process_value(&self, input: &str) -> Result<f64, DataError> {
        let parsed = input.parse::<f64>().map_err(|_| DataError::InvalidFormat)?;
        
        if parsed < 0.0 {
            return Err(DataError::OutOfRange);
        }

        let processed = (parsed * 1.5).min(self.threshold);
        Ok(processed)
    }

    pub fn batch_process(&self, inputs: &[&str]) -> Vec<Result<f64, DataError>> {
        inputs.iter().map(|&input| self.process_value(input)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processing() {
        let processor = DataProcessor::new(50.0).unwrap();
        let result = processor.process_value("10.5").unwrap();
        assert_eq!(result, 15.75);
    }

    #[test]
    fn test_threshold_limit() {
        let processor = DataProcessor::new(20.0).unwrap();
        let result = processor.process_value("50.0").unwrap();
        assert_eq!(result, 20.0);
    }

    #[test]
    fn test_invalid_input() {
        let processor = DataProcessor::new(50.0).unwrap();
        let result = processor.process_value("invalid");
        assert!(matches!(result, Err(DataError::InvalidFormat)));
    }
}