
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

impl DataRecord {
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
        
        let mut seen_tags = std::collections::HashSet::new();
        for tag in &self.tags {
            if !seen_tags.insert(tag) {
                return Err(ValidationError::DuplicateTag);
            }
        }
        
        Ok(())
    }
    
    pub fn normalize(&mut self) {
        self.name = self.name.trim().to_string();
        self.tags.sort();
        self.tags.dedup();
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    statistics: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            statistics: HashMap::new(),
        }
    }
    
    pub fn add_record(&mut self, mut record: DataRecord) -> Result<(), ValidationError> {
        record.normalize();
        record.validate()?;
        self.records.push(record);
        self.update_statistics();
        Ok(())
    }
    
    pub fn process_records(&mut self) -> HashMap<String, f64> {
        let mut results = HashMap::new();
        
        if self.records.is_empty() {
            return results;
        }
        
        let total_value: f64 = self.records.iter().map(|r| r.value).sum();
        let avg_value = total_value / self.records.len() as f64;
        
        results.insert("total_records".to_string(), self.records.len() as f64);
        results.insert("total_value".to_string(), total_value);
        results.insert("average_value".to_string(), avg_value);
        
        let max_record = self.records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
        if let Some(record) = max_record {
            results.insert("max_value".to_string(), record.value);
            results.insert("max_value_id".to_string(), record.id as f64);
        }
        
        self.statistics = results.clone();
        results
    }
    
    fn update_statistics(&mut self) {
        self.process_records();
    }
    
    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }
    
    pub fn get_statistics(&self) -> &HashMap<String, f64> {
        &self.statistics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 100.0,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec![],
        };
        
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "Record 1".to_string(),
            value: 50.0,
            tags: vec!["important".to_string()],
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Record 2".to_string(),
            value: 150.0,
            tags: vec!["important".to_string(), "processed".to_string()],
        };
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        let stats = processor.process_records();
        assert_eq!(stats.get("total_records").unwrap(), &2.0);
        assert_eq!(stats.get("total_value").unwrap(), &200.0);
        assert_eq!(stats.get("average_value").unwrap(), &100.0);
        
        let filtered = processor.filter_by_tag("important");
        assert_eq!(filtered.len(), 2);
    }
}