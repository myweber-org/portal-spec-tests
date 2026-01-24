use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, field: &str, min: f64, max: f64) {
        self.validation_rules.push(ValidationRule {
            field_name: field.to_string(),
            min_value: min,
            max_value: max,
        });
    }

    pub fn process_dataset(&mut self, dataset: &[HashMap<String, f64>]) -> Result<Vec<HashMap<String, f64>>, String> {
        let mut results = Vec::new();

        for (index, record) in dataset.iter().enumerate() {
            match self.validate_record(record) {
                Ok(validated) => {
                    let processed = self.transform_record(&validated);
                    self.cache_record(index, &processed);
                    results.push(processed);
                }
                Err(e) => return Err(format!("Validation failed at record {}: {}", index, e)),
            }
        }

        Ok(results)
    }

    fn validate_record(&self, record: &HashMap<String, f64>) -> Result<HashMap<String, f64>, String> {
        let mut validated = HashMap::new();

        for rule in &self.validation_rules {
            if let Some(&value) = record.get(&rule.field_name) {
                if value < rule.min_value || value > rule.max_value {
                    return Err(format!("Field '{}' value {} out of range [{}, {}]", 
                        rule.field_name, value, rule.min_value, rule.max_value));
                }
                validated.insert(rule.field_name.clone(), value);
            }
        }

        Ok(validated)
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> HashMap<String, f64> {
        let mut transformed = HashMap::new();

        for (key, value) in record {
            let transformed_value = match key.as_str() {
                "temperature" => (value - 32.0) * 5.0 / 9.0,
                "pressure" => value * 1000.0,
                "humidity" => value.min(100.0).max(0.0),
                _ => *value,
            };
            transformed.insert(key.clone(), transformed_value);
        }

        transformed
    }

    fn cache_record(&mut self, index: usize, record: &HashMap<String, f64>) {
        let key = format!("record_{}", index);
        let values: Vec<f64> = record.values().copied().collect();
        self.cache.insert(key, values);
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
        processor.add_validation_rule("temperature", -50.0, 150.0);
        processor.add_validation_rule("pressure", 0.0, 100.0);

        let dataset = vec![
            [("temperature".to_string(), 68.0), ("pressure".to_string(), 14.7)]
                .iter().cloned().collect(),
            [("temperature".to_string(), 32.0), ("pressure".to_string(), 29.9)]
                .iter().cloned().collect(),
        ];

        let result = processor.process_dataset(&dataset);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule("temperature", 0.0, 100.0);

        let dataset = vec![
            [("temperature".to_string(), 120.0)]
                .iter().cloned().collect(),
        ];

        let result = processor.process_dataset(&dataset);
        assert!(result.is_err());
    }
}