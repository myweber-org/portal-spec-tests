
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidValue,
    InvalidTimestamp,
    ValidationFailed(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidValue => write!(f, "Invalid data value"),
            DataError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            DataError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

pub fn validate_record(record: &DataRecord) -> Result<(), DataError> {
    if record.value.is_nan() || record.value.is_infinite() {
        return Err(DataError::InvalidValue);
    }
    
    if record.timestamp < 0 {
        return Err(DataError::InvalidTimestamp);
    }
    
    if record.value < 0.0 {
        return Err(DataError::ValidationFailed(
            "Negative values are not allowed".to_string()
        ));
    }
    
    Ok(())
}

pub fn transform_record(record: DataRecord, multiplier: f64) -> DataRecord {
    DataRecord {
        id: record.id,
        value: record.value * multiplier,
        timestamp: record.timestamp,
    }
}

pub fn process_records(
    records: Vec<DataRecord>,
    multiplier: f64,
) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        validate_record(&record)?;
        let transformed = transform_record(record, multiplier);
        processed.push(transformed);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1234567890,
        };
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_invalid_value() {
        let record = DataRecord {
            id: 1,
            value: f64::NAN,
            timestamp: 1234567890,
        };
        assert!(validate_record(&record).is_err());
    }
    
    #[test]
    fn test_transform_record() {
        let record = DataRecord {
            id: 1,
            value: 10.0,
            timestamp: 1000,
        };
        let transformed = transform_record(record, 2.5);
        assert_eq!(transformed.value, 25.0);
        assert_eq!(transformed.id, 1);
    }
}
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
            if let Some(data) = self.data.get(&rule.field_name) {
                let validation_result = self.validate_dataset(data, rule);
                results.push(validation_result);
            } else if rule.required {
                results.push(ValidationResult {
                    field_name: rule.field_name.clone(),
                    valid: false,
                    message: "Required field not found".to_string(),
                });
            }
        }

        results
    }

    fn validate_dataset(&self, data: &[f64], rule: &ValidationRule) -> ValidationResult {
        let mut invalid_count = 0;
        let mut max_violation = f64::NEG_INFINITY;
        let mut min_violation = f64::INFINITY;

        for &value in data {
            if value < rule.min_value {
                invalid_count += 1;
                min_violation = min_violation.min(value);
            }
            if value > rule.max_value {
                invalid_count += 1;
                max_violation = max_violation.max(value);
            }
        }

        if invalid_count == 0 {
            ValidationResult {
                field_name: rule.field_name.clone(),
                valid: true,
                message: "All values within valid range".to_string(),
            }
        } else {
            let mut message = format!("Found {} invalid values", invalid_count);
            if min_violation != f64::INFINITY {
                message.push_str(&format!(", min violation: {}", min_violation));
            }
            if max_violation != f64::NEG_INFINITY {
                message.push_str(&format!(", max violation: {}", max_violation));
            }
            
            ValidationResult {
                field_name: rule.field_name.clone(),
                valid: false,
                message,
            }
        }
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<DatasetStatistics> {
        self.data.get(dataset_name).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            
            let std_dev = variance.sqrt();

            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            DatasetStatistics {
                count,
                mean,
                std_dev,
                min,
                max,
                sum,
            }
        })
    }

    pub fn normalize_data(&mut self, dataset_name: &str) -> Result<(), String> {
        if let Some(values) = self.data.get_mut(dataset_name) {
            if values.is_empty() {
                return Err("Cannot normalize empty dataset".to_string());
            }

            let stats = self.calculate_statistics(dataset_name).unwrap();
            
            if stats.std_dev == 0.0 {
                return Err("Cannot normalize dataset with zero standard deviation".to_string());
            }

            for value in values.iter_mut() {
                *value = (*value - stats.mean) / stats.std_dev;
            }
            
            Ok(())
        } else {
            Err(format!("Dataset '{}' not found", dataset_name))
        }
    }
}

pub struct ValidationResult {
    field_name: String,
    valid: bool,
    message: String,
}

pub struct DatasetStatistics {
    count: usize,
    mean: f64,
    std_dev: f64,
    min: f64,
    max: f64,
    sum: f64,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }

    pub fn get_field_name(&self) -> &str {
        &self.field_name
    }
}

impl DatasetStatistics {
    pub fn get_count(&self) -> usize {
        self.count
    }

    pub fn get_mean(&self) -> f64 {
        self.mean
    }

    pub fn get_std_dev(&self) -> f64 {
        self.std_dev
    }

    pub fn get_min(&self) -> f64 {
        self.min
    }

    pub fn get_max(&self) -> f64 {
        self.max
    }

    pub fn get_sum(&self) -> f64 {
        self.sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("test_data".to_string(), vec![1.0, 2.0, 3.0]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("empty".to_string(), vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("numbers".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("numbers").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.sum, 15.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_validation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("temperature".to_string(), vec![20.5, 25.0, 30.5, 35.0]).unwrap();
        
        let rule = ValidationRule {
            field_name: "temperature".to_string(),
            min_value: 15.0,
            max_value: 30.0,
            required: true,
        };
        
        processor.add_validation_rule(rule);
        let results = processor.validate_all();
        
        assert_eq!(results.len(), 1);
        assert!(!results[0].is_valid());
    }
}