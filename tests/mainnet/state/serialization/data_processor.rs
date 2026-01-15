
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidName,
    InvalidValue,
    MissingMetadata,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::InvalidName => write!(f, "Name cannot be empty"),
            ValidationError::InvalidValue => write!(f, "Value must be positive"),
            ValidationError::MissingMetadata => write!(f, "Required metadata field is missing"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Self {
        Self {
            id,
            name,
            value,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::InvalidName);
        }
        
        if self.value <= 0.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        if self.metadata.get("source").is_none() {
            return Err(ValidationError::MissingMetadata);
        }
        
        Ok(())
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }

    pub fn get_normalized_value(&self, base: f64) -> f64 {
        self.value / base
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, ValidationError> {
    let mut valid_records = Vec::new();
    
    for record in records {
        record.validate()?;
        
        let mut processed_record = record.clone();
        processed_record.transform_value(1.5);
        
        if processed_record.get_normalized_value(100.0) > 1.0 {
            processed_record.add_metadata("category".to_string(), "high".to_string());
        } else {
            processed_record.add_metadata("category".to_string(), "normal".to_string());
        }
        
        valid_records.push(processed_record);
    }
    
    Ok(valid_records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let mut record = DataRecord::new(1, "test".to_string(), 50.0);
        record.add_metadata("source".to_string(), "test_source".to_string());
        
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, "test".to_string(), 50.0);
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, "test".to_string(), 100.0);
        record.transform_value(2.0);
        
        assert_eq!(record.value, 200.0);
    }
}