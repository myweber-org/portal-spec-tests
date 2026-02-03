
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

        let processed = Self::normalize_data(data)?;
        self.cache.insert(key.to_string(), processed.clone());
        
        Ok(processed)
    }

    fn normalize_data(data: &[f64]) -> Result<Vec<f64>, String> {
        let max_value = data.iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if max_value <= 0.0 {
            return Err("Invalid data range for normalization".to_string());
        }

        let normalized: Vec<f64> = data.iter()
            .map(|&x| x / max_value)
            .collect();

        Ok(normalized)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> (f64, f64, f64) {
        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;
        
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let processor = DataProcessor::new();
        
        let result = DataProcessor::normalize_data(&data).unwrap();
        assert_eq!(result, vec![0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("test", &[]);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let data = vec![2.0, 4.0, 6.0, 8.0];
        let processor = DataProcessor::new();
        
        let (mean, variance, std_dev) = processor.calculate_statistics(&data);
        
        assert_eq!(mean, 5.0);
        assert_eq!(variance, 5.0);
        assert_eq!(std_dev, 5.0f64.sqrt());
    }
}