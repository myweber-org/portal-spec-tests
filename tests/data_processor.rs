
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: HashMap<String, ValidationRule>,
}

pub struct ValidationRule {
    min_value: Option<f64>,
    max_value: Option<f64>,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, name: String, values: Vec<f64>) {
        self.data.insert(name, values);
    }

    pub fn set_validation_rule(&mut self, field: String, rule: ValidationRule) {
        self.validation_rules.insert(field, rule);
    }

    pub fn validate_dataset(&self, dataset_name: &str) -> Result<(), String> {
        let data = self.data.get(dataset_name)
            .ok_or_else(|| format!("Dataset '{}' not found", dataset_name))?;

        if let Some(rule) = self.validation_rules.get(dataset_name) {
            if rule.required && data.is_empty() {
                return Err(format!("Dataset '{}' is required but empty", dataset_name));
            }

            for &value in data {
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
        }

        Ok(())
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
                min: values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                max: values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            }
        })
    }

    pub fn normalize_data(&mut self, dataset_name: &str) -> Result<(), String> {
        let stats = self.calculate_statistics(dataset_name)
            .ok_or_else(|| format!("Dataset '{}' not found", dataset_name))?;

        if stats.variance == 0.0 {
            return Err("Cannot normalize dataset with zero variance".to_string());
        }

        if let Some(values) = self.data.get_mut(dataset_name) {
            for value in values.iter_mut() {
                *value = (*value - stats.mean) / stats.variance.sqrt();
            }
        }

        Ok(())
    }
}

pub struct Statistics {
    pub mean: f64,
    pub variance: f64,
    pub count: usize,
    pub min: f64,
    pub max: f64,
}

impl ValidationRule {
    pub fn new() -> Self {
        ValidationRule {
            min_value: None,
            max_value: None,
            required: false,
        }
    }

    pub fn with_min(mut self, min: f64) -> Self {
        self.min_value = Some(min);
        self
    }

    pub fn with_max(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_validation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("temperatures".to_string(), vec![20.5, 22.1, 19.8, 25.3]);
        
        let rule = ValidationRule::new()
            .with_min(15.0)
            .with_max(30.0)
            .required();
        
        processor.set_validation_rule("temperatures".to_string(), rule);
        
        assert!(processor.validate_dataset("temperatures").is_ok());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("measurements".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        
        let stats = processor.calculate_statistics("measurements").unwrap();
        
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.variance, 2.0);
        assert_eq!(stats.count, 5);
    }
}use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }

        if self.values.is_empty() {
            return Err("Values cannot be empty".to_string());
        }

        for (i, &value) in self.values.iter().enumerate() {
            if !value.is_finite() {
                return Err(format!("Value at index {} is not finite", i));
            }
        }

        Ok(())
    }

    pub fn transform(&mut self, factor: f64) {
        for value in &mut self.values {
            *value *= factor;
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

pub fn process_records(records: &mut [DataRecord], factor: f64) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    for (index, record) in records.iter_mut().enumerate() {
        match record.validate() {
            Ok(_) => {
                record.transform(factor);
            }
            Err(err) => {
                errors.push(format!("Record {}: {}", index, err));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_transform() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        record.transform(2.0);
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_metadata() {
        let mut record = DataRecord::new(1, vec![1.0]);
        record.add_metadata("source".to_string(), "test".to_string());
        assert_eq!(record.metadata.get("source"), Some(&"test".to_string()));
    }
}