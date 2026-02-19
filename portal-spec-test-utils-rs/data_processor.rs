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
pub enum DataError {
    InvalidId,
    InvalidValue,
    EmptyName,
    DuplicateTag,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than zero"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::DuplicateTag => write!(f, "Tags must be unique"),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }
        
        if value < 0.0 || value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        
        let mut tag_set = HashMap::new();
        for tag in &tags {
            if tag_set.contains_key(tag) {
                return Err(DataError::DuplicateTag);
            }
            tag_set.insert(tag.clone(), true);
        }
        
        Ok(Self {
            id,
            name,
            value,
            tags,
        })
    }
    
    pub fn transform_value(&mut self, multiplier: f64) -> Result<(), DataError> {
        let new_value = self.value * multiplier;
        if new_value < 0.0 || new_value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        self.value = new_value;
        Ok(())
    }
    
    pub fn add_tag(&mut self, tag: String) -> Result<(), DataError> {
        if self.tags.contains(&tag) {
            return Err(DataError::DuplicateTag);
        }
        self.tags.push(tag);
        Ok(())
    }
    
    pub fn calculate_score(&self) -> f64 {
        let tag_bonus = self.tags.len() as f64 * 5.0;
        let name_bonus = if self.name.len() > 10 { 15.0 } else { 0.0 };
        self.value * 0.8 + tag_bonus + name_bonus
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<f64, DataError>> {
    records.iter_mut()
        .map(|record| {
            record.transform_value(1.1)?;
            record.add_tag("processed".to_string())?;
            Ok(record.calculate_score())
        })
        .collect()
}

pub fn validate_records(records: &[DataRecord]) -> Vec<Result<(), DataError>> {
    records.iter()
        .map(|record| {
            DataRecord::new(record.id, record.name.clone(), record.value, record.tags.clone())
                .map(|_| ())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(
            1,
            "Test Record".to_string(),
            100.0,
            vec!["tag1".to_string(), "tag2".to_string()]
        );
        assert!(record.is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(
            0,
            "Test".to_string(),
            100.0,
            vec![]
        );
        assert!(matches!(record, Err(DataError::InvalidId)));
    }
    
    #[test]
    fn test_calculate_score() {
        let record = DataRecord::new(
            1,
            "Very Long Name Here".to_string(),
            100.0,
            vec!["tag1".to_string(), "tag2".to_string()]
        ).unwrap();
        
        let score = record.calculate_score();
        assert!(score > 0.0);
    }
}