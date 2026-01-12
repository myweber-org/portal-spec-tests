use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    id: u64,
    timestamp: i64,
    values: HashMap<String, f64>,
    tags: Vec<String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn add_value(&mut self, key: String, value: f64) -> Result<(), String> {
        if value.is_nan() || value.is_infinite() {
            return Err(format!("Invalid value for key '{}': {}", key, value));
        }
        self.values.insert(key, value);
        Ok(())
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.id == 0 {
            errors.push("ID cannot be zero".to_string());
        }

        if self.timestamp < 0 {
            errors.push("Timestamp cannot be negative".to_string());
        }

        if self.values.is_empty() {
            errors.push("Record must contain at least one value".to_string());
        }

        for (key, value) in &self.values {
            if value.is_nan() || value.is_infinite() {
                errors.push(format!("Invalid value for key '{}': {}", key, value));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn transform(&self, multiplier: f64) -> Self {
        let mut transformed = self.clone();
        
        for value in transformed.values.values_mut() {
            *value *= multiplier;
        }

        transformed.tags.push("transformed".to_string());
        transformed
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|record| record.validate().is_ok())
        .map(|record| record.transform(2.0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_record_validation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("temperature".to_string(), 25.5).unwrap();
        
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, -1);
        let result = record.validate();
        
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors.len() >= 2);
        }
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("pressure".to_string(), 100.0).unwrap();
        
        let transformed = record.transform(1.5);
        assert_eq!(transformed.values.get("pressure"), Some(&150.0));
        assert!(transformed.tags.contains(&"transformed".to_string()));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataSet {
    values: Vec<f64>,
}

impl DataSet {
    pub fn new() -> Self {
        DataSet { values: Vec::new() }
    }

    pub fn from_csv(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut values = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                values.push(value);
            }
        }

        Ok(DataSet { values })
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn variance(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        let mean = self.mean()?;
        let sum_sq_diff: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum();
        Some(sum_sq_diff / (self.values.len() - 1) as f64)
    }

    pub fn standard_deviation(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn min(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::min)
    }

    pub fn max(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_dataset() {
        let ds = DataSet::new();
        assert_eq!(ds.count(), 0);
        assert_eq!(ds.mean(), None);
        assert_eq!(ds.variance(), None);
    }

    #[test]
    fn test_basic_statistics() {
        let mut ds = DataSet::new();
        ds.add_value(1.0);
        ds.add_value(2.0);
        ds.add_value(3.0);
        ds.add_value(4.0);
        ds.add_value(5.0);

        assert_eq!(ds.count(), 5);
        assert_eq!(ds.mean(), Some(3.0));
        assert_eq!(ds.variance(), Some(2.5));
        assert_eq!(ds.standard_deviation(), Some(2.5_f64.sqrt()));
        assert_eq!(ds.min(), Some(1.0));
        assert_eq!(ds.max(), Some(5.0));
    }

    #[test]
    fn test_csv_parsing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "10.5")?;
        writeln!(temp_file, "20.3")?;
        writeln!(temp_file, "15.7")?;
        writeln!(temp_file, "invalid")?;
        writeln!(temp_file, "25.1")?;

        let ds = DataSet::from_csv(temp_file.path().to_str().unwrap())?;
        assert_eq!(ds.count(), 4);
        assert!((ds.mean().unwrap() - 17.9).abs() < 0.001);
        Ok(())
    }
}