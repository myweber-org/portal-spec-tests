
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
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

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_unique_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.records
            .iter()
            .map(|record| record.category.clone())
            .collect();

        categories.sort();
        categories.dedup();
        categories
    }

    pub fn validate_records(&self) -> Vec<u32> {
        self.records
            .iter()
            .filter(|record| record.value.is_nan() || record.value.is_infinite())
            .map(|record| record.id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,42.5,TypeA").unwrap();
        writeln!(temp_file, "2,18.3,TypeB").unwrap();
        writeln!(temp_file, "3,75.0,TypeA").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);

        let filtered = processor.filter_by_threshold(20.0);
        assert_eq!(filtered.len(), 2);

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 45.266).abs() < 0.001);

        let categories = processor.get_unique_categories();
        assert_eq!(categories, vec!["TypeA", "TypeB"]);
    }
}use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    ValidationFailed(String),
    TransformationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_threshold: f64,
    max_records: usize,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, max_records: usize) -> Self {
        DataProcessor {
            validation_threshold,
            max_records,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::InvalidData("Empty values vector".to_string()));
        }

        if record.values.len() > self.max_records {
            return Err(ProcessingError::ValidationFailed(
                format!("Exceeds maximum record size: {}", self.max_records)
            ));
        }

        for &value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::InvalidData(
                    "Invalid numeric value detected".to_string()
                ));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::TransformationError(
                "Cannot normalize empty values".to_string()
            ));
        }

        let max_value = record.values
            .iter()
            .fold(f64::MIN, |a, &b| a.max(b));
        
        if max_value.abs() < f64::EPSILON {
            return Err(ProcessingError::TransformationError(
                "All values are zero, cannot normalize".to_string()
            ));
        }

        for value in &mut record.values {
            *value /= max_value;
        }

        Ok(())
    }

    pub fn filter_outliers(&self, records: &mut Vec<DataRecord>) -> Vec<DataRecord> {
        let mut filtered = Vec::new();
        
        for record in records.drain(..) {
            let avg: f64 = record.values.iter().sum::<f64>() / record.values.len() as f64;
            let variance: f64 = record.values.iter()
                .map(|&v| (v - avg).powi(2))
                .sum::<f64>() / record.values.len() as f64;
            let std_dev = variance.sqrt();

            let mut has_outlier = false;
            for &value in &record.values {
                if (value - avg).abs() > self.validation_threshold * std_dev {
                    has_outlier = true;
                    break;
                }
            }

            if !has_outlier {
                filtered.push(record);
            }
        }

        filtered
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let all_values: Vec<f64> = records.iter()
            .flat_map(|r| r.values.clone())
            .collect();

        let count = all_values.len() as f64;
        let sum: f64 = all_values.iter().sum();
        let mean = sum / count;

        let variance: f64 = all_values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());
        stats.insert("min".to_string(), *all_values.iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(&0.0));
        stats.insert("max".to_string(), *all_values.iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(&0.0));
        stats.insert("total_records".to_string(), records.len() as f64);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(3.0, 100);
        let mut record = DataRecord {
            id: 1,
            values: vec![1.0, 2.0, 3.0],
            timestamp: 1234567890,
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());

        record.values = vec![];
        assert!(processor.validate_record(&record).is_err());

        record.values = vec![f64::NAN];
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_normalize_values() {
        let processor = DataProcessor::new(3.0, 100);
        let mut record = DataRecord {
            id: 1,
            values: vec![2.0, 4.0, 6.0],
            timestamp: 1234567890,
            metadata: HashMap::new(),
        };

        assert!(processor.normalize_values(&mut record).is_ok());
        assert_eq!(record.values, vec![1.0/3.0, 2.0/3.0, 1.0]);
    }

    #[test]
    fn test_filter_outliers() {
        let processor = DataProcessor::new(2.0, 100);
        let mut records = vec![
            DataRecord {
                id: 1,
                values: vec![1.0, 2.0, 3.0],
                timestamp: 1234567890,
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                values: vec![100.0, 200.0, 300.0],
                timestamp: 1234567891,
                metadata: HashMap::new(),
            },
        ];

        let filtered = processor.filter_outliers(&mut records);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_data(&mut self, key: &str, values: &[f64]) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data array provided".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Invalid numeric values detected".to_string());
        }

        let processed: Vec<f64> = values
            .iter()
            .map(|&x| x * 2.0)
            .collect();

        self.cache.insert(key.to_string(), processed.clone());

        Ok(processed)
    }

    pub fn get_cached_result(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<(f64, f64, f64)> {
        self.cache.get(key).map(|values| {
            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = values
                .iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();
            
            (mean, variance, std_dev)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_data() {
        let mut processor = DataProcessor::new();
        let result = processor.process_data("test", &[1.0, 2.0, 3.0]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_empty_data() {
        let mut processor = DataProcessor::new();
        let result = processor.process_data("empty", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cached_results() {
        let mut processor = DataProcessor::new();
        processor.process_data("cached", &[5.0, 10.0]).unwrap();
        
        assert!(processor.get_cached_result("cached").is_some());
        assert!(processor.get_cached_result("missing").is_none());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
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
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();

            if value < 0.0 {
                return Err(format!("Negative value at line {}", line_num + 1).into());
            }

            self.records.push(DataRecord {
                id,
                value,
                category,
            });
        }

        Ok(())
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.3,beta").unwrap();
        writeln!(temp_file, "3,15.7,alpha").unwrap();

        let mut processor = DataProcessor::new();
        processor
            .load_from_csv(temp_file.path().to_str().unwrap())
            .unwrap();

        assert_eq!(processor.total_records(), 3);
        assert!((processor.calculate_average() - 15.5).abs() < 0.01);

        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
    }
}