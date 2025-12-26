use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_tax(&self, rate: f64) -> f64 {
        self.value * rate
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
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
            if parts.len() != 3 {
                continue;
            }

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();

            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(e) => eprintln!("Invalid record at line {}: {}", line_num + 1, e),
            }
        }

        Ok(count)
    }

    pub fn total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.total_value() / self.records.len() as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 100.0, "A".to_string()).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 100.0);
        assert_eq!(record.category, "A");
    }

    #[test]
    fn test_invalid_record() {
        assert!(DataRecord::new(1, -10.0, "A".to_string()).is_err());
        assert!(DataRecord::new(1, 10.0, "".to_string()).is_err());
    }

    #[test]
    fn test_tax_calculation() {
        let record = DataRecord::new(1, 100.0, "A".to_string()).unwrap();
        assert_eq!(record.calculate_tax(0.1), 10.0);
    }

    #[test]
    fn test_data_processor() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,value,category").unwrap();
        writeln!(file, "1,100.0,A").unwrap();
        writeln!(file, "2,200.0,B").unwrap();
        writeln!(file, "3,300.0,A").unwrap();

        let mut processor = DataProcessor::new();
        let count = processor.load_from_csv(file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.total_value(), 600.0);
        assert_eq!(processor.filter_by_category("A").len(), 2);
        assert_eq!(processor.average_value(), Some(200.0));
    }
}use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    OutOfRange(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

pub struct DataProcessor {
    validation_rules: HashMap<String, ValidationRule>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            validation_rules: HashMap::new(),
        }
    }

    pub fn add_validation_rule(&mut self, field: &str, rule: ValidationRule) {
        self.validation_rules.insert(field.to_string(), rule);
    }

    pub fn process_record(&self, record: &DataRecord) -> Result<ProcessedRecord, ProcessingError> {
        self.validate_record(record)?;
        
        let normalized_values = self.normalize_values(&record.values);
        let computed_metrics = self.compute_metrics(&normalized_values);
        
        Ok(ProcessedRecord {
            original_id: record.id,
            processed_timestamp: chrono::Utc::now().timestamp(),
            normalized_values,
            metrics: computed_metrics,
            metadata: record.metadata.clone(),
        })
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::InvalidFormat);
        }

        for (field, rule) in &self.validation_rules {
            match field.as_str() {
                "id" => rule.validate_number(record.id as f64)?,
                "timestamp" => rule.validate_number(record.timestamp as f64)?,
                _ => {
                    if let Some(value) = record.metadata.get(field) {
                        rule.validate_string(value)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn normalize_values(&self, values: &[f64]) -> Vec<f64> {
        if values.is_empty() {
            return Vec::new();
        }

        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let range = max - min;

        if range.abs() < f64::EPSILON {
            return vec![0.0; values.len()];
        }

        values
            .iter()
            .map(|&v| (v - min) / range)
            .collect()
    }

    fn compute_metrics(&self, values: &[f64]) -> Metrics {
        let count = values.len() as f64;
        
        if count == 0.0 {
            return Metrics::default();
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();

        Metrics {
            count: count as u32,
            mean,
            std_dev,
            min: values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            max: values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub required: bool,
    pub pattern: Option<regex::Regex>,
}

impl ValidationRule {
    pub fn validate_number(&self, value: f64) -> Result<(), ProcessingError> {
        if let Some(min) = self.min {
            if value < min {
                return Err(ProcessingError::OutOfRange(format!("Value {} below minimum {}", value, min)));
            }
        }
        
        if let Some(max) = self.max {
            if value > max {
                return Err(ProcessingError::OutOfRange(format!("Value {} above maximum {}", value, max)));
            }
        }
        
        Ok(())
    }

    pub fn validate_string(&self, value: &str) -> Result<(), ProcessingError> {
        if self.required && value.is_empty() {
            return Err(ProcessingError::MissingField("string field".to_string()));
        }
        
        if let Some(pattern) = &self.pattern {
            if !pattern.is_match(value) {
                return Err(ProcessingError::InvalidFormat);
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedRecord {
    pub original_id: u32,
    pub processed_timestamp: i64,
    pub normalized_values: Vec<f64>,
    pub metrics: Metrics,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Metrics {
    pub count: u32,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_values() {
        let processor = DataProcessor::new();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = processor.normalize_values(&values);
        
        assert_eq!(normalized.len(), 5);
        assert!((normalized[0] - 0.0).abs() < f64::EPSILON);
        assert!((normalized[4] - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_metrics() {
        let processor = DataProcessor::new();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let metrics = processor.compute_metrics(&values);
        
        assert_eq!(metrics.count, 5);
        assert!((metrics.mean - 3.0).abs() < f64::EPSILON);
        assert!((metrics.std_dev - 1.4142135623730951).abs() < f64::EPSILON);
        assert!((metrics.min - 1.0).abs() < f64::EPSILON);
        assert!((metrics.max - 5.0).abs() < f64::EPSILON);
    }
}