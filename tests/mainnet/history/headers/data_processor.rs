
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

    pub fn process_dataset(&mut self, dataset_name: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x.is_nan()) {
                return Err(format!("Field {} contains invalid NaN values", rule.field_name));
            }

            if let Some(&value) = data.iter().find(|&&x| x < rule.min_value || x > rule.max_value) {
                return Err(format!(
                    "Value {} in field {} is outside allowed range [{}, {}]",
                    value, rule.field_name, rule.min_value, rule.max_value
                ));
            }
        }

        let processed_data: Vec<f64> = data
            .iter()
            .map(|&x| {
                if x.is_nan() {
                    0.0
                } else {
                    x * 2.0
                }
            })
            .collect();

        self.cache.insert(dataset_name.to_string(), processed_data.clone());

        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<DatasetStatistics> {
        self.cache.get(dataset_name).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();

            DatasetStatistics {
                mean,
                std_dev,
                min: *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                max: *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                count: data.len(),
            }
        })
    }
}

pub struct DatasetStatistics {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
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
        processor.add_validation_rule(ValidationRule::new("temperature", -50.0, 150.0, true));
        
        let data = vec![20.5, 25.0, 18.3, 22.7];
        let result = processor.process_dataset("weather", &data);
        
        assert!(result.is_ok());
        assert_eq!(processor.get_cached_data("weather").unwrap().len(), 4);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule::new("pressure", 0.0, 100.0, true));
        
        let data = vec![50.0, 120.0, 30.0];
        let result = processor.process_dataset("pressure_data", &data);
        
        assert!(result.is_err());
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    config: ProcessingConfig,
}

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub max_values: usize,
    pub min_timestamp: i64,
    pub require_metadata: bool,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.config.max_values {
            return Err(ProcessingError::ValidationFailed(
                format!("Too many values: {}", record.values.len())
            ));
        }

        if record.timestamp < self.config.min_timestamp {
            return Err(ProcessingError::ValidationFailed(
                format!("Timestamp too old: {}", record.timestamp)
            ));
        }

        if self.config.require_metadata && record.metadata.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "Metadata required but missing".to_string()
            ));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        transformed.values = transformed.values
            .into_iter()
            .map(|v| v * 2.0)
            .collect();

        transformed.metadata.insert(
            "processed_timestamp".to_string(),
            chrono::Utc::now().timestamp().to_string()
        );

        Ok(transformed)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(record)?;
            processed.push(transformed);
        }

        Ok(processed)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        let sum_all: f64 = records.iter()
            .flat_map(|r| r.values.iter())
            .sum();
        
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);
        stats.insert("average_value".to_string(), sum_all / total_values as f64);

        stats
    }
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        ProcessingConfig {
            max_values: 100,
            min_timestamp: 0,
            require_metadata: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let config = ProcessingConfig::default();
        let processor = DataProcessor::new(config);
        
        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let config = ProcessingConfig {
            max_values: 2,
            min_timestamp: 0,
            require_metadata: false,
        };
        
        let processor = DataProcessor::new(config);
        
        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        
        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0],
            metadata: HashMap::new(),
        };

        let transformed = processor.transform_record(record).unwrap();
        assert_eq!(transformed.values, vec![2.0, 4.0]);
        assert!(transformed.metadata.contains_key("processed_timestamp"));
    }
}