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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::DuplicateTag => write!(f, "Duplicate tags are not allowed"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    tag_index: HashMap<String, Vec<u32>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            tag_index: HashMap::new(),
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }
        
        if record.value < 0.0 || record.value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        
        let mut seen_tags = std::collections::HashSet::new();
        for tag in &record.tags {
            if !seen_tags.insert(tag) {
                return Err(DataError::DuplicateTag);
            }
        }
        
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        self.validate_record(&record)?;
        
        if self.records.contains_key(&record.id) {
            return Err("Record with this ID already exists".into());
        }
        
        let id = record.id;
        self.records.insert(id, record.clone());
        
        for tag in &record.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(id);
        }
        
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_records_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        match self.tag_index.get(tag) {
            Some(ids) => ids.iter()
                .filter_map(|id| self.records.get(id))
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.values().map(|r| r.value).collect();
        
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
    }

    pub fn merge_tags(&mut self) -> HashMap<String, Vec<u32>> {
        let mut merged = HashMap::new();
        
        for (tag, ids) in &self.tag_index {
            let normalized_tag = tag.to_lowercase();
            merged.entry(normalized_tag)
                .or_insert_with(Vec::new)
                .extend(ids);
        }
        
        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec![],
        };
        
        assert!(matches!(processor.validate_record(&record), Err(DataError::InvalidId)));
    }

    #[test]
    fn test_duplicate_tags() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec!["tag1".to_string(), "tag1".to_string()],
        };
        
        assert!(matches!(processor.validate_record(&record), Err(DataError::DuplicateTag)));
    }
}