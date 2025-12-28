
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let value: f64 = line.trim().parse()?;
            self.data.push(value);
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

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }

        let mean = self.calculate_mean()?;
        let variance: f64 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;

        Some(variance.sqrt())
    }

    pub fn get_min_max(&self) -> Option<(f64, f64)> {
        if self.data.is_empty() {
            return None;
        }

        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Some((min, max))
    }

    pub fn filter_outliers(&self, threshold: f64) -> Vec<f64> {
        if self.data.len() < 2 {
            return self.data.clone();
        }

        let mean = match self.calculate_mean() {
            Some(m) => m,
            None => return self.data.clone(),
        };

        let std_dev = match self.calculate_standard_deviation() {
            Some(s) => s,
            None => return self.data.clone(),
        };

        self.data
            .iter()
            .filter(|&&x| (x - mean).abs() <= threshold * std_dev)
            .cloned()
            .collect()
    }

    pub fn data_count(&self) -> usize {
        self.data.len()
    }

    pub fn clear_data(&mut self) {
        self.data.clear();
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
        writeln!(temp_file, "10.5\n20.3\n15.7\n25.1\n18.9").unwrap();
        
        assert!(processor.load_from_csv(temp_file.path()).is_ok());
        assert_eq!(processor.data_count(), 5);
        
        let mean = processor.calculate_mean().unwrap();
        assert!((mean - 18.1).abs() < 0.01);
        
        let (min, max) = processor.get_min_max().unwrap();
        assert_eq!(min, 10.5);
        assert_eq!(max, 25.1);
        
        let filtered = processor.filter_outliers(2.0);
        assert_eq!(filtered.len(), 5);
    }
}
use std::collections::HashMap;
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

impl ValidationError {
    fn new(message: &str) -> Self {
        ValidationError {
            message: message.to_string(),
        }
    }
}

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, key: &str, values: Vec<f64>) -> Result<(), ValidationError> {
        if values.is_empty() {
            return Err(ValidationError::new("Dataset cannot be empty"));
        }

        for value in &values {
            if value.is_nan() || value.is_infinite() {
                return Err(ValidationError::new("Dataset contains invalid numeric values"));
            }
        }

        self.data.insert(key.to_string(), values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Result<Statistics, ValidationError> {
        let values = self.data.get(key)
            .ok_or_else(|| ValidationError::new("Dataset not found"))?;

        if values.is_empty() {
            return Err(ValidationError::new("Dataset is empty"));
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let sorted_values = {
            let mut sorted = values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted
        };

        let median = if count as usize % 2 == 0 {
            let mid = count as usize / 2;
            (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
        } else {
            sorted_values[count as usize / 2]
        };

        Ok(Statistics {
            count: values.len(),
            mean,
            variance,
            median,
            min: *sorted_values.first().unwrap(),
            max: *sorted_values.last().unwrap(),
        })
    }

    pub fn normalize_data(&self, key: &str) -> Result<Vec<f64>, ValidationError> {
        let values = self.data.get(key)
            .ok_or_else(|| ValidationError::new("Dataset not found"))?;

        let stats = self.calculate_statistics(key)?;
        
        if stats.variance == 0.0 {
            return Ok(vec![0.0; values.len()]);
        }

        let normalized: Vec<f64> = values.iter()
            .map(|&x| (x - stats.mean) / stats.variance.sqrt())
            .collect();

        Ok(normalized)
    }

    pub fn merge_datasets(&self, keys: &[&str]) -> Result<Vec<f64>, ValidationError> {
        if keys.is_empty() {
            return Err(ValidationError::new("No datasets specified for merging"));
        }

        let mut merged = Vec::new();
        for key in keys {
            let values = self.data.get(*key)
                .ok_or_else(|| ValidationError::new(&format!("Dataset '{}' not found", key)))?;
            merged.extend(values);
        }

        Ok(merged)
    }

    pub fn list_datasets(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Statistics: count={}, mean={:.4}, variance={:.4}, median={:.4}, min={:.4}, max={:.4}",
            self.count, self.mean, self.variance, self.median, self.min, self.max
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_calculate_statistics() {
        let mut processor = DataProcessor::new();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        processor.add_dataset("test", values).unwrap();
        let stats = processor.calculate_statistics("test").unwrap();
        
        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_normalize_data() {
        let mut processor = DataProcessor::new();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        processor.add_dataset("test", values).unwrap();
        let normalized = processor.normalize_data("test").unwrap();
        
        assert_eq!(normalized.len(), 5);
        let sum: f64 = normalized.iter().sum();
        assert!(sum.abs() < 1e-10);
    }

    #[test]
    fn test_merge_datasets() {
        let mut processor = DataProcessor::new();
        
        processor.add_dataset("set1", vec![1.0, 2.0]).unwrap();
        processor.add_dataset("set2", vec![3.0, 4.0]).unwrap();
        
        let merged = processor.merge_datasets(&["set1", "set2"]).unwrap();
        assert_eq!(merged, vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_validation_error() {
        let mut processor = DataProcessor::new();
        
        let result = processor.add_dataset("empty", vec![]);
        assert!(result.is_err());
        
        let result = processor.calculate_statistics("nonexistent");
        assert!(result.is_err());
    }
}