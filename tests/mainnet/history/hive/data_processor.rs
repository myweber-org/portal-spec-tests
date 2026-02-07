
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
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
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
            if parts.len() != 4 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }

            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let active = parts[3].parse::<bool>()?;

            self.records.push(Record {
                id,
                name,
                value,
                active,
            });
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records.iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter()
            .map(|record| record.value)
            .sum();
        
        Some(sum / self.records.len() as f64)
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&Record> {
        self.records.iter()
            .find(|record| record.id == target_id)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_from_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,20.0,false").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.record_count(), 2);
    }

    #[test]
    fn test_filter_active() {
        let mut processor = DataProcessor::new();
        processor.records.push(Record {
            id: 1,
            name: "Active".to_string(),
            value: 10.0,
            active: true,
        });
        processor.records.push(Record {
            id: 2,
            name: "Inactive".to_string(),
            value: 20.0,
            active: false,
        });

        let active = processor.filter_active();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, 1);
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(Record {
            id: 1,
            name: "Test1".to_string(),
            value: 10.0,
            active: true,
        });
        processor.records.push(Record {
            id: 2,
            name: "Test2".to_string(),
            value: 20.0,
            active: true,
        });

        let avg = processor.calculate_average();
        assert_eq!(avg, Some(15.0));
    }

    #[test]
    fn test_find_by_id() {
        let mut processor = DataProcessor::new();
        processor.records.push(Record {
            id: 42,
            name: "Target".to_string(),
            value: 99.9,
            active: true,
        });

        let found = processor.find_by_id(42);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Target");
        
        let not_found = processor.find_by_id(999);
        assert!(not_found.is_none());
    }
}