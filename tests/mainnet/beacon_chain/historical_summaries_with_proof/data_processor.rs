
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
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
            data: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_dataset(&mut self, name: String, values: Vec<f64>) -> Result<(), String> {
        if name.is_empty() {
            return Err("Dataset name cannot be empty".to_string());
        }
        
        if values.is_empty() {
            return Err("Dataset values cannot be empty".to_string());
        }

        self.data.insert(name, values);
        Ok(())
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn validate_all(&self) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        for rule in &self.validation_rules {
            if let Some(values) = self.data.get(&rule.field_name) {
                let result = self.validate_dataset(values, rule);
                results.push(result);
            } else if rule.required {
                results.push(ValidationResult {
                    field_name: rule.field_name.clone(),
                    passed: false,
                    message: "Required field not found".to_string(),
                });
            }
        }
        
        results
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<Statistics> {
        self.data.get(dataset_name).map(|values| {
            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            Statistics {
                mean,
                variance,
                count: values.len(),
                min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
            }
        })
    }

    fn validate_dataset(&self, values: &[f64], rule: &ValidationRule) -> ValidationResult {
        let mut passed = true;
        let mut message = String::new();
        
        for &value in values {
            if value < rule.min_value || value > rule.max_value {
                passed = false;
                message = format!("Value {} out of range [{}, {}]", value, rule.min_value, rule.max_value);
                break;
            }
        }
        
        ValidationResult {
            field_name: rule.field_name.clone(),
            passed,
            message: if passed { "All values valid".to_string() } else { message },
        }
    }
}

pub struct Statistics {
    pub mean: f64,
    pub variance: f64,
    pub count: usize,
    pub min: f64,
    pub max: f64,
}

pub struct ValidationResult {
    pub field_name: String,
    pub passed: bool,
    pub message: String,
}

impl ValidationRule {
    pub fn new(field_name: String, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name,
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
    fn test_add_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("temperatures".to_string(), vec![20.5, 21.0, 22.3]);
        assert!(result.is_ok());
        assert_eq!(processor.data.len(), 1);
    }

    #[test]
    fn test_calculate_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("test_data".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("test_data").unwrap();
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.variance, 2.0);
        assert_eq!(stats.count, 5);
    }
}