
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

    pub fn process_data(&mut self, dataset: &[HashMap<String, f64>]) -> Result<Vec<ProcessedRecord>, String> {
        let mut results = Vec::with_capacity(dataset.len());

        for (index, record) in dataset.iter().enumerate() {
            match self.validate_record(record) {
                Ok(_) => {
                    let processed = self.transform_record(record);
                    self.cache.insert(format!("record_{}", index), processed.values.clone());
                    results.push(processed);
                }
                Err(e) => return Err(format!("Validation failed at record {}: {}", index, e)),
            }
        }

        Ok(results)
    }

    fn validate_record(&self, record: &HashMap<String, f64>) -> Result<(), String> {
        for rule in &self.validation_rules {
            if let Some(&value) = record.get(&rule.field_name) {
                if value < rule.min_value || value > rule.max_value {
                    return Err(format!("Field '{}' value {} out of range [{}, {}]", 
                        rule.field_name, value, rule.min_value, rule.max_value));
                }
            } else if rule.required {
                return Err(format!("Required field '{}' missing", rule.field_name));
            }
        }
        Ok(())
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> ProcessedRecord {
        let mut values = Vec::new();
        let mut sum = 0.0;

        for (key, &value) in record {
            let transformed = match key.as_str() {
                "temperature" => (value - 32.0) * 5.0 / 9.0,
                "pressure" => value * 0.01,
                _ => value,
            };
            values.push(transformed);
            sum += transformed;
        }

        let average = if !values.is_empty() {
            sum / values.len() as f64
        } else {
            0.0
        };

        ProcessedRecord {
            values,
            average,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn get_cached_data(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

pub struct ProcessedRecord {
    pub values: Vec<f64>,
    pub average: f64,
    pub timestamp: u64,
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
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule::new("temperature".to_string(), -50.0, 150.0, true));
        
        let mut record = HashMap::new();
        record.insert("temperature".to_string(), 68.0);
        record.insert("pressure".to_string(), 1013.25);
        
        let dataset = vec![record];
        let result = processor.process_data(&dataset);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule::new("temperature".to_string(), 0.0, 100.0, true));
        
        let mut record = HashMap::new();
        record.insert("temperature".to_string(), 150.0);
        
        let dataset = vec![record];
        let result = processor.process_data(&dataset);
        
        assert!(result.is_err());
    }
}