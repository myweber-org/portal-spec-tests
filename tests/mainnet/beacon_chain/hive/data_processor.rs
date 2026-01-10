
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

    pub fn add_dataset(&mut self, name: &str, values: Vec<f64>) {
        self.data.insert(name.to_string(), values);
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn validate_all(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for rule in &self.validation_rules {
            if let Some(data_values) = self.data.get(&rule.field_name) {
                if rule.required && data_values.is_empty() {
                    errors.push(format!("Field '{}' is required but empty", rule.field_name));
                }

                for &value in data_values {
                    if value < rule.min_value || value > rule.max_value {
                        errors.push(format!(
                            "Value {} in field '{}' is outside valid range [{}, {}]",
                            value, rule.field_name, rule.min_value, rule.max_value
                        ));
                    }
                }
            } else if rule.required {
                errors.push(format!("Required field '{}' not found", rule.field_name));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn calculate_statistics(&self, field_name: &str) -> Option<Statistics> {
        self.data.get(field_name).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = if count > 0 { sum / count as f64 } else { 0.0 };
            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            Statistics {
                count,
                sum,
                mean,
                min,
                max,
            }
        })
    }

    pub fn normalize_data(&mut self, field_name: &str) -> Result<(), String> {
        if let Some(values) = self.data.get_mut(field_name) {
            if values.is_empty() {
                return Err("Cannot normalize empty dataset".to_string());
            }

            let stats = self.calculate_statistics(field_name).unwrap();
            let range = stats.max - stats.min;

            if range == 0.0 {
                return Err("Cannot normalize dataset with zero range".to_string());
            }

            for value in values.iter_mut() {
                *value = (*value - stats.min) / range;
            }

            Ok(())
        } else {
            Err(format!("Field '{}' not found", field_name))
        }
    }
}

pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("temperatures", vec![20.5, 22.0, 18.5, 25.0, 19.5]);
        
        let rule = ValidationRule::new("temperatures", 15.0, 30.0, true);
        processor.add_validation_rule(rule);

        assert!(processor.validate_all().is_ok());
        
        let stats = processor.calculate_statistics("temperatures").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.min, 18.5);
        assert_eq!(stats.max, 25.0);
    }

    #[test]
    fn test_normalization() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("scores", vec![10.0, 20.0, 30.0, 40.0, 50.0]);
        
        assert!(processor.normalize_data("scores").is_ok());
        
        let normalized = processor.data.get("scores").unwrap();
        assert_eq!(normalized[0], 0.0);
        assert_eq!(normalized[4], 1.0);
    }
}