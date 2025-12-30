
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

pub struct DataProcessor {
    threshold: f64,
    multiplier: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64, multiplier: f64) -> Result<Self, ProcessingError> {
        if threshold <= 0.0 {
            return Err(ProcessingError::new("Threshold must be positive"));
        }
        if multiplier <= 0.0 {
            return Err(ProcessingError::new("Multiplier must be positive"));
        }
        
        Ok(DataProcessor {
            threshold,
            multiplier,
        })
    }
    
    pub fn process_value(&self, value: f64) -> Result<f64, ProcessingError> {
        if value < 0.0 {
            return Err(ProcessingError::new("Value cannot be negative"));
        }
        
        if value > self.threshold {
            let adjusted = value * self.multiplier;
            if adjusted.is_infinite() {
                return Err(ProcessingError::new("Result exceeds numerical limits"));
            }
            Ok(adjusted)
        } else {
            Ok(value)
        }
    }
    
    pub fn batch_process(&self, values: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        let mut results = Vec::with_capacity(values.len());
        
        for &value in values {
            match self.process_value(value) {
                Ok(result) => results.push(result),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }
}

pub fn validate_input_range(values: &[f64], min: f64, max: f64) -> bool {
    values.iter().all(|&v| v >= min && v <= max)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_processor_creation() {
        let processor = DataProcessor::new(10.0, 2.0);
        assert!(processor.is_ok());
        
        let invalid = DataProcessor::new(0.0, 2.0);
        assert!(invalid.is_err());
    }
    
    #[test]
    fn test_value_processing() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        
        assert_eq!(processor.process_value(5.0).unwrap(), 5.0);
        assert_eq!(processor.process_value(15.0).unwrap(), 30.0);
        
        let negative_result = processor.process_value(-5.0);
        assert!(negative_result.is_err());
    }
    
    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        let values = vec![5.0, 15.0, 8.0, 20.0];
        
        let results = processor.batch_process(&values).unwrap();
        assert_eq!(results, vec![5.0, 30.0, 8.0, 40.0]);
    }
    
    #[test]
    fn test_validation() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        assert!(validate_input_range(&values, 0.0, 5.0));
        assert!(!validate_input_range(&values, 2.0, 5.0));
    }
}