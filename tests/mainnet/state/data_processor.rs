
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

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated_data = self.validate_data(data)?;
        let normalized_data = self.normalize_data(&validated_data);
        let transformed_data = self.apply_transformations(&normalized_data);

        self.cache.insert(key.to_string(), transformed_data.clone());
        Ok(transformed_data)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        let mut valid_data = Vec::new();
        
        for &value in data {
            if value.is_finite() {
                valid_data.push(value);
            } else {
                return Err(format!("Invalid data point detected: {}", value));
            }
        }

        if valid_data.len() < 2 {
            return Err("Insufficient valid data points".to_string());
        }

        Ok(valid_data)
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let range = max - min;

        if range.abs() < f64::EPSILON {
            return vec![0.5; data.len()];
        }

        data.iter()
            .map(|&x| (x - min) / range)
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev.abs() < f64::EPSILON {
            return data.to_vec();
        }

        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_items = self.cache.len();
        let total_values = self.cache.values()
            .map(|v| v.len())
            .sum();
        (total_items, total_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let test_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_dataset("test", &test_data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 5);
        
        let stats = processor.get_cache_stats();
        assert_eq!(stats.0, 1);
        assert_eq!(stats.1, 5);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let invalid_data = vec![1.0, f64::NAN, 3.0];
        
        let result = processor.process_dataset("invalid", &invalid_data);
        assert!(result.is_err());
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

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(data)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        let mut valid_data = Vec::new();
        
        for &value in data {
            if value.is_finite() {
                valid_data.push(value);
            } else {
                return Err("Invalid numeric value detected".to_string());
            }
        }

        if valid_data.len() < 2 {
            return Err("Insufficient valid data points".to_string());
        }

        Ok(valid_data)
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let range = max - min;

        if range.abs() < f64::EPSILON {
            return vec![0.5; data.len()];
        }

        data.iter()
            .map(|&x| (x - min) / range)
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.sqrt().abs())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let total_keys = self.cache.len();
        let total_values: usize = self.cache.values().map(|v| v.len()).sum();
        (total_keys, total_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_validation() {
        let processor = DataProcessor::new();
        let valid_data = vec![1.0, 2.0, 3.0];
        let result = processor.validate_data(&valid_data);
        assert!(result.is_ok());

        let invalid_data = vec![1.0, f64::NAN, 3.0];
        let result = processor.validate_data(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new();
        let data = vec![0.0, 5.0, 10.0];
        let normalized = processor.normalize_data(&data);
        
        assert_eq!(normalized.len(), 3);
        assert!((normalized[0] - 0.0).abs() < 0.001);
        assert!((normalized[1] - 0.5).abs() < 0.001);
        assert!((normalized[2] - 1.0).abs() < 0.001);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataPoint {
    timestamp: i64,
    value: f64,
    category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidTimestamp,
    InvalidValue,
    EmptyCategory,
    TransformationFailed,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidTimestamp => write!(f, "Timestamp must be positive"),
            ProcessingError::InvalidValue => write!(f, "Value must be within valid range"),
            ProcessingError::EmptyCategory => write!(f, "Category cannot be empty"),
            ProcessingError::TransformationFailed => write!(f, "Data transformation failed"),
        }
    }
}

impl Error for ProcessingError {}

impl DataPoint {
    pub fn new(timestamp: i64, value: f64, category: String) -> Result<Self, ProcessingError> {
        if timestamp <= 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }
        if !value.is_finite() {
            return Err(ProcessingError::InvalidValue);
        }
        if category.trim().is_empty() {
            return Err(ProcessingError::EmptyCategory);
        }

        Ok(Self {
            timestamp,
            value,
            category,
        })
    }

    pub fn transform(&self, multiplier: f64) -> Result<Self, ProcessingError> {
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(ProcessingError::TransformationFailed);
        }

        let transformed_value = self.value * multiplier;
        
        Ok(Self {
            timestamp: self.timestamp,
            value: transformed_value,
            category: self.category.clone(),
        })
    }

    pub fn normalize(&self, min: f64, max: f64) -> Result<f64, ProcessingError> {
        if min >= max || !min.is_finite() || !max.is_finite() {
            return Err(ProcessingError::TransformationFailed);
        }

        let normalized = (self.value - min) / (max - min);
        if normalized.is_finite() {
            Ok(normalized)
        } else {
            Err(ProcessingError::TransformationFailed)
        }
    }
}

pub fn process_dataset(points: Vec<DataPoint>) -> Result<Vec<DataPoint>, ProcessingError> {
    if points.is_empty() {
        return Err(ProcessingError::TransformationFailed);
    }

    let mut processed = Vec::with_capacity(points.len());
    for point in points {
        let transformed = point.transform(2.0)?;
        processed.push(transformed);
    }

    Ok(processed)
}

pub fn calculate_statistics(points: &[DataPoint]) -> Result<(f64, f64), ProcessingError> {
    if points.is_empty() {
        return Err(ProcessingError::TransformationFailed);
    }

    let sum: f64 = points.iter().map(|p| p.value).sum();
    let count = points.len() as f64;
    let mean = sum / count;

    let variance: f64 = points.iter()
        .map(|p| (p.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();

    if mean.is_finite() && std_dev.is_finite() {
        Ok((mean, std_dev))
    } else {
        Err(ProcessingError::TransformationFailed)
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let active_count = records.iter().filter(|r| r.active).count();
    
    (sum, average, active_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,20.0,false").unwrap();
        
        let result = process_data_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];
        
        let (sum, avg, active_count) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(avg, 20.0);
        assert_eq!(active_count, 2);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

        Ok(DataProcessor { threshold })
    }

    pub fn process_data(&self, input: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError {
                message: "Input data cannot be empty".to_string(),
            });
        }

        let normalized: Vec<f64> = input
            .iter()
            .map(|&value| {
                if value.is_nan() || value.is_infinite() {
                    0.0
                } else {
                    value
                }
            })
            .collect();

        let max_value = normalized
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        if max_value <= 0.0 {
            return Err(ValidationError {
                message: "All values must be positive for processing".to_string(),
            });
        }

        let processed: Vec<f64> = normalized
            .iter()
            .map(|&value| {
                let normalized_value = value / max_value;
                if normalized_value >= self.threshold {
                    normalized_value
                } else {
                    0.0
                }
            })
            .collect();

        Ok(processed)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> Result<(f64, f64, f64), ValidationError> {
        if data.len() < 2 {
            return Err(ValidationError {
                message: "Insufficient data for statistics calculation".to_string(),
            });
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;

        let variance: f64 = data
            .iter()
            .map(|value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>()
            / (data.len() - 1) as f64;

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
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let result = processor.process_data(&input);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 4);
        assert!(processed[3] > 0.0);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(0.5).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = processor.calculate_statistics(&data);
        
        assert!(stats.is_ok());
        let (mean, variance, std_dev) = stats.unwrap();
        assert!((mean - 3.0).abs() < 0.0001);
        assert!(variance > 0.0);
        assert!(std_dev > 0.0);
    }
}