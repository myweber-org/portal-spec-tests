
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum DataError {
    InvalidFormat,
    OutOfRange,
    ConversionFailed,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidFormat => write!(f, "Data format is invalid"),
            DataError::OutOfRange => write!(f, "Value is out of acceptable range"),
            DataError::ConversionFailed => write!(f, "Failed to convert data type"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, DataError> {
        if threshold < 0.0 || threshold > 100.0 {
            return Err(DataError::OutOfRange);
        }
        Ok(Self { threshold })
    }

    pub fn process_value(&self, input: &str) -> Result<f64, DataError> {
        let parsed = input.parse::<f64>().map_err(|_| DataError::InvalidFormat)?;
        
        if parsed < 0.0 {
            return Err(DataError::OutOfRange);
        }

        let processed = (parsed * 1.5).min(self.threshold);
        Ok(processed)
    }

    pub fn batch_process(&self, inputs: &[&str]) -> Vec<Result<f64, DataError>> {
        inputs.iter().map(|&input| self.process_value(input)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processing() {
        let processor = DataProcessor::new(50.0).unwrap();
        let result = processor.process_value("10.5").unwrap();
        assert_eq!(result, 15.75);
    }

    #[test]
    fn test_threshold_limit() {
        let processor = DataProcessor::new(20.0).unwrap();
        let result = processor.process_value("50.0").unwrap();
        assert_eq!(result, 20.0);
    }

    #[test]
    fn test_invalid_input() {
        let processor = DataProcessor::new(50.0).unwrap();
        let result = processor.process_value("invalid");
        assert!(matches!(result, Err(DataError::InvalidFormat)));
    }
}use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: vec![
                ValidationRule {
                    min_value: Some(0.0),
                    max_value: Some(100.0),
                    required: true,
                },
                ValidationRule {
                    min_value: Some(-50.0),
                    max_value: Some(50.0),
                    required: false,
                },
            ],
        }
    }

    pub fn process_data(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty data provided".to_string());
        }

        let validated_data = self.validate_data(data)?;
        let transformed_data = self.transform_data(&validated_data);
        
        self.cache.insert(key.to_string(), transformed_data.clone());
        
        Ok(transformed_data)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        let rule = &self.validation_rules[0];
        
        for &value in data {
            if rule.required && value.is_nan() {
                return Err("NaN value found in required data".to_string());
            }
            
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
        
        Ok(data.to_vec())
    }

    fn transform_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        data.iter()
            .map(|&x| (x - mean).abs())
            .collect()
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
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        
        let result = processor.process_data("test_key", &data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), data.len());
        
        let cached = processor.get_cached_data("test_key");
        assert!(cached.is_some());
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let invalid_data = vec![150.0];
        
        let result = processor.process_data("invalid", &invalid_data);
        assert!(result.is_err());
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

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    
    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        validate_record(&record)?;
        records.push(record);
    }
    
    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    
    if record.value < 0.0 {
        return Err("Value cannot be negative".to_string());
    }
    
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Invalid category".to_string());
    }
    
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Test1,10.5,A").unwrap();
        writeln!(temp_file, "2,Test2,20.0,B").unwrap();
        
        let result = process_data_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "C".to_string() },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!(std_dev > 8.16 && std_dev < 8.17);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ProcessingError {
    InvalidInput(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_rules: Vec<Box<dyn Fn(&str) -> Result<(), ProcessingError>>>,
    transformation_rules: Vec<Box<dyn Fn(String) -> Result<String, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&str) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation_rule<F>(&mut self, rule: F)
    where
        F: Fn(String) -> Result<String, ProcessingError> + 'static,
    {
        self.transformation_rules.push(Box::new(rule));
    }

    pub fn process(&self, input: &str) -> Result<String, ProcessingError> {
        for rule in &self.validation_rules {
            rule(input)?;
        }

        let mut result = input.to_string();
        for rule in &self.transformation_rules {
            result = rule(result)?;
        }

        Ok(result)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validation_rule(|input| {
        if input.is_empty() {
            Err(ProcessingError::InvalidInput("Input cannot be empty".to_string()))
        } else {
            Ok(())
        }
    });

    processor.add_validation_rule(|input| {
        if input.len() > 1000 {
            Err(ProcessingError::InvalidInput("Input too long".to_string()))
        } else {
            Ok(())
        }
    });

    processor.add_transformation_rule(|input| {
        Ok(input.trim().to_string())
    });

    processor.add_transformation_rule(|input| {
        Ok(input.to_uppercase())
    });

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let processor = create_default_processor();
        let result = processor.process("");
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_processing() {
        let processor = create_default_processor();
        let result = processor.process("  hello world  ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HELLO WORLD");
    }

    #[test]
    fn test_long_input() {
        let processor = create_default_processor();
        let long_input = "a".repeat(1001);
        let result = processor.process(&long_input);
        assert!(result.is_err());
    }
}