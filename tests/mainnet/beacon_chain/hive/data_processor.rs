
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
}use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: vec![
                ValidationRule {
                    min_value: Some(0.0),
                    max_value: Some(100.0),
                    required: true,
                },
                ValidationRule {
                    min_value: Some(-50.0),
                    max_value: Some(50.0),
                    required: false,
                },
            ],
        }
    }

    pub fn process_data(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty data provided".to_string());
        }

        let validated_data = self.validate_data(data)?;
        let transformed_data = self.transform_data(&validated_data);
        
        self.cache.insert(key.to_string(), transformed_data.clone());
        
        Ok(transformed_data)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        let rule = &self.validation_rules[0];
        
        for &value in data {
            if rule.required && value.is_nan() {
                return Err("NaN value found in required data".to_string());
            }
            
            if let Some(min) = rule.min_value {
                if value < min {
                    return Err(format!("Value {} below minimum {}", value, min));
                }
            }
            
            if let Some(max) = rule.max_value {
                if value > max {
                    return Err(format!("Value {} above maximum {}", value, max));
                }
            }
        }
        
        Ok(data.to_vec())
    }

    fn transform_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        data.iter()
            .map(|&x| (x - mean).abs())
            .collect()
    }

    pub fn get_cached_data(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        
        let result = processor.process_data("test_key", &data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), data.len());
        
        let cached = processor.get_cached_data("test_key");
        assert!(cached.is_some());
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let invalid_data = vec![150.0];
        
        let result = processor.process_data("invalid", &invalid_data);
        assert!(result.is_err());
    }
}