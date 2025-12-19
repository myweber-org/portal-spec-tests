
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    MissingTags,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::MissingTags => write!(f, "At least one tag is required"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Self {
        DataRecord {
            id,
            name,
            value,
            tags,
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        if self.tags.is_empty() {
            return Err(ValidationError::MissingTags);
        }
        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        stats.insert("value".to_string(), self.value);
        stats.insert("tag_count".to_string(), self.tags.len() as f64);
        stats.insert("name_length".to_string(), self.name.len() as f64);
        stats
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<HashMap<String, f64>>, ValidationError> {
    let mut results = Vec::new();
    
    for record in records {
        record.validate()?;
        record.transform(multiplier);
        results.push(record.get_statistics());
    }
    
    Ok(results)
}

pub fn filter_records(records: &[DataRecord], min_value: f64) -> Vec<&DataRecord> {
    records.iter()
        .filter(|r| r.value >= min_value)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(
            1,
            "test".to_string(),
            100.0,
            vec!["tag1".to_string(), "tag2".to_string()]
        );
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(
            0,
            "test".to_string(),
            100.0,
            vec!["tag1".to_string()]
        );
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_transform() {
        let mut record = DataRecord::new(
            1,
            "test".to_string(),
            100.0,
            vec!["tag".to_string()]
        );
        record.transform(2.0);
        assert_eq!(record.value, 200.0);
        assert_eq!(record.name, "TEST");
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            DataRecord::new(1, "a".to_string(), 50.0, vec!["t1".to_string()]),
            DataRecord::new(2, "b".to_string(), 150.0, vec!["t2".to_string()]),
            DataRecord::new(3, "c".to_string(), 75.0, vec!["t3".to_string()]),
        ];
        
        let filtered = filter_records(&records, 100.0);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 2);
    }
}