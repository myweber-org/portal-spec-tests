
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub field_name: String,
    pub min_value: f64,
    pub max_value: f64,
    pub required: bool,
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
        let mut results = Vec::new();

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
                    return Err(format!(
                        "Field '{}' value {} out of range [{}, {}]",
                        rule.field_name, value, rule.min_value, rule.max_value
                    ));
                }
            } else if rule.required {
                return Err(format!("Required field '{}' missing", rule.field_name));
            }
        }
        Ok(())
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> ProcessedRecord {
        let mut values = Vec::new();
        let mut metadata = HashMap::new();

        for (key, &value) in record {
            let transformed = match key.as_str() {
                "temperature" => (value - 32.0) * 5.0 / 9.0,
                "pressure" => value * 1000.0,
                "humidity" => value.min(100.0).max(0.0),
                _ => value,
            };

            values.push(transformed);
            metadata.insert(key.clone(), transformed);
        }

        ProcessedRecord {
            values,
            metadata,
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
    pub metadata: HashMap<String, f64>,
    pub timestamp: u64,
}

impl ProcessedRecord {
    pub fn average(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.values.iter().sum::<f64>() / self.values.len() as f64
        }
    }

    pub fn max_value(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::max)
    }

    pub fn min_value(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(ValidationRule {
            field_name: "temperature".to_string(),
            min_value: -50.0,
            max_value: 150.0,
            required: true,
        });

        let test_data = vec![
            [("temperature".to_string(), 68.0), ("humidity".to_string(), 45.0)]
                .iter()
                .cloned()
                .collect(),
        ];

        let result = processor.process_data(&test_data);
        assert!(result.is_ok());
        
        if let Ok(records) = result {
            assert_eq!(records.len(), 1);
            assert!(records[0].average() > 0.0);
        }
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(ValidationRule {
            field_name: "pressure".to_string(),
            min_value: 0.0,
            max_value: 100.0,
            required: true,
        });

        let invalid_data = vec![
            [("pressure".to_string(), 150.0)].iter().cloned().collect(),
        ];

        let result = processor.process_data(&invalid_data);
        assert!(result.is_err());
    }
}