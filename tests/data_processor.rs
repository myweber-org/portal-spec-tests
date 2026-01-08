use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                self.parse_header(&line);
                continue;
            }
            
            if let Ok(value) = line.trim().parse::<f64>() {
                self.data.push(value);
            }
        }
        
        self.metadata.insert("source".to_string(), filepath.to_string());
        self.metadata.insert("loaded_at".to_string(), chrono::Local::now().to_rfc3339());
        
        Ok(())
    }

    fn parse_header(&mut self, header_line: &str) {
        let columns: Vec<&str> = header_line.split(',').collect();
        self.metadata.insert("columns".to_string(), columns.len().to_string());
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.data.is_empty() {
            return stats;
        }
        
        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        
        stats
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x >= threshold)
            .cloned()
            .collect()
    }

    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn data_count(&self) -> usize {
        self.data.len()
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
        writeln!(temp_file, "value").unwrap();
        writeln!(temp_file, "10.5").unwrap();
        writeln!(temp_file, "20.3").unwrap();
        writeln!(temp_file, "15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.data_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert!((stats["mean"] - 15.5).abs() < 0.1);
        assert_eq!(stats["count"], 3.0);
        
        let filtered = processor.filter_by_threshold(16.0);
        assert_eq!(filtered.len(), 1);
        assert!((filtered[0] - 20.3).abs() < 0.1);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationError("Timestamp cannot be negative".to_string()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::ValidationError("Values cannot be empty".to_string()));
        }
        
        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationError("Key cannot be empty".to_string()));
            }
            if !value.is_finite() {
                return Err(DataError::ValidationError(format!("Value for {} is not finite", key)));
            }
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), DataError> {
        if !multiplier.is_finite() || multiplier == 0.0 {
            return Err(DataError::InvalidInput("Invalid multiplier".to_string()));
        }
        
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
        
        Ok(())
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    pub fn calculate_sum(&self) -> f64 {
        self.values.values().sum()
    }
    
    pub fn calculate_average(&self) -> Option<f64> {
        let count = self.values.len() as f64;
        if count > 0.0 {
            Some(self.calculate_sum() / count)
        } else {
            None
        }
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    statistics: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            statistics: HashMap::new(),
        }
    }
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        record.validate()?;
        self.records.push(record);
        self.update_statistics();
        Ok(())
    }
    
    pub fn process_all(&mut self, multiplier: f64) -> Result<(), DataError> {
        if !multiplier.is_finite() {
            return Err(DataError::InvalidInput("Multiplier must be finite".to_string()));
        }
        
        for record in &mut self.records {
            record.transform(multiplier)?;
        }
        
        self.update_statistics();
        Ok(())
    }
    
    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }
    
    pub fn get_statistics(&self) -> &HashMap<String, f64> {
        &self.statistics
    }
    
    fn update_statistics(&mut self) {
        self.statistics.clear();
        
        if self.records.is_empty() {
            return;
        }
        
        let total_records = self.records.len() as f64;
        let mut sum_of_sums = 0.0;
        let mut sum_of_averages = 0.0;
        let mut valid_averages_count = 0;
        
        for record in &self.records {
            sum_of_sums += record.calculate_sum();
            if let Some(avg) = record.calculate_average() {
                sum_of_averages += avg;
                valid_averages_count += 1;
            }
        }
        
        self.statistics.insert("total_records".to_string(), total_records);
        self.statistics.insert("sum_of_sums".to_string(), sum_of_sums);
        
        if valid_averages_count > 0 {
            self.statistics.insert(
                "average_of_averages".to_string(),
                sum_of_averages / valid_averages_count as f64
            );
        }
        
        self.statistics.insert(
            "mean_sum".to_string(),
            sum_of_sums / total_records
        );
    }
    
    pub fn clear(&mut self) {
        self.records.clear();
        self.statistics.clear();
    }
    
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), 25.5);
        
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: values.clone(),
            tags: vec!["sensor".to_string()],
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            timestamp: 1234567890,
            values: values,
            tags: vec![],
        };
        
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut values = HashMap::new();
        values.insert("value".to_string(), 10.0);
        
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values,
            tags: vec![],
        };
        
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.values.get("value").unwrap(), &20.0);
    }
    
    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut values = HashMap::new();
        values.insert("metric".to_string(), 5.0);
        
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values,
            tags: vec!["test".to_string()],
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.record_count(), 1);
        
        let filtered = processor.filter_by_tag("test");
        assert_eq!(filtered.len(), 1);
    }
}