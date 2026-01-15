
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn validate_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value.is_finite() && !r.category.is_empty())
            .collect()
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.3,beta").unwrap();
        writeln!(temp_file, "3,15.7,alpha").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.5).abs() < 0.1);
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
        
        let stats = processor.get_statistics();
        assert!((stats.0 - 10.5).abs() < 0.1);
        assert!((stats.1 - 20.3).abs() < 0.1);
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
pub enum DataError {
    ValidationFailed(String),
    TransformationError(String),
    InvalidFormat(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            DataError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            DataError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    validation_threshold: f64,
    normalization_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, normalization_factor: f64) -> Self {
        DataProcessor {
            validation_threshold,
            normalization_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.values.is_empty() {
            return Err(DataError::ValidationFailed("Empty values vector".to_string()));
        }

        for (i, &value) in record.values.iter().enumerate() {
            if value.is_nan() || value.is_infinite() {
                return Err(DataError::ValidationFailed(
                    format!("Invalid value at index {}: {}", i, value)
                ));
            }

            if value.abs() > self.validation_threshold {
                return Err(DataError::ValidationFailed(
                    format!("Value {} exceeds threshold at index {}", value, i)
                ));
            }
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord) -> Result<(), DataError> {
        self.validate_record(record)?;

        for value in record.values.iter_mut() {
            *value = (*value * self.normalization_factor).tanh();
            
            if value.is_nan() || value.is_infinite() {
                return Err(DataError::TransformationError(
                    "Numerical overflow during transformation".to_string()
                ));
            }
        }

        record.metadata.insert(
            "processed".to_string(),
            "true".to_string()
        );
        record.metadata.insert(
            "normalization_factor".to_string(),
            self.normalization_factor.to_string()
        );

        Ok(())
    }

    pub fn batch_process(&self, records: &mut [DataRecord]) -> Vec<Result<(), DataError>> {
        records.iter_mut()
            .map(|record| self.transform_values(record))
            .collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> Result<HashMap<String, f64>, DataError> {
        if records.is_empty() {
            return Err(DataError::InvalidFormat("No records provided".to_string()));
        }

        let mut stats = HashMap::new();
        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        
        if total_values == 0 {
            return Err(DataError::InvalidFormat("All records have empty values".to_string()));
        }

        let sum: f64 = records.iter()
            .flat_map(|r| r.values.iter())
            .sum();
        
        let mean = sum / total_values as f64;
        
        let variance: f64 = records.iter()
            .flat_map(|r| r.values.iter())
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / total_values as f64;

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        DataRecord {
            id: 1,
            values: vec![0.5, 1.0, -0.5],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(10.0, 1.0);
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(0.1, 1.0);
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transformation() {
        let processor = DataProcessor::new(10.0, 2.0);
        let mut record = create_test_record();
        
        assert!(processor.transform_values(&mut record).is_ok());
        assert_eq!(record.metadata.get("processed"), Some(&"true".to_string()));
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(10.0, 1.0);
        let mut records = vec![create_test_record(), create_test_record()];
        
        let results = processor.batch_process(&mut records);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(10.0, 1.0);
        let records = vec![create_test_record(), create_test_record()];
        
        let stats = processor.calculate_statistics(&records);
        assert!(stats.is_ok());
        
        let stats_map = stats.unwrap();
        assert!(stats_map.contains_key("mean"));
        assert!(stats_map.contains_key("variance"));
    }
}