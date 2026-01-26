use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<Box<dyn Fn(&[f64]) -> bool>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: vec![
                Box::new(|data: &[f64]| !data.is_empty()),
                Box::new(|data: &[f64]| data.iter().all(|&x| x.is_finite())),
                Box::new(|data: &[f64]| data.len() < 1000),
            ],
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if !self.validate_data(&data) {
            return Err("Data validation failed".to_string());
        }

        let processed = self.transform_data(data);
        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    fn validate_data(&self, data: &[f64]) -> bool {
        self.validation_rules.iter().all(|rule| rule(data))
    }

    fn transform_data(&self, mut data: Vec<f64>) -> Vec<f64> {
        if data.len() > 1 {
            let mean = data.iter().sum::<f64>() / data.len() as f64;
            data.iter_mut().for_each(|x| *x = (*x - mean).abs());
        }
        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        data
    }

    pub fn get_cached_result(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![3.5, 1.2, 4.8, 2.1];
        
        let result = processor.process_dataset("test", data.clone());
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 4);
        assert!(processed.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![f64::INFINITY, 2.0];
        
        let result = processor.process_dataset("invalid", data);
        assert!(result.is_err());
    }
}