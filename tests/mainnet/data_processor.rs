use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines().skip(1) {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    self.data.push(value);
                }
                
                let category = parts[0].to_string();
                *self.frequency_map.entry(category).or_insert(0) += 1;
            }
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

    pub fn calculate_median(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mid = sorted_data.len() / 2;
        if sorted_data.len() % 2 == 0 {
            Some((sorted_data[mid - 1] + sorted_data[mid]) / 2.0)
        } else {
            Some(sorted_data[mid])
        }
    }

    pub fn get_category_frequency(&self, category: &str) -> u32 {
        *self.frequency_map.get(category).unwrap_or(&0)
    }

    pub fn get_unique_categories(&self) -> Vec<String> {
        self.frequency_map.keys().cloned().collect()
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
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
        writeln!(temp_file, "category,value").unwrap();
        writeln!(temp_file, "A,10.5").unwrap();
        writeln!(temp_file, "B,20.3").unwrap();
        writeln!(temp_file, "A,15.7").unwrap();
        writeln!(temp_file, "C,8.9").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.calculate_mean(), Some(13.85));
        assert_eq!(processor.calculate_median(), Some(13.1));
        assert_eq!(processor.get_category_frequency("A"), 2);
        
        let filtered = processor.filter_by_threshold(12.0);
        assert_eq!(filtered.len(), 2);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
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

    pub fn add_value(&mut self, key: &str, value: f64) {
        self.values.insert(key.to_string(), value);
    }

    pub fn add_tag(&mut self, tag: &str) {
        self.tags.push(tag.to_string());
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationFailed("ID cannot be zero".to_string()));
        }

        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        if self.values.is_empty() {
            return Err(DataError::ValidationFailed(
                "Record must contain at least one value".to_string(),
            ));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationFailed(
                    "Value key cannot be empty".to_string(),
                ));
            }
            if !value.is_finite() {
                return Err(DataError::ValidationFailed(format!(
                    "Value for '{}' must be finite",
                    key
                )));
            }
        }

        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) {
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        let values: Vec<f64> = self.values.values().copied().collect();

        if !values.is_empty() {
            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;

            let variance: f64 = values
                .iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>()
                / count;

            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("count".to_string(), count);
            stats.insert("sum".to_string(), sum);

            if let Some(min) = values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()) {
                stats.insert("min".to_string(), *min);
            }
            if let Some(max) = values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
                stats.insert("max".to_string(), *max);
            }
        }

        stats
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }

    pub fn process_all(&mut self, multiplier: f64) {
        for record in &mut self.records {
            record.transform(multiplier);
        }
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut global_stats = HashMap::new();
        let mut all_values = Vec::new();

        for record in &self.records {
            let stats = record.calculate_statistics();
            for (key, value) in stats {
                *global_stats.entry(key).or_insert(0.0) += value;
            }

            all_values.extend(record.values.values().copied());
        }

        if !all_values.is_empty() {
            let total_count = all_values.len() as f64;
            let total_sum: f64 = all_values.iter().sum();
            let total_mean = total_sum / total_count;

            let total_variance: f64 = all_values
                .iter()
                .map(|v| (v - total_mean).powi(2))
                .sum::<f64>()
                / total_count;

            global_stats.insert("global_mean".to_string(), total_mean);
            global_stats.insert("global_variance".to_string(), total_variance);
            global_stats.insert("total_count".to_string(), total_count);
            global_stats.insert("total_sum".to_string(), total_sum);
        }

        global_stats
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("temperature", 25.5);
        record.add_tag("sensor");

        assert!(record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1625097600);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("value", 10.0);
        record.transform(2.0);

        assert_eq!(record.values.get("value"), Some(&20.0));
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();

        let mut record1 = DataRecord::new(1, 1625097600);
        record1.add_value("a", 1.0);
        record1.add_tag("test");

        let mut record2 = DataRecord::new(2, 1625097601);
        record2.add_value("b", 2.0);
        record2.add_tag("test");

        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        assert_eq!(processor.len(), 2);

        let filtered = processor.filter_by_tag("test");
        assert_eq!(filtered.len(), 2);

        processor.process_all(2.0);
        let stats = processor.get_statistics();
        assert!(stats.contains_key("global_mean"));
    }
}