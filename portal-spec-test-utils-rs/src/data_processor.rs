
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn process(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn batch_process(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());
        
        for record in records {
            match self.process(record) {
                Ok(processed) => results.push(processed),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    
    processor.add_validation_rule(|record| {
        if record.values.is_empty() {
            Err(ProcessingError::ValidationError("Empty values array".to_string()))
        } else {
            Ok(())
        }
    });
    
    processor.add_validation_rule(|record| {
        if record.timestamp < 0 {
            Err(ProcessingError::ValidationError("Negative timestamp".to_string()))
        } else {
            Ok(())
        }
    });
    
    processor.add_transformation(|mut record| {
        let sum: f64 = record.values.iter().sum();
        let count = record.values.len() as f64;
        let average = sum / count;
        
        record.metadata.insert("average".to_string(), average.to_string());
        record.metadata.insert("count".to_string(), count.to_string());
        
        Ok(record)
    });
    
    processor.add_transformation(|mut record| {
        record.values = record.values.iter().map(|&x| x * 2.0).collect();
        Ok(record)
    });
    
    processor
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_processing() {
        let processor = create_default_processor();
        
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0, 4.0],
            metadata,
        };
        
        let result = processor.process(record).unwrap();
        
        assert_eq!(result.values, vec![2.0, 4.0, 6.0, 8.0]);
        assert_eq!(result.metadata.get("average").unwrap(), "2.5");
        assert_eq!(result.metadata.get("count").unwrap(), "4");
    }
    
    #[test]
    fn test_validation_error() {
        let processor = create_default_processor();
        
        let record = DataRecord {
            id: 2,
            timestamp: -100,
            values: vec![1.0, 2.0],
            metadata: HashMap::new(),
        };
        
        let result = processor.process(record);
        assert!(result.is_err());
    }
}