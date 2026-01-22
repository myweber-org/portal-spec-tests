
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
        let filtered_data: Vec<f64> = data
            .iter()
            .filter(|&&value| value >= mean * self.threshold)
            .cloned()
            .collect();

        if filtered_data.is_empty() {
            return Err(ValidationError {
                message: "No data points passed the threshold filter".to_string(),
            });
        }

        Ok(filtered_data)
    }

    pub fn normalize_data(&self, data: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if data.is_empty() {
            return Err(ValidationError {
                message: "Input data cannot be empty".to_string(),
            });
        }

        let max_value = data
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        if max_value <= 0.0 {
            return Err(ValidationError {
                message: "Maximum value must be positive for normalization".to_string(),
            });
        }

        let normalized: Vec<f64> = data
            .iter()
            .map(|&value| value / max_value)
            .collect();

        Ok(normalized)
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
        let processed = result.unwrap();
        assert!(!processed.is_empty());
    }

    #[test]
    fn test_normalize_data() {
        let processor = DataProcessor::new(0.5).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = processor.normalize_data(&data);
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert_eq!(normalized.last(), Some(&1.0));
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn process_dataset(&mut self, dataset_name: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x.is_nan()) {
                return Err(format!("Field {} contains invalid values", rule.field_name));
            }
        }

        let processed: Vec<f64> = data
            .iter()
            .map(|&x| {
                if x.is_nan() {
                    0.0
                } else {
                    x.clamp(-1000.0, 1000.0)
                }
            })
            .collect();

        self.cache.insert(dataset_name.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<DatasetStats> {
        self.cache.get(dataset_name).map(|data| {
            let count = data.len();
            let sum: f64 = data.iter().sum();
            let mean = sum / count as f64;
            let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count as f64;
            
            DatasetStats {
                count,
                sum,
                mean,
                variance,
                min: *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                max: *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
            }
        })
    }
}

pub struct DatasetStats {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
}

impl ValidationRule {
    pub fn new(field_name: &str, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name: field_name.to_string(),
            min_value,
            max_value,
            required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_dataset("test_data", &data);
        assert!(result.is_ok());
        
        let stats = processor.calculate_statistics("test_data").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
    }

    #[test]
    fn test_validation_rules() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("temperature", -50.0, 150.0, true);
        processor.add_validation_rule(rule);
        
        let invalid_data = vec![f64::NAN, 25.0, 30.0];
        let result = processor.process_dataset("invalid", &invalid_data);
        assert!(result.is_err());
    }
}