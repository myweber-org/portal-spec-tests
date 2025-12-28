
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
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ProcessingError> {
        if threshold <= 0.0 {
            return Err(ProcessingError::new("Threshold must be positive"));
        }
        Ok(DataProcessor { threshold })
    }

    pub fn validate_data(&self, data: &[f64]) -> Result<(), ProcessingError> {
        if data.is_empty() {
            return Err(ProcessingError::new("Data slice cannot be empty"));
        }

        for &value in data {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::new("Data contains invalid numeric values"));
            }
        }

        Ok(())
    }

    pub fn filter_values(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .filter(|&&x| x >= self.threshold)
            .cloned()
            .collect()
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> Result<(f64, f64), ProcessingError> {
        self.validate_data(data)?;

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;

        let variance: f64 = data
            .iter()
            .map(|&x| {
                let diff = x - mean;
                diff * diff
            })
            .sum::<f64>()
            / data.len() as f64;

        Ok((mean, variance.sqrt()))
    }

    pub fn normalize_data(&self, data: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        self.validate_data(data)?;

        let (mean, std_dev) = self.calculate_statistics(data)?;

        if std_dev.abs() < 1e-10 {
            return Err(ProcessingError::new("Cannot normalize data with zero standard deviation"));
        }

        Ok(data
            .iter()
            .map(|&x| (x - mean) / std_dev)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = DataProcessor::new(5.0);
        assert!(processor.is_ok());

        let invalid_processor = DataProcessor::new(0.0);
        assert!(invalid_processor.is_err());
    }

    #[test]
    fn test_validation() {
        let processor = DataProcessor::new(1.0).unwrap();
        let valid_data = vec![1.0, 2.0, 3.0];
        assert!(processor.validate_data(&valid_data).is_ok());

        let empty_data: Vec<f64> = vec![];
        assert!(processor.validate_data(&empty_data).is_err());
    }

    #[test]
    fn test_filtering() {
        let processor = DataProcessor::new(2.5).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let filtered = processor.filter_values(&data);
        assert_eq!(filtered, vec![3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_statistics() {
        let processor = DataProcessor::new(0.1).unwrap();
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let (mean, std_dev) = processor.calculate_statistics(&data).unwrap();
        assert!((mean - 5.0).abs() < 1e-10);
        assert!((std_dev - 2.0).abs() < 1e-10);
    }
}