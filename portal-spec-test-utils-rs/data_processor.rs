
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
use std::collections::HashMap;

pub struct DataProcessor {
    filters: Vec<Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            filters: Vec::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn add_transformer<N, T>(&mut self, name: N, transformer: T)
    where
        N: Into<String>,
        T: Fn(String) -> String + 'static,
    {
        self.transformers.insert(name.into(), Box::new(transformer));
    }

    pub fn process(&self, input: &str) -> Option<String> {
        if !self.filters.iter().all(|f| f(input)) {
            return None;
        }

        let mut result = input.to_string();
        for transformer in self.transformers.values() {
            result = transformer(result);
        }

        Some(result)
    }

    pub fn batch_process(&self, inputs: Vec<&str>) -> Vec<String> {
        inputs
            .iter()
            .filter_map(|&input| self.process(input))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_filter(|s| s.len() > 3);
        processor.add_transformer("uppercase", |s| s.to_uppercase());
        processor.add_transformer("trim", |s| s.trim().to_string());

        let result = processor.process("  test data  ");
        assert_eq!(result, Some("TEST DATA".to_string()));

        let filtered = processor.process("abc");
        assert_eq!(filtered, None);

        let batch_results = processor.batch_process(vec!["  one  ", "two", "  three  "]);
        assert_eq!(batch_results, vec!["ONE", "THREE"]);
    }
}