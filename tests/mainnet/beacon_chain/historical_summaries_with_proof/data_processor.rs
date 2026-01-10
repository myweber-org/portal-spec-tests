
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
pub enum ProcessingError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidId => write!(f, "ID must be greater than zero"),
            ProcessingError::EmptyName => write!(f, "Name cannot be empty"),
            ProcessingError::NegativeValue => write!(f, "Value cannot be negative"),
            ProcessingError::DuplicateTag => write!(f, "Duplicate tags are not allowed"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    statistics: ProcessingStats,
}

#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub total_records: usize,
    pub total_value: f64,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            statistics: ProcessingStats::default(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(&record)?;
        
        if self.records.contains_key(&record.id) {
            return Err(ProcessingError::ValidationFailed(
                format!("Record with ID {} already exists", record.id)
            ));
        }

        self.update_statistics(&record);
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        if let Some(record) = self.records.remove(&id) {
            self.statistics.total_records -= 1;
            self.statistics.total_value -= record.value;
            self.recalculate_average();
            Some(record)
        } else {
            None
        }
    }

    pub fn transform_records<F>(&mut self, transform_fn: F) -> Vec<DataRecord>
    where
        F: Fn(&DataRecord) -> DataRecord,
    {
        let mut transformed = Vec::new();
        
        for record in self.records.values() {
            let transformed_record = transform_fn(record);
            transformed.push(transformed_record);
        }
        
        transformed
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<&DataRecord>
    where
        F: Fn(&DataRecord) -> bool,
    {
        self.records.values()
            .filter(|record| predicate(record))
            .collect()
    }

    pub fn get_statistics(&self) -> &ProcessingStats {
        &self.statistics
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.id == 0 {
            return Err(ProcessingError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(ProcessingError::EmptyName);
        }
        
        if record.value < 0.0 {
            return Err(ProcessingError::NegativeValue);
        }
        
        let mut seen_tags = std::collections::HashSet::new();
        for tag in &record.tags {
            if !seen_tags.insert(tag) {
                return Err(ProcessingError::DuplicateTag);
            }
        }
        
        Ok(())
    }

    fn update_statistics(&mut self, record: &DataRecord) {
        self.statistics.total_records += 1;
        self.statistics.total_value += record.value;
        self.recalculate_average();
    }

    fn recalculate_average(&mut self) {
        if self.statistics.total_records > 0 {
            self.statistics.average_value = 
                self.statistics.total_value / self.statistics.total_records as f64;
        } else {
            self.statistics.average_value = 0.0;
        }
    }
}

pub fn process_data_batch(records: Vec<DataRecord>) -> Result<DataProcessor, ProcessingError> {
    let mut processor = DataProcessor::new();
    
    for record in records {
        processor.add_record(record)?;
    }
    
    Ok(processor)
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
            value: 100.0,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.get_statistics().total_records, 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            tags: vec![],
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_filter_records() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record A".to_string(),
                value: 50.0,
                tags: vec![],
            },
            DataRecord {
                id: 2,
                name: "Record B".to_string(),
                value: 150.0,
                tags: vec![],
            },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        let high_value = processor.filter_records(|r| r.value > 100.0);
        assert_eq!(high_value.len(), 1);
        assert_eq!(high_value[0].name, "Record B");
    }
}