
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u64,
    timestamp: i64,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64, values: Vec<f64>) -> Self {
        Self {
            id,
            timestamp,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        if self.timestamp < 0 {
            return Err("Invalid timestamp".into());
        }
        if self.values.is_empty() {
            return Err("Empty values array".into());
        }
        Ok(())
    }

    pub fn normalize_values(&mut self) {
        if let Some(max) = self.values.iter().copied().reduce(f64::max) {
            if max != 0.0 {
                for value in &mut self.values {
                    *value /= max;
                }
            }
        }
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::new();
    
    for record in records {
        record.validate()?;
        let mut processed_record = DataRecord::new(
            record.id,
            record.timestamp,
            record.values.clone(),
        );
        processed_record.normalize_values();
        processed.push(processed_record);
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }
    
    let all_values: Vec<f64> = records.iter()
        .flat_map(|r| r.values.iter().copied())
        .collect();
    
    let count = all_values.len() as f64;
    let sum: f64 = all_values.iter().sum();
    let mean = sum / count;
    
    let variance: f64 = all_values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / count;
    
    stats.insert("count".to_string(), count);
    stats.insert("mean".to_string(), mean);
    stats.insert("variance".to_string(), variance);
    stats.insert("min".to_string(), all_values.iter().copied().fold(f64::INFINITY, f64::min));
    stats.insert("max".to_string(), all_values.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_normalize_values() {
        let mut record = DataRecord::new(1, 1234567890, vec![2.0, 4.0, 6.0]);
        record.normalize_values();
        assert_eq!(record.values, vec![1.0/3.0, 2.0/3.0, 1.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord::new(1, 1000, vec![1.0, 2.0]),
            DataRecord::new(2, 2000, vec![3.0, 4.0]),
        ];
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats["count"], 4.0);
        assert_eq!(stats["mean"], 2.5);
        assert_eq!(stats["min"], 1.0);
        assert_eq!(stats["max"], 4.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.trim().is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        
        Ok(Self {
            id,
            value,
            category: category.to_string(),
        })
    }
    
    pub fn is_valid(&self) -> bool {
        self.value >= 0.0 && !self.category.trim().is_empty()
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
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), String> {
        if !record.is_valid() {
            return Err("Invalid record data".to_string());
        }
        
        if self.records.iter().any(|r| r.id == record.id) {
            return Err(format!("Duplicate ID found: {}", record.id));
        }
        
        self.records.push(record);
        Ok(())
    }
    
    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut loaded_count = 0;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            
            if parts.len() != 3 {
                eprintln!("Warning: Invalid format on line {}", line_num + 1);
                continue;
            }
            
            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => {
                    eprintln!("Warning: Invalid ID on line {}", line_num + 1);
                    continue;
                }
            };
            
            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    eprintln!("Warning: Invalid value on line {}", line_num + 1);
                    continue;
                }
            };
            
            let category = parts[2];
            
            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    if let Err(e) = self.add_record(record) {
                        eprintln!("Warning: {} on line {}", e, line_num + 1);
                    } else {
                        loaded_count += 1;
                    }
                }
                Err(e) => {
                    eprintln!("Warning: {} on line {}", e, line_num + 1);
                }
            }
        }
        
        Ok(loaded_count)
    }
    
    pub fn get_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }
    
    pub fn get_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.get_total_value() / self.records.len() as f64)
        }
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }
    
    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }
    
    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, "test").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
        assert!(record.is_valid());
    }
    
    #[test]
    fn test_invalid_record_negative_value() {
        let result = DataRecord::new(1, -10.0, "test");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_data_processor_add_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord::new(1, 100.0, "category_a").unwrap();
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.get_records().len(), 1);
    }
    
    #[test]
    fn test_duplicate_id_rejection() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord::new(1, 100.0, "cat1").unwrap();
        let record2 = DataRecord::new(1, 200.0, "cat2").unwrap();
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_err());
    }
    
    #[test]
    fn test_total_value_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, 10.0, "test").unwrap()).unwrap();
        processor.add_record(DataRecord::new(2, 20.0, "test").unwrap()).unwrap();
        processor.add_record(DataRecord::new(3, 30.0, "test").unwrap()).unwrap();
        
        assert_eq!(processor.get_total_value(), 60.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: String) -> Self {
        DataRecord {
            id,
            value,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.timestamp.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut count = 0;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let timestamp = parts[2].to_string();

            let record = DataRecord::new(id, value, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value > threshold)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "2024-01-15T10:30:00Z".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, f64::NAN, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,timestamp").unwrap();
        writeln!(temp_file, "1,10.5,2024-01-15T10:30:00Z").unwrap();
        writeln!(temp_file, "2,20.3,2024-01-15T11:30:00Z").unwrap();
        writeln!(temp_file, "3,15.7,2024-01-15T12:30:00Z").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 15.5).abs() < 0.1);
        
        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 2);
    }
}