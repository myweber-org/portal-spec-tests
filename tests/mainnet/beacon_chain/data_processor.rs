
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: u64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    RecordNotFound,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid data value"),
            ProcessingError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            ProcessingError::RecordNotFound => write!(f, "Record not found"),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: u64) -> Result<Self, ProcessingError> {
        if value < 0.0 || value > 1000.0 {
            return Err(ProcessingError::InvalidValue);
        }
        
        if timestamp == 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }
        
        Ok(Self {
            id,
            value,
            timestamp,
        })
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        if multiplier <= 0.0 {
            return Err(ProcessingError::InvalidValue);
        }
        
        self.value *= multiplier;
        Ok(())
    }
    
    pub fn validate(&self) -> bool {
        self.value >= 0.0 && self.value <= 1000.0 && self.timestamp > 0
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
    
    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.iter().find(|r| r.id == id)
    }
    
    pub fn process_records(&mut self) -> Result<Vec<f64>, ProcessingError> {
        let mut results = Vec::new();
        
        for record in &mut self.records {
            if !record.validate() {
                return Err(ProcessingError::InvalidValue);
            }
            
            record.transform(1.5)?;
            results.push(record.value);
        }
        
        Ok(results)
    }
    
    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value > threshold)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 100.0, 1234567890);
        assert!(record.is_ok());
        
        let invalid_record = DataRecord::new(2, -10.0, 1234567890);
        assert!(invalid_record.is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, 100.0, 1234567890).unwrap();
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.value, 200.0);
    }
    
    #[test]
    fn test_processor_functionality() {
        let mut processor = DataProcessor::new();
        let record = DataRecord::new(1, 50.0, 1234567890).unwrap();
        processor.add_record(record);
        
        let result = processor.process_records();
        assert!(result.is_ok());
        assert_eq!(result.unwrap()[0], 75.0);
    }
}