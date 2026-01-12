
use std::error::Error;
use std::fs::File;
use std::path::Path;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub struct DataProcessor {
    records: Vec<Record>,
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
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && r.value <= 1000.0)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_by_id(&self, id: u32) -> Option<&Record> {
        self.records.iter().find(|r| r.id == id)
    }

    pub fn active_count(&self) -> usize {
        self.records.iter().filter(|r| r.active).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,active").unwrap();
        writeln!(file, "1,Test1,100.5,true").unwrap();
        writeln!(file, "2,Test2,200.0,false").unwrap();
        writeln!(file, "3,Test3,300.0,true").unwrap();
        file
    }

    #[test]
    fn test_load_and_validate() {
        let test_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.validate_records().len(), 3);
        assert_eq!(processor.active_count(), 2);
        
        let avg = processor.calculate_average().unwrap();
        assert!((avg - 200.16666666666666).abs() < 0.0001);
        
        let record = processor.find_by_id(2).unwrap();
        assert_eq!(record.name, "Test2");
        assert!(!record.active);
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

    pub fn process_numeric_data(&mut self, key: &str, values: &[f64]) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(values)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, values: &[f64]) -> Result<Vec<f64>, String> {
        let mut result = Vec::with_capacity(values.len());
        
        for &value in values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".to_string());
            }
            if value < 0.0 {
                return Err("Negative values not allowed".to_string());
            }
            result.push(value);
        }
        
        Ok(result)
    }

    fn normalize_data(&self, values: &[f64]) -> Vec<f64> {
        let max_value = values.iter().fold(0.0, |acc, &x| acc.max(x));
        
        if max_value == 0.0 {
            return values.to_vec();
        }

        values.iter()
            .map(|&x| x / max_value)
            .collect()
    }

    fn apply_transformations(&self, values: &[f64]) -> Vec<f64> {
        values.iter()
            .map(|&x| (x * 100.0).round() / 100.0)
            .collect()
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if values.is_empty() {
            return stats;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();

        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
        stats.insert("max".to_string(), *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
        stats.insert("sum".to_string(), sum);
        stats.insert("count".to_string(), count);

        stats
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_numeric_data("test", &data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 5);
        
        let stats = processor.calculate_statistics(&processed);
        assert!(stats.contains_key("mean"));
        assert!(stats.contains_key("std_dev"));
    }

    #[test]
    fn test_validation() {
        let processor = DataProcessor::new();
        let invalid_data = vec![1.0, f64::NAN, 3.0];
        
        let result = processor.validate_data(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0];
        
        let _ = processor.process_numeric_data("cached", &data);
        assert_eq!(processor.cache_size(), 1);
        
        processor.clear_cache();
        assert_eq!(processor.cache_size(), 0);
    }
}