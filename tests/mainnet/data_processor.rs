
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::DuplicateTag => write!(f, "Duplicate tags are not allowed"),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ValidationError> {
        self.validate_record(&record)?;
        
        if self.records.contains_key(&record.id) {
            return Err(ValidationError::InvalidId);
        }
        
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.values().map(|r| r.value).sum()
    }

    pub fn find_records_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|r| r.tags.iter().any(|t| t == tag))
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if record.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let mut seen_tags = std::collections::HashSet::new();
        for tag in &record.tags {
            if !seen_tags.insert(tag) {
                return Err(ValidationError::DuplicateTag);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 42.5,
            tags: vec!["test".to_string(), "sample".to_string()],
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_validation_errors() {
        let processor = DataProcessor::new();
        
        let invalid_id = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 10.0,
            tags: vec![],
        };
        
        let empty_name = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 10.0,
            tags: vec![],
        };
        
        let negative_value = DataRecord {
            id: 2,
            name: "Test".to_string(),
            value: -5.0,
            tags: vec![],
        };
        
        let duplicate_tags = DataRecord {
            id: 3,
            name: "Test".to_string(),
            value: 10.0,
            tags: vec!["tag".to_string(), "tag".to_string()],
        };
        
        assert!(processor.validate_record(&invalid_id).is_err());
        assert!(processor.validate_record(&empty_name).is_err());
        assert!(processor.validate_record(&negative_value).is_err());
        assert!(processor.validate_record(&duplicate_tags).is_err());
    }

    #[test]
    fn test_calculate_total() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 10.0,
                tags: vec!["a".to_string()],
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 20.0,
                tags: vec!["b".to_string()],
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 30.0,
                tags: vec!["a".to_string(), "c".to_string()],
            },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        assert_eq!(processor.calculate_total_value(), 60.0);
        
        let tagged_records = processor.find_records_by_tag("a");
        assert_eq!(tagged_records.len(), 2);
    }
}