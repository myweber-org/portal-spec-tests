use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.id == 0 {
            return Err("Invalid record ID");
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative");
        }
        if self.values.is_empty() {
            return Err("Record must contain at least one value");
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> Option<DataStatistics> {
        if self.values.is_empty() {
            return None;
        }

        let count = self.values.len();
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count as f64;
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Some(DataStatistics {
            count,
            sum,
            mean,
            variance,
            std_dev,
        })
    }
}

#[derive(Debug)]
pub struct DataStatistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

pub fn process_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records.iter()
        .filter(|record| record.validate().is_ok())
        .filter(|record| {
            if let Some(stats) = record.calculate_statistics() {
                stats.std_dev > 0.0 && stats.mean.is_finite()
            } else {
                false
            }
        })
        .cloned()
        .collect()
}

pub fn transform_values(record: &mut DataRecord, transformer: fn(f64) -> f64) {
    record.values = record.values.iter()
        .map(|&value| transformer(value))
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(42.0);
        assert!(record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(10.0).add_value(20.0).add_value(30.0);

        let stats = record.calculate_statistics().unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.sum, 60.0);
    }

    #[test]
    fn test_process_records() {
        let mut valid_record = DataRecord::new(1, 1234567890);
        valid_record.add_value(5.0).add_value(15.0);

        let mut invalid_record = DataRecord::new(0, 1234567890);
        invalid_record.add_value(10.0);

        let records = vec![valid_record.clone(), invalid_record];
        let processed = process_records(&records);

        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].id, valid_record.id);
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

    pub fn process_dataset(&mut self, dataset_name: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x.is_nan()) {
                return Err(format!("Field {} contains invalid NaN values", rule.field_name));
            }

            if let Some(&value) = data.iter().find(|&&x| x < rule.min_value || x > rule.max_value) {
                return Err(format!("Value {} out of range for field {}", value, rule.field_name));
            }
        }

        let processed_data = self.transform_data(data);
        self.cache.insert(dataset_name.to_string(), processed_data.clone());
        
        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    fn transform_data(&self, data: Vec<f64>) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        data.into_iter()
            .map(|x| (x - mean).abs())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
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
        processor.add_validation_rule(ValidationRule::new("temperature", -50.0, 100.0, true));
        
        let data = vec![25.5, 30.2, 22.8, 18.9];
        let result = processor.process_dataset("weather", data);
        
        assert!(result.is_ok());
        assert_eq!(processor.get_cached_data("weather").unwrap().len(), 4);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule::new("pressure", 950.0, 1050.0, true));
        
        let invalid_data = vec![920.5, 980.2, 1020.8];
        let result = processor.process_dataset("pressure_readings", invalid_data);
        
        assert!(result.is_err());
    }
}