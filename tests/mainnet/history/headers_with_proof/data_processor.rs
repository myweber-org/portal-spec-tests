use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
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
    validation_threshold: f64,
    transformation_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, transformation_factor: f64) -> Self {
        DataProcessor {
            validation_threshold,
            transformation_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record contains no values".to_string(),
            ));
        }

        for (i, &value) in record.values.iter().enumerate() {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::InvalidData(format!(
                    "Invalid value at position {}: {}",
                    i, value
                )));
            }
            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::ValidationError(format!(
                    "Value {} exceeds threshold {} at position {}",
                    value, self.validation_threshold, i
                )));
            }
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::TransformationFailed(
                "Cannot transform empty record".to_string(),
            ));
        }

        for value in record.values.iter_mut() {
            *value = (*value * self.transformation_factor).sin().abs();
            
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::TransformationFailed(
                    "Transformation produced invalid result".to_string(),
                ));
            }
        }

        record.metadata.insert(
            "transformation_applied".to_string(),
            format!("factor_{}", self.transformation_factor),
        );

        Ok(())
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        let all_values: Vec<f64> = records.iter().flat_map(|r| r.values.clone()).collect();

        if !all_values.is_empty() {
            let sum: f64 = all_values.iter().sum();
            let count = all_values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = all_values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();

            stats.insert("mean".to_string(), mean);
            stats.insert("std_dev".to_string(), std_dev);
            stats.insert("min".to_string(), all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
            stats.insert("max".to_string(), all_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
            stats.insert("total_records".to_string(), records.len() as f64);
            stats.insert("total_values".to_string(), total_values as f64);
        }

        stats
    }
}

pub fn process_records(
    processor: &DataProcessor,
    records: &mut [DataRecord],
) -> Result<HashMap<String, f64>, ProcessingError> {
    let mut valid_records = Vec::new();
    
    for record in records.iter_mut() {
        if let Err(e) = processor.validate_record(record) {
            eprintln!("Validation failed for record {}: {}", record.id, e);
            continue;
        }

        if let Err(e) = processor.transform_values(record) {
            eprintln!("Transformation failed for record {}: {}", record.id, e);
            continue;
        }

        valid_records.push(record.clone());
    }

    if valid_records.is_empty() {
        return Err(ProcessingError::ProcessingError(
            "No valid records after processing".to_string(),
        ));
    }

    Ok(processor.calculate_statistics(&valid_records))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let record = DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_threshold_exceeded() {
        let processor = DataProcessor::new(10.0, 2.0);
        let record = DataRecord {
            id: 1,
            values: vec![5.0, 15.0, 25.0],
            metadata: HashMap::new(),
        };
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transformation() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let mut record = DataRecord {
            id: 1,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(processor.transform_values(&mut record).is_ok());
        assert_eq!(record.values.len(), 3);
        assert!(record.metadata.contains_key("transformation_applied"));
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(1000.0, 1.0);
        let records = vec![
            DataRecord {
                id: 1,
                values: vec![1.0, 2.0, 3.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                values: vec![4.0, 5.0, 6.0],
                metadata: HashMap::new(),
            },
        ];
        
        let stats = processor.calculate_statistics(&records);
        assert_eq!(stats["mean"], 3.5);
        assert_eq!(stats["total_records"], 2.0);
        assert_eq!(stats["total_values"], 6.0);
    }
}