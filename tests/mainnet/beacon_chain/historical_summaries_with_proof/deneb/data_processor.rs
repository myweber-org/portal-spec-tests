use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Transformation failed")]
    TransformationFailed,
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub metadata: Option<HashMap<String, String>>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationError("Timestamp cannot be negative".to_string()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::ValidationError("Values cannot be empty".to_string()));
        }
        
        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationError("Key cannot be empty".to_string()));
            }
            
            if !value.is_finite() {
                return Err(DataError::ValidationError(
                    format!("Value for key '{}' must be finite", key)
                ));
            }
        }
        
        Ok(())
    }
    
    pub fn transform_values<F>(&mut self, transformer: F) -> Result<(), DataError>
    where
        F: Fn(f64) -> Result<f64, DataError>,
    {
        let mut transformed = HashMap::new();
        
        for (key, value) in &self.values {
            match transformer(*value) {
                Ok(transformed_value) => {
                    transformed.insert(key.clone(), transformed_value);
                }
                Err(e) => {
                    return Err(DataError::TransformationFailed);
                }
            }
        }
        
        self.values = transformed;
        Ok(())
    }
    
    pub fn normalize(&mut self) -> Result<(), DataError> {
        if self.values.is_empty() {
            return Ok(());
        }
        
        let sum: f64 = self.values.values().sum();
        if sum.abs() < f64::EPSILON {
            return Err(DataError::TransformationFailed);
        }
        
        let mut normalized = HashMap::new();
        for (key, value) in &self.values {
            normalized.insert(key.clone(), value / sum);
        }
        
        self.values = normalized;
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::new();
    
    for record in records {
        match record.validate() {
            Ok(_) => {
                let mut processed_record = record.clone();
                
                processed_record.transform_values(|x| {
                    if x < 0.0 {
                        Err(DataError::TransformationFailed)
                    } else {
                        Ok(x.ln())
                    }
                })?;
                
                processed_record.normalize()?;
                processed.push(processed_record);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), 25.5);
        values.insert("humidity".to_string(), 60.0);
        
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values,
            metadata: None,
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_record() {
        let mut values = HashMap::new();
        values.insert("".to_string(), f64::NAN);
        
        let record = DataRecord {
            id: 0,
            timestamp: -1,
            values,
            metadata: None,
        };
        
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_normalization() {
        let mut values = HashMap::new();
        values.insert("a".to_string(), 1.0);
        values.insert("b".to_string(), 2.0);
        values.insert("c".to_string(), 3.0);
        
        let mut record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values,
            metadata: None,
        };
        
        assert!(record.normalize().is_ok());
        let sum: f64 = record.values.values().sum();
        assert!((sum - 1.0).abs() < f64::EPSILON);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, key: String, values: Vec<f64>) -> Result<(), String> {
        if values.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }
        
        for &value in &values {
            if !value.is_finite() {
                return Err("Dataset contains non-finite values".to_string());
            }
        }
        
        self.data.insert(key, values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            
            let std_dev = variance.sqrt();
            
            Statistics {
                count,
                mean,
                std_dev,
                min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            }
        })
    }

    pub fn normalize_data(&self, key: &str) -> Option<Vec<f64>> {
        self.data.get(key).map(|values| {
            let stats = self.calculate_statistics(key).unwrap();
            values.iter()
                .map(|&x| (x - stats.mean) / stats.std_dev)
                .collect()
        })
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("test".to_string(), vec![1.0, 2.0, 3.0]);
        assert!(result.is_ok());
        assert_eq!(processor.get_keys(), vec!["test"]);
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
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_normalize_data() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("values".to_string(), vec![2.0, 4.0, 6.0]).unwrap();
        
        let normalized = processor.normalize_data("values").unwrap();
        assert_eq!(normalized.len(), 3);
        assert!((normalized.iter().sum::<f64>()).abs() < 1e-10);
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
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

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value < 0.0 || record.name.is_empty())
            .collect()
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, Vec<String>) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let categories: Vec<String> = self
            .records
            .iter()
            .map(|r| r.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        (count, avg, categories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,15.7,Category1").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let filtered = processor.filter_by_category("Category1");
        assert_eq!(filtered.len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.5).abs() < 0.1);
        
        let (count, _, categories) = processor.get_statistics();
        assert_eq!(count, 3);
        assert_eq!(categories.len(), 2);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn register_validator(&mut self, name: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validators.insert(name.to_string(), validator);
    }

    pub fn register_transformer(&mut self, name: &str, transformer: Box<dyn Fn(String) -> String>) {
        self.transformers.insert(name.to_string(), transformer);
    }

    pub fn validate(&self, name: &str, data: &str) -> bool {
        self.validators
            .get(name)
            .map(|validator| validator(data))
            .unwrap_or(false)
    }

    pub fn transform(&self, name: &str, data: String) -> Option<String> {
        self.transformers
            .get(name)
            .map(|transformer| transformer(data))
    }

    pub fn process_pipeline(&self, data: &str, validators: &[&str], transformers: &[&str]) -> Option<String> {
        for validator_name in validators {
            if !self.validate(validator_name, data) {
                return None;
            }
        }

        let mut result = data.to_string();
        for transformer_name in transformers {
            if let Some(transformed) = self.transform(transformer_name, result) {
                result = transformed;
            } else {
                return None;
            }
        }

        Some(result)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.register_validator("is_numeric", Box::new(|s| s.chars().all(|c| c.is_ascii_digit())));
    processor.register_validator("is_alpha", Box::new(|s| s.chars().all(|c| c.is_ascii_alphabetic())));

    processor.register_transformer("to_uppercase", Box::new(|s| s.to_uppercase()));
    processor.register_transformer("reverse", Box::new(|s| s.chars().rev().collect()));
    processor.register_transformer("trim_spaces", Box::new(|s| s.trim().to_string()));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("is_numeric", "12345"));
        assert!(!processor.validate("is_numeric", "123a45"));
        assert!(processor.validate("is_alpha", "abc"));
        assert!(!processor.validate("is_alpha", "abc123"));
    }

    #[test]
    fn test_transformation() {
        let processor = create_default_processor();
        assert_eq!(processor.transform("to_uppercase", "hello".to_string()), Some("HELLO".to_string()));
        assert_eq!(processor.transform("reverse", "rust".to_string()), Some("tsur".to_string()));
        assert_eq!(processor.transform("trim_spaces", "  data  ".to_string()), Some("data".to_string()));
    }

    #[test]
    fn test_pipeline() {
        let processor = create_default_processor();
        let result = processor.process_pipeline("123", &["is_numeric"], &["reverse"]);
        assert_eq!(result, Some("321".to_string()));

        let result = processor.process_pipeline("abc123", &["is_alpha"], &["to_uppercase"]);
        assert_eq!(result, None);
    }
}