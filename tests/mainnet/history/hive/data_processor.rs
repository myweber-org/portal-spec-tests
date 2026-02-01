
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }
        Ok(Self { threshold })
    }

    pub fn process_data(&self, data: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if data.is_empty() {
            return Err(ValidationError {
                message: "Input data cannot be empty".to_string(),
            });
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let filtered: Vec<f64> = data
            .iter()
            .filter(|&&value| value >= mean * self.threshold)
            .cloned()
            .collect();

        if filtered.is_empty() {
            return Err(ValidationError {
                message: "All data filtered out".to_string(),
            });
        }

        Ok(filtered)
    }

    pub fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        if data.is_empty() {
            return Vec::new();
        }

        let max_value = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min_value = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let range = max_value - min_value;

        if range.abs() < f64::EPSILON {
            return vec![0.5; data.len()];
        }

        data.iter()
            .map(|&value| (value - min_value) / range)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(1.5);
        assert!(processor.is_err());
    }

    #[test]
    fn test_process_data() {
        let processor = DataProcessor::new(0.5).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = processor.process_data(&data);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_normalize_data() {
        let processor = DataProcessor::new(0.5).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = processor.normalize_data(&data);
        assert_eq!(normalized.len(), data.len());
        assert!(normalized[0] >= 0.0 && normalized[0] <= 1.0);
    }
}