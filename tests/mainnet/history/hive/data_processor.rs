use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }
        Ok(Self { threshold })
    }

    pub fn process_data(&self, data: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if data.is_empty() {
            return Err(ValidationError {
                message: "Input data cannot be empty".to_string(),
            });
        }

        let filtered_data: Vec<f64> = data
            .iter()
            .filter(|&&value| value >= self.threshold)
            .cloned()
            .collect();

        if filtered_data.is_empty() {
            return Err(ValidationError {
                message: format!(
                    "No data points above threshold {} found",
                    self.threshold
                ),
            });
        }

        let mean = filtered_data.iter().sum::<f64>() / filtered_data.len() as f64;
        let processed: Vec<f64> = filtered_data.iter().map(|&x| x / mean).collect();

        Ok(processed)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> Result<(f64, f64, f64), ValidationError> {
        if data.is_empty() {
            return Err(ValidationError {
                message: "Cannot calculate statistics for empty dataset".to_string(),
            });
        }

        let sum: f64 = data.iter().sum();
        let count = data.len() as f64;
        let mean = sum / count;

        let variance: f64 = data
            .iter()
            .map(|&value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(1.5);
        assert!(processor.is_err());
    }

    #[test]
    fn test_data_processing() {
        let processor = DataProcessor::new(0.3).unwrap();
        let data = vec![0.1, 0.4, 0.5, 0.2, 0.6];
        let result = processor.process_data(&data);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 3);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(0.0).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = processor.calculate_statistics(&data).unwrap();
        assert!((stats.0 - 3.0).abs() < 0.0001);
        assert!((stats.1 - 2.0).abs() < 0.0001);
        assert!((stats.2 - 1.41421356).abs() < 0.0001);
    }
}
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
pub enum ValidationError {
    InvalidId,
    EmptyValues,
    ValueOutOfRange(f64, f64, f64),
    MissingMetadata(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "Record ID must be greater than zero"),
            ValidationError::EmptyValues => write!(f, "Record must contain at least one value"),
            ValidationError::ValueOutOfRange(val, min, max) => 
                write!(f, "Value {} is outside allowed range [{}, {}]", val, min, max),
            ValidationError::MissingMetadata(key) => 
                write!(f, "Required metadata field '{}' is missing", key),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
    required_metadata: Vec<String>,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64, required_metadata: Vec<String>) -> Self {
        DataProcessor {
            min_value,
            max_value,
            required_metadata,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }

        if record.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }

        for &value in &record.values {
            if value < self.min_value || value > self.max_value {
                return Err(ValidationError::ValueOutOfRange(value, self.min_value, self.max_value));
            }
        }

        for key in &self.required_metadata {
            if !record.metadata.contains_key(key) {
                return Err(ValidationError::MissingMetadata(key.clone()));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) {
        if let Some(max_val) = record.values.iter().copied().reduce(f64::max) {
            if max_val != 0.0 {
                for value in &mut record.values {
                    *value /= max_val;
                }
            }
        }
    }

    pub fn process_records(&self, records: &mut [DataRecord]) -> Vec<Result<DataRecord, ValidationError>> {
        let mut results = Vec::new();

        for record in records {
            match self.validate_record(record) {
                Ok(_) => {
                    let mut processed_record = record.clone();
                    self.normalize_values(&mut processed_record);
                    results.push(Ok(processed_record));
                }
                Err(e) => {
                    results.push(Err(e));
                }
            }
        }

        results
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> Option<HashMap<String, f64>> {
        if records.is_empty() {
            return None;
        }

        let value_count = records[0].values.len();
        let mut sums = vec![0.0; value_count];
        let mut counts = vec![0; value_count];

        for record in records {
            for (i, &value) in record.values.iter().enumerate() {
                sums[i] += value;
                counts[i] += 1;
            }
        }

        let mut stats = HashMap::new();
        for i in 0..value_count {
            if counts[i] > 0 {
                let avg = sums[i] / counts[i] as f64;
                stats.insert(format!("value_{}_average", i), avg);
            }
        }

        Some(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        metadata.insert("timestamp".to_string(), "2024-01-01".to_string());

        DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata,
        }
    }

    #[test]
    fn test_valid_record_validation() {
        let processor = DataProcessor::new(
            0.0,
            100.0,
            vec!["source".to_string(), "timestamp".to_string()]
        );
        let record = create_test_record();
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id_validation() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        let mut record = create_test_record();
        record.id = 0;
        
        assert!(matches!(
            processor.validate_record(&record),
            Err(ValidationError::InvalidId)
        ));
    }

    #[test]
    fn test_normalize_values() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        let mut record = create_test_record();
        
        processor.normalize_values(&mut record);
        
        let expected = vec![10.0/30.0, 20.0/30.0, 30.0/30.0];
        assert_eq!(record.values, expected);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        let records = vec![
            DataRecord {
                id: 1,
                values: vec![10.0, 20.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                values: vec![30.0, 40.0],
                metadata: HashMap::new(),
            },
        ];
        
        let stats = processor.calculate_statistics(&records).unwrap();
        
        assert_eq!(stats.get("value_0_average"), Some(&20.0));
        assert_eq!(stats.get("value_1_average"), Some(&30.0));
    }
}