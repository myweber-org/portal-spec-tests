
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
            if !self.validate_data(&data, rule) {
                return Err(format!("Validation failed for rule: {}", rule.field_name));
            }
        }

        let processed_data = self.transform_data(data);
        self.cache.insert(dataset_name.to_string(), processed_data.clone());
        
        Ok(processed_data)
    }

    fn validate_data(&self, data: &[f64], rule: &ValidationRule) -> bool {
        if rule.required && data.is_empty() {
            return false;
        }

        for &value in data {
            if value < rule.min_value || value > rule.max_value {
                return false;
            }
        }
        true
    }

    fn transform_data(&self, mut data: Vec<f64>) -> Vec<f64> {
        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        data.iter().map(|&x| (x - mean).abs()).collect()
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
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
        let rule = ValidationRule::new("temperature", -50.0, 100.0, true);
        processor.add_validation_rule(rule);

        let test_data = vec![25.5, 30.2, 18.7, 22.1];
        let result = processor.process_dataset("weather", test_data);

        assert!(result.is_ok());
        assert!(processor.get_cached_data("weather").is_some());
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("pressure", 0.0, 10.0, true);
        processor.add_validation_rule(rule);

        let invalid_data = vec![15.0, 5.0, 8.0];
        let result = processor.process_dataset("invalid", invalid_data);

        assert!(result.is_err());
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum DataError {
    InvalidValue,
    InvalidCategory,
    EmptyData,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidValue => write!(f, "Value must be positive"),
            DataError::InvalidCategory => write!(f, "Category cannot be empty"),
            DataError::EmptyData => write!(f, "No data records provided"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        Self::validate_record(&record)?;
        self.records.push(record);
        Ok(())
    }

    pub fn process_records(&self) -> Result<Vec<DataRecord>, DataError> {
        if self.records.is_empty() {
            return Err(DataError::EmptyData);
        }

        let mut processed = Vec::with_capacity(self.records.len());
        for record in &self.records {
            let transformed = DataRecord {
                id: record.id,
                value: record.value * 1.1,
                category: record.category.to_uppercase(),
            };
            processed.push(transformed);
        }

        Ok(processed)
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    fn validate_record(record: &DataRecord) -> Result<(), DataError> {
        if record.value <= 0.0 {
            return Err(DataError::InvalidValue);
        }

        if record.category.trim().is_empty() {
            return Err(DataError::InvalidCategory);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            category: "test".to_string(),
        };
        assert!(DataProcessor::validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_value() {
        let record = DataRecord {
            id: 1,
            value: -5.0,
            category: "test".to_string(),
        };
        assert!(matches!(
            DataProcessor::validate_record(&record),
            Err(DataError::InvalidValue)
        ));
    }

    #[test]
    fn test_process_records() {
        let mut processor = DataProcessor::new();
        processor
            .add_record(DataRecord {
                id: 1,
                value: 10.0,
                category: "alpha".to_string(),
            })
            .unwrap();

        let processed = processor.process_records().unwrap();
        assert_eq!(processed[0].value, 11.0);
        assert_eq!(processed[0].category, "ALPHA");
    }
}