
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