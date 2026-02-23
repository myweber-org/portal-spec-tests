
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
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
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            let record = DataRecord::new(id, value, parts[2]);
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.valid {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record);
            }
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn count_valid(&self) -> usize {
        self.filter_valid().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
        assert!(record.valid);
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -5.0, "");
        assert!(!record.valid);
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.0,beta").unwrap();
        writeln!(temp_file, "3,invalid,gamma").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 2);
        assert_eq!(processor.count_valid(), 2);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "test"));
        processor.records.push(DataRecord::new(2, 20.0, "test"));
        processor.records.push(DataRecord::new(3, -5.0, "test"));

        let average = processor.calculate_average();
        assert_eq!(average, Some(15.0));
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);
        assert_eq!(processor.calculate_average(), None);
    }
}use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct ProcessedData {
    pub record_id: u32,
    pub normalized_values: Vec<f64>,
    pub checksum: u32,
    pub processed_at: i64,
}

pub fn validate_record(record: &DataRecord) -> Result<(), Box<dyn Error>> {
    if record.id == 0 {
        return Err("Invalid record ID".into());
    }
    
    if record.timestamp <= 0 {
        return Err("Invalid timestamp".into());
    }
    
    if record.values.is_empty() {
        return Err("Empty values array".into());
    }
    
    for value in &record.values {
        if !value.is_finite() {
            return Err("Non-finite value detected".into());
        }
    }
    
    Ok(())
}

pub fn normalize_values(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    
    let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    if (max - min).abs() < f64::EPSILON {
        return values.iter().map(|_| 0.0).collect();
    }
    
    values.iter()
        .map(|&v| (v - min) / (max - min))
        .collect()
}

pub fn calculate_checksum(values: &[f64]) -> u32 {
    values.iter()
        .map(|&v| v.to_bits() as u32)
        .fold(0u32, |acc, bits| acc.wrapping_add(bits))
}

pub fn process_data_record(record: DataRecord) -> Result<ProcessedData, Box<dyn Error>> {
    validate_record(&record)?;
    
    let normalized = normalize_values(&record.values);
    let checksum = calculate_checksum(&normalized);
    
    Ok(ProcessedData {
        record_id: record.id,
        normalized_values: normalized,
        checksum,
        processed_at: chrono::Utc::now().timestamp(),
    })
}

pub fn batch_process_records(records: Vec<DataRecord>) -> Vec<Result<ProcessedData, Box<dyn Error>>> {
    records.into_iter()
        .map(process_data_record)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_normalize_values() {
        let values = vec![1.0, 2.0, 3.0];
        let normalized = normalize_values(&values);
        
        assert_eq!(normalized.len(), 3);
        assert!((normalized[0] - 0.0).abs() < 0.001);
        assert!((normalized[1] - 0.5).abs() < 0.001);
        assert!((normalized[2] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_process_data_record() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 42,
            timestamp: 1609459200,
            values: vec![10.0, 20.0, 30.0],
            metadata,
        };
        
        let result = process_data_record(record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.record_id, 42);
        assert_eq!(processed.normalized_values.len(), 3);
        assert!(processed.processed_at > 0);
    }
}use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        
        if self.values.is_empty() {
            return Err("Empty values vector".into());
        }

        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".into());
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

pub fn process_records(records: &[DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::new();
    
    for record in records {
        record.validate()?;
        
        let mut processed_record = record.clone();
        
        let stats = record.calculate_statistics();
        processed_record.metadata.insert(
            "mean".to_string(),
            format!("{:.4}", stats.0)
        );
        processed_record.metadata.insert(
            "std_dev".to_string(),
            format!("{:.4}", stats.2)
        );
        
        if stats.2 > 0.0 {
            let normalized_values: Vec<f64> = record.values
                .iter()
                .map(|&x| (x - stats.0) / stats.2)
                .collect();
            processed_record.values = normalized_values;
        }
        
        processed.push(processed_record);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }

    #[test]
    fn test_record_processing() {
        let records = vec![
            DataRecord::new(1, vec![1.0, 2.0, 3.0]),
            DataRecord::new(2, vec![4.0, 5.0, 6.0]),
        ];
        
        let processed = process_records(&records);
        assert!(processed.is_ok());
        assert_eq!(processed.unwrap().len(), 2);
    }
}use std::error::Error;
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

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }
        
        let mean = self.calculate_mean()?;
        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;
        
        Some(variance.sqrt())
    }

    pub fn get_category_frequency(&self, category: &str) -> u32 {
        *self.frequency_map.get(category).unwrap_or(&0)
    }

    pub fn get_top_categories(&self, limit: usize) -> Vec<(String, u32)> {
        let mut categories: Vec<_> = self.frequency_map.iter().collect();
        categories.sort_by(|a, b| b.1.cmp(a.1));
        
        categories.iter()
            .take(limit)
            .map(|(&ref k, &v)| (k.clone(), v))
            .collect()
    }

    pub fn filter_data(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x >= threshold)
            .cloned()
            .collect()
    }

    pub fn data_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let std_dev = self.calculate_standard_deviation().unwrap_or(0.0);
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        format!(
            "Records: {}, Mean: {:.2}, StdDev: {:.2}, Min: {:.2}, Max: {:.2}",
            self.data.len(),
            mean,
            std_dev,
            min,
            max
        )
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
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.calculate_mean().unwrap(), 13.85);
        assert_eq!(processor.get_category_frequency("A"), 2);
        assert_eq!(processor.get_category_frequency("B"), 1);
        
        let filtered = processor.filter_data(12.0);
        assert_eq!(filtered.len(), 2);
        
        let top_categories = processor.get_top_categories(2);
        assert_eq!(top_categories[0].0, "A");
        assert_eq!(top_categories[0].1, 2);
    }
}