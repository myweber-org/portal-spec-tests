
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, key: &str, values: Vec<f64>) -> Result<(), String> {
        if values.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Dataset contains invalid numeric values".to_string());
        }

        self.data.insert(key.to_string(), values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            
            let std_dev = variance.sqrt();

            Statistics {
                count,
                mean,
                variance,
                std_dev,
                min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            }
        })
    }

    pub fn normalize_data(&self, key: &str) -> Option<Vec<f64>> {
        self.calculate_statistics(key).map(|stats| {
            self.data[key].iter()
                .map(|&x| (x - stats.min) / (stats.max - stats.min))
                .collect()
        })
    }

    pub fn merge_datasets(&mut self, target_key: &str, source_keys: &[&str]) -> Result<(), String> {
        let mut merged_data = Vec::new();
        
        for &key in source_keys {
            if let Some(data) = self.data.get(key) {
                merged_data.extend(data.clone());
            } else {
                return Err(format!("Dataset '{}' not found", key));
            }
        }

        if merged_data.is_empty() {
            return Err("No valid data to merge".to_string());
        }

        self.add_dataset(target_key, merged_data)
    }

    pub fn list_datasets(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Count: {}, Mean: {:.4}, StdDev: {:.4}, Min: {:.4}, Max: {:.4}",
            self.count, self.mean, self.std_dev, self.min, self.max
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        assert!(processor.add_dataset("temperatures", vec![20.5, 22.1, 19.8, 21.3, 23.0]).is_ok());
        assert!(processor.add_dataset("empty", vec![]).is_err());
        
        let stats = processor.calculate_statistics("temperatures").unwrap();
        assert_eq!(stats.count, 5);
        assert!(stats.mean > 20.0 && stats.mean < 22.0);
        
        let normalized = processor.normalize_data("temperatures").unwrap();
        assert_eq!(normalized.len(), 5);
        assert!(normalized.iter().all(|&x| x >= 0.0 && x <= 1.0));
        
        assert!(processor.merge_datasets("combined", &["temperatures"]).is_ok());
        assert!(processor.list_datasets().contains(&"combined".to_string()));
    }
}