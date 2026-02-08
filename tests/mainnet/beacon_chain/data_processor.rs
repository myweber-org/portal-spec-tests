
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let name = parts[1].to_string();
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[3].to_string();

            let record = DataRecord::new(id, name, value, category);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_stats(&self) -> (usize, Option<f64>, Option<f64>) {
        let count = self.records.len();
        
        if count == 0 {
            return (0, None, None);
        }

        let min = self.records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);
        let max = self.records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);

        (count, Some(min), Some(max))
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error")]
    TransformationError,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

pub struct DataProcessor {
    validation_rules: Vec<ValidationRule>,
    transformation_pipeline: Vec<Transformation>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformation_pipeline.push(transformation);
    }

    pub fn process(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        self.validate(record)?;
        self.transform(record)?;
        Ok(())
    }

    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        for rule in &self.validation_rules {
            if !rule.check(record) {
                return Err(ProcessingError::ValidationFailed(
                    rule.error_message.clone(),
                ));
            }
        }
        Ok(())
    }

    fn transform(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        for transformation in &self.transformation_pipeline {
            transformation.apply(record)?;
        }
        Ok(())
    }
}

pub struct ValidationRule {
    pub name: String,
    pub error_message: String,
    pub check_fn: Box<dyn Fn(&DataRecord) -> bool>,
}

impl ValidationRule {
    pub fn new<F>(name: &str, error_message: &str, check_fn: F) -> Self
    where
        F: Fn(&DataRecord) -> bool + 'static,
    {
        ValidationRule {
            name: name.to_string(),
            error_message: error_message.to_string(),
            check_fn: Box::new(check_fn),
        }
    }
}

pub trait Transformation {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError>;
}

pub struct NormalizeTransformation {
    pub field: String,
}

impl Transformation for NormalizeTransformation {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if let Some(value) = record.values.get_mut(&self.field) {
            if *value < 0.0 {
                return Err(ProcessingError::TransformationError);
            }
            *value = value.clamp(0.0, 1.0);
        }
        Ok(())
    }
}

pub struct TagFilterTransformation {
    pub allowed_tags: Vec<String>,
}

impl Transformation for TagFilterTransformation {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        record.tags.retain(|tag| self.allowed_tags.contains(tag));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();

        processor.add_validation_rule(ValidationRule::new(
            "timestamp_check",
            "Timestamp must be positive",
            |r| r.timestamp > 0,
        ));

        processor.add_transformation(NormalizeTransformation {
            field: "temperature".to_string(),
        });

        let mut record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: {
                let mut map = HashMap::new();
                map.insert("temperature".to_string(), 25.5);
                map.insert("humidity".to_string(), 0.75);
                map
            },
            tags: vec!["sensor".to_string(), "room1".to_string()],
        };

        assert!(processor.process(&mut record).is_ok());
        assert_eq!(*record.values.get("temperature").unwrap(), 0.255);
    }
}