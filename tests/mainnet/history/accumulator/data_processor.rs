use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataSet {
    values: Vec<f64>,
}

impl DataSet {
    pub fn new() -> Self {
        DataSet { values: Vec::new() }
    }

    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut values = Vec::new();

        for result in rdr.records() {
            let record = result?;
            if let Some(field) = record.get(0) {
                if let Ok(num) = field.parse::<f64>() {
                    values.push(num);
                }
            }
        }

        Ok(DataSet { values })
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn calculate_std_dev(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        let mean = self.calculate_mean()?;
        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.values.len() - 1) as f64;
        Some(variance.sqrt())
    }

    pub fn get_summary(&self) -> DataSummary {
        DataSummary {
            count: self.values.len(),
            mean: self.calculate_mean(),
            std_dev: self.calculate_std_dev(),
            min: self.values.iter().copied().reduce(f64::min),
            max: self.values.iter().copied().reduce(f64::max),
        }
    }
}

pub struct DataSummary {
    pub count: usize,
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl std::fmt::Display for DataSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Data Summary:")?;
        writeln!(f, "  Count: {}", self.count)?;
        writeln!(f, "  Mean: {:.4}", self.mean.unwrap_or(f64::NAN))?;
        writeln!(f, "  Std Dev: {:.4}", self.std_dev.unwrap_or(f64::NAN))?;
        writeln!(f, "  Min: {:.4}", self.min.unwrap_or(f64::NAN))?;
        write!(f, "  Max: {:.4}", self.max.unwrap_or(f64::NAN))
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    config: ProcessingConfig,
}

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub max_values: usize,
    pub require_timestamp: bool,
    pub allowed_metadata_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.config.max_values {
            return Err(ProcessingError::ValidationFailed(
                format!("Too many values: {}", record.values.len())
            ));
        }

        if self.config.require_timestamp && record.timestamp <= 0 {
            return Err(ProcessingError::ValidationFailed(
                "Invalid timestamp".to_string()
            ));
        }

        for key in record.metadata.keys() {
            if !self.config.allowed_metadata_keys.contains(key) {
                return Err(ProcessingError::ValidationFailed(
                    format!("Disallowed metadata key: {}", key)
                ));
            }
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        transformed.values = record.values
            .iter()
            .map(|&v| v * 2.0)
            .collect();

        transformed.metadata.insert(
            "processed".to_string(),
            "true".to_string()
        );

        transformed.metadata.insert(
            "transformation_timestamp".to_string(),
            chrono::Utc::now().timestamp().to_string()
        );

        Ok(transformed)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());

        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record)?;
            processed_records.push(transformed);
        }

        Ok(processed_records)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        let all_values: Vec<f64> = records.iter()
            .flat_map(|r| r.values.iter().copied())
            .collect();

        if !all_values.is_empty() {
            let sum: f64 = all_values.iter().sum();
            let count = all_values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = all_values.iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>() / count;

            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("total_records".to_string(), records.len() as f64);
            stats.insert("total_values".to_string(), total_values as f64);
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::from([
                ("source".to_string(), "test".to_string()),
                ("version".to_string(), "1.0".to_string())
            ]),
        }
    }

    fn create_test_config() -> ProcessingConfig {
        ProcessingConfig {
            max_values: 10,
            require_timestamp: true,
            allowed_metadata_keys: vec![
                "source".to_string(),
                "version".to_string(),
                "processed".to_string(),
                "transformation_timestamp".to_string()
            ],
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(create_test_config());
        let record = create_test_record();
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_too_many_values() {
        let config = ProcessingConfig {
            max_values: 2,
            ..create_test_config()
        };
        let processor = DataProcessor::new(config);
        let record = create_test_record();
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(create_test_config());
        let record = create_test_record();
        let transformed = processor.transform_record(&record).unwrap();
        
        assert_eq!(transformed.values, vec![2.0, 4.0, 6.0]);
        assert_eq!(transformed.metadata.get("processed"), Some(&"true".to_string()));
    }

    #[test]
    fn test_process_batch() {
        let processor = DataProcessor::new(create_test_config());
        let records = vec![create_test_record(), create_test_record()];
        
        let processed = processor.process_batch(records).unwrap();
        assert_eq!(processed.len(), 2);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(create_test_config());
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: vec![1.0, 2.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: vec![3.0, 4.0],
                metadata: HashMap::new(),
            },
        ];
        
        let stats = processor.calculate_statistics(&records);
        
        assert_eq!(stats.get("mean"), Some(&2.5));
        assert_eq!(stats.get("total_records"), Some(&2.0));
        assert_eq!(stats.get("total_values"), Some(&4.0));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if let Some(value_str) = parts.get(0) {
                if let Ok(value) = value_str.parse::<f64>() {
                    self.data.push(value);
                }
            }
            
            if let Some(category) = parts.get(1) {
                *self.frequency_map.entry(category.to_string()).or_insert(0) += 1;
            }
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_median(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mid = sorted_data.len() / 2;
        if sorted_data.len() % 2 == 0 {
            Some((sorted_data[mid - 1] + sorted_data[mid]) / 2.0)
        } else {
            Some(sorted_data[mid])
        }
    }

    pub fn get_frequency_distribution(&self) -> &HashMap<String, u32> {
        &self.frequency_map
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
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
        writeln!(temp_file, "10.5,category_a").unwrap();
        writeln!(temp_file, "20.3,category_b").unwrap();
        writeln!(temp_file, "15.7,category_a").unwrap();
        writeln!(temp_file, "25.1,category_c").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.calculate_mean(), Some(17.9));
        assert_eq!(processor.calculate_median(), Some(17.9));
        
        let freq = processor.get_frequency_distribution();
        assert_eq!(freq.get("category_a"), Some(&2));
        assert_eq!(freq.get("category_b"), Some(&1));
        
        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 3);
    }
}