
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    EmptyCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::EmptyCategory => write!(f, "Category cannot be empty"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, ValidationError> {
        if id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if value < 0.0 || value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        if category.trim().is_empty() {
            return Err(ValidationError::EmptyCategory);
        }
        
        Ok(Self {
            id,
            value,
            category: category.trim().to_string(),
        })
    }
    
    pub fn transform_value(&mut self, multiplier: f64) -> Result<(), ValidationError> {
        let new_value = self.value * multiplier;
        
        if new_value < 0.0 || new_value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        self.value = new_value;
        Ok(())
    }
    
    pub fn normalize_value(&mut self, max_value: f64) {
        if max_value > 0.0 {
            self.value = (self.value / max_value) * 100.0;
        }
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
    
    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }
    
    pub fn process_records(&mut self) -> Vec<DataRecord> {
        let max_value = self.records
            .iter()
            .map(|r| r.value)
            .fold(0.0, f64::max);
        
        for record in &mut self.records {
            record.normalize_value(max_value);
        }
        
        self.records.clone()
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }
    
    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 100.0, "test".to_string());
        assert!(record.is_ok());
        
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 100.0);
        assert_eq!(record.category, "test");
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 100.0, "test".to_string());
        assert!(matches!(record, Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_empty_category() {
        let record = DataRecord::new(1, 100.0, "   ".to_string());
        assert!(matches!(record, Err(ValidationError::EmptyCategory)));
    }
    
    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, 100.0, "test".to_string()).unwrap();
        assert!(record.transform_value(2.0).is_ok());
        assert_eq!(record.value, 200.0);
    }
    
    #[test]
    fn test_invalid_transformation() {
        let mut record = DataRecord::new(1, 500.0, "test".to_string()).unwrap();
        let result = record.transform_value(3.0);
        assert!(matches!(result, Err(ValidationError::InvalidValue)));
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_data(&mut self, key: &str, values: &[f64]) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let processed: Vec<f64> = values
            .iter()
            .filter(|&&x| x.is_finite())
            .map(|&x| x * 2.0)
            .collect();

        if processed.len() != values.len() {
            return Err("Invalid data points detected".to_string());
        }

        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<(f64, f64, f64)> {
        self.cache.get(key).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();
            
            (mean, variance, std_dev)
        })
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
        let data = vec![1.0, 2.0, 3.0];
        
        let result = processor.process_data("test", &data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_empty_data() {
        let mut processor = DataProcessor::new();
        let result = processor.process_data("empty", &[]);
        assert!(result.is_err());
    }
}