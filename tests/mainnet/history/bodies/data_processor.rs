
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

    pub fn add_dataset(&mut self, name: String, values: Vec<f64>) -> Result<(), String> {
        if self.data.contains_key(&name) {
            return Err(format!("Dataset '{}' already exists", name));
        }
        
        if let Some(rule) = self.validation_rules.get(&name) {
            if rule.required && values.is_empty() {
                return Err(format!("Dataset '{}' cannot be empty", name));
            }
            
            for &value in &values {
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
        
        self.data.insert(name, values);
        Ok(())
    }

    pub fn set_validation_rule(&mut self, dataset_name: String, rule: ValidationRule) {
        self.validation_rules.insert(dataset_name, rule);
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<Statistics> {
        self.data.get(dataset_name).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = if count > 0 { sum / count as f64 } else { 0.0 };
            
            let variance = if count > 1 {
                let squared_diff: f64 = values.iter()
                    .map(|&x| (x - mean).powi(2))
                    .sum();
                squared_diff / (count - 1) as f64
            } else {
                0.0
            };
            
            Statistics {
                count,
                sum,
                mean,
                variance,
                std_dev: variance.sqrt(),
            }
        })
    }

    pub fn transform_data<F>(&mut self, dataset_name: &str, transform_fn: F) -> Result<(), String>
    where
        F: Fn(f64) -> f64,
    {
        if let Some(values) = self.data.get_mut(dataset_name) {
            for value in values {
                *value = transform_fn(*value);
            }
            Ok(())
        } else {
            Err(format!("Dataset '{}' not found", dataset_name))
        }
    }

    pub fn merge_datasets(&mut self, target_name: String, source_names: Vec<&str>) -> Result<(), String> {
        let mut merged_data = Vec::new();
        
        for &source_name in &source_names {
            if let Some(data) = self.data.get(source_name) {
                merged_data.extend(data.clone());
            } else {
                return Err(format!("Source dataset '{}' not found", source_name));
            }
        }
        
        self.data.insert(target_name, merged_data);
        Ok(())
    }
}

pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

impl ValidationRule {
    pub fn new(min: Option<f64>, max: Option<f64>, required: bool) -> Self {
        ValidationRule {
            min_value: min,
            max_value: max,
            required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_validate_dataset() {
        let mut processor = DataProcessor::new();
        
        let rule = ValidationRule::new(Some(0.0), Some(100.0), true);
        processor.set_validation_rule("scores".to_string(), rule);
        
        let result = processor.add_dataset("scores".to_string(), vec![85.5, 92.0, 78.3]);
        assert!(result.is_ok());
        
        let invalid_result = processor.add_dataset("scores".to_string(), vec![120.0]);
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("test").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.sum, 15.0);
        assert_eq!(stats.mean, 3.0);
    }
}
use std::collections::HashMap;

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

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }

        if self.values.is_empty() {
            return Err("Values cannot be empty".to_string());
        }

        for &value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err(format!("Invalid value detected: {}", value));
            }
        }

        Ok(())
    }

    pub fn normalize(&mut self) {
        if let Some(max) = self.values.iter().copied().reduce(f64::max) {
            if max != 0.0 {
                for value in &mut self.values {
                    *value /= max;
                }
            }
        }
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<DataRecord, String>> {
    let mut results = Vec::new();

    for record in records {
        match record.validate() {
            Ok(_) => {
                let mut processed = record.clone();
                processed.normalize();
                results.push(Ok(processed));
            }
            Err(e) => {
                results.push(Err(format!("Record {} failed: {}", record.id, e)));
            }
        }
    }

    results
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
    fn test_invalid_record() {
        let record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![2.0, 4.0, 6.0]);
        record.normalize();
        assert_eq!(record.values, vec![1.0 / 3.0, 2.0 / 3.0, 1.0]);
    }
}