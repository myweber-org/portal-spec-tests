
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64, values: Vec<f64>) -> Self {
        Self {
            id,
            timestamp,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".into());
        }
        
        if self.values.is_empty() {
            return Err("Values vector cannot be empty".into());
        }
        
        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".into());
            }
        }
        
        Ok(())
    }

    pub fn transform_values<F>(&mut self, transformer: F) 
    where
        F: Fn(f64) -> f64,
    {
        self.values = self.values.iter().map(|&v| transformer(v)).collect();
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        
        let mut processed_record = record.clone();
        processed_record.transform_values(|v| v * 2.0);
        processed_record.add_metadata("processed".to_string(), "true".to_string());
        
        processed.push(processed_record);
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }
    
    let total_values: usize = records.iter().map(|r| r.values.len()).sum();
    let all_values: Vec<f64> = records.iter()
        .flat_map(|r| r.values.iter().copied())
        .collect();
    
    let sum: f64 = all_values.iter().sum();
    let count = all_values.len() as f64;
    
    if count > 0.0 {
        let mean = sum / count;
        let variance: f64 = all_values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);
    }
    
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
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        record.transform_values(|v| v * 2.0);
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord::new(1, 1000, vec![1.0, 2.0]),
            DataRecord::new(2, 2000, vec![3.0, 4.0]),
        ];
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats.get("mean").unwrap(), &2.5);
        assert_eq!(stats.get("total_records").unwrap(), &2.0);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(file_path)?;
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value < 0.0 {
            return Err("Negative value found in data".into());
        }
        
        if !is_valid_category(&record.category) {
            return Err(format!("Invalid category: {}", record.category).into());
        }
        
        records.push(record);
    }

    Ok(records)
}

fn is_valid_category(category: &str) -> bool {
    let valid_categories = ["A", "B", "C", "D"];
    valid_categories.contains(&category)
}

pub fn calculate_total(records: &[Record]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
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
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records
            .iter()
            .map(|record| record.value)
            .sum()
    }

    pub fn save_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut wtr = Writer::from_writer(file);
        
        for record in &self.records {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, id: u32, name: String, value: f64, active: bool) {
        self.records.push(Record {
            id,
            name,
            value,
            active,
        });
    }

    pub fn get_statistics(&self) -> (usize, f64, f64) {
        let count = self.records.len();
        let total = self.calculate_total();
        let average = if count > 0 {
            total / count as f64
        } else {
            0.0
        };
        
        (count, total, average)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(1, "Test1".to_string(), 10.5, true);
        processor.add_record(2, "Test2".to_string(), 20.0, false);
        processor.add_record(3, "Test3".to_string(), 30.5, true);
        
        let active_records = processor.filter_active();
        assert_eq!(active_records.len(), 2);
        
        let total = processor.calculate_total();
        assert_eq!(total, 61.0);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
        assert_eq!(stats.1, 61.0);
        assert_eq!(stats.2, 61.0 / 3.0);
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        processor.add_record(1, "Alpha".to_string(), 100.0, true);
        processor.add_record(2, "Beta".to_string(), 200.0, false);
        
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_str().unwrap();
        
        processor.save_to_csv(temp_path)?;
        
        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(temp_path)?;
        
        assert_eq!(new_processor.records.len(), 2);
        Ok(())
    }
}