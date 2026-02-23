
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line_content = line?;
            
            if line_number == 0 && self.has_header {
                continue;
            }

            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), String> {
        if records.is_empty() {
            return Err("No valid records found".to_string());
        }

        let expected_len = records[0].len();
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!(
                    "Record {} has {} fields, expected {}",
                    idx + 1,
                    record.len(),
                    expected_len
                ));
            }
        }

        Ok(())
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Result<Vec<String>, String> {
        if column_index >= records[0].len() {
            return Err(format!(
                "Column index {} out of bounds (max {})",
                column_index,
                records[0].len() - 1
            ));
        }

        let column_data: Vec<String> = records
            .iter()
            .map(|record| record[column_index].clone())
            .collect();

        Ok(column_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_records(&records).is_ok());
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["x".to_string(), "y".to_string()],
            vec!["p".to_string(), "q".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&records, 1).unwrap();
        assert_eq!(column, vec!["y", "q"]);
    }
}
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

    pub fn add_dataset(&mut self, key: &str, values: Vec<f64>) {
        self.data.insert(key.to_string(), values);
    }

    pub fn set_validation_rule(&mut self, key: &str, rule: ValidationRule) {
        self.validation_rules.insert(key.to_string(), rule);
    }

    pub fn validate_dataset(&self, key: &str) -> Result<(), String> {
        let data = self.data.get(key);
        let rule = self.validation_rules.get(key);

        match (data, rule) {
            (Some(values), Some(rule)) => {
                if rule.required && values.is_empty() {
                    return Err(format!("Dataset '{}' is required but empty", key));
                }

                for &value in values {
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
                Ok(())
            }
            (None, Some(rule)) if rule.required => {
                Err(format!("Required dataset '{}' not found", key))
            }
            _ => Ok(()),
        }
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            Statistics {
                mean,
                variance,
                count: values.len(),
                min: values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                max: values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            }
        })
    }

    pub fn normalize_data(&mut self, key: &str) -> Result<(), String> {
        if let Some(values) = self.data.get_mut(key) {
            if values.is_empty() {
                return Ok(());
            }

            let stats = self.calculate_statistics(key).unwrap();
            if stats.variance == 0.0 {
                return Err("Cannot normalize data with zero variance".to_string());
            }

            for value in values.iter_mut() {
                *value = (*value - stats.mean) / stats.variance.sqrt();
            }
            Ok(())
        } else {
            Err(format!("Dataset '{}' not found", key))
        }
    }
}

pub struct Statistics {
    pub mean: f64,
    pub variance: f64,
    pub count: usize,
    pub min: f64,
    pub max: f64,
}

impl ValidationRule {
    pub fn new() -> Self {
        ValidationRule {
            min_value: None,
            max_value: None,
            required: false,
        }
    }

    pub fn with_min(mut self, min: f64) -> Self {
        self.min_value = Some(min);
        self
    }

    pub fn with_max(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_validation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("temperatures", vec![20.5, 22.1, 19.8, 23.4]);
        
        let rule = ValidationRule::new()
            .with_min(15.0)
            .with_max(30.0)
            .required();
        
        processor.set_validation_rule("temperatures", rule);
        
        assert!(processor.validate_dataset("temperatures").is_ok());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("test_data", vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        
        let stats = processor.calculate_statistics("test_data").unwrap();
        
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.count, 5);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

pub struct DataProcessor {
    validation_rules: HashMap<String, ValidationRule>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: HashMap::new(),
        }
    }

    pub fn add_validation_rule(&mut self, field: String, rule: ValidationRule) {
        self.validation_rules.insert(field, rule);
    }

    pub fn process_record(&self, record: &DataRecord) -> Result<ProcessedRecord, ProcessingError> {
        self.validate_record(record)?;
        
        let transformed_values = self.transform_values(&record.values);
        let normalized_tags = self.normalize_tags(&record.tags);
        
        Ok(ProcessedRecord {
            id: record.id,
            timestamp: record.timestamp,
            values: transformed_values,
            tags: normalized_tags,
            processed_at: chrono::Utc::now().timestamp(),
        })
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.id == 0 {
            return Err(ProcessingError::ValidationFailed("ID cannot be zero".into()));
        }

        if record.timestamp <= 0 {
            return Err(ProcessingError::ValidationFailed("Invalid timestamp".into()));
        }

        for (field, rule) in &self.validation_rules {
            if let Some(value) = record.values.get(field) {
                if !rule.validate(*value) {
                    return Err(ProcessingError::ValidationFailed(
                        format!("Field '{}' failed validation", field)
                    ));
                }
            }
        }

        Ok(())
    }

    fn transform_values(&self, values: &HashMap<String, f64>) -> HashMap<String, f64> {
        values.iter()
            .map(|(key, value)| {
                let transformed = match key.as_str() {
                    "temperature" => (value - 32.0) * 5.0 / 9.0,
                    "pressure" => value * 100.0,
                    _ => *value,
                };
                (key.clone(), transformed)
            })
            .collect()
    }

    fn normalize_tags(&self, tags: &[String]) -> Vec<String> {
        let mut normalized: Vec<String> = tags.iter()
            .map(|tag| tag.trim().to_lowercase())
            .filter(|tag| !tag.is_empty())
            .collect();
        
        normalized.sort();
        normalized.dedup();
        normalized
    }
}

pub struct ValidationRule {
    min: Option<f64>,
    max: Option<f64>,
    allowed_values: Option<Vec<f64>>,
}

impl ValidationRule {
    pub fn new() -> Self {
        ValidationRule {
            min: None,
            max: None,
            allowed_values: None,
        }
    }

    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self
    }

    pub fn with_allowed_values(mut self, values: Vec<f64>) -> Self {
        self.allowed_values = Some(values);
        self
    }

    pub fn validate(&self, value: f64) -> bool {
        if let Some(min) = self.min {
            if value < min {
                return false;
            }
        }

        if let Some(max) = self.max {
            if value > max {
                return false;
            }
        }

        if let Some(allowed) = &self.allowed_values {
            if !allowed.contains(&value) {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
    pub processed_at: i64,
}

pub fn calculate_statistics(records: &[ProcessedRecord]) -> HashMap<String, f64> {
    if records.is_empty() {
        return HashMap::new();
    }

    let mut stats = HashMap::new();
    let mut value_aggregates: HashMap<String, Vec<f64>> = HashMap::new();

    for record in records {
        for (key, value) in &record.values {
            value_aggregates.entry(key.clone())
                .or_insert_with(Vec::new)
                .push(*value);
        }
    }

    for (key, values) in value_aggregates {
        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        stats.insert(format!("{}_mean", key), mean);
        stats.insert(format!("{}_std_dev", key), std_dev);
        stats.insert(format!("{}_count", key), count);
    }

    stats
}