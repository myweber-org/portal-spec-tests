
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
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
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let name = parts[1].to_string();

            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let active = match parts[3].to_lowercase().as_str() {
                "true" => true,
                "false" => false,
                _ => continue,
            };

            let record = Record::new(id, name, value, active);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_value(&self, threshold: f64) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value > threshold)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn count_active(&self) -> usize {
        self.records.iter().filter(|record| record.active).count()
    }

    pub fn get_records(&self) -> &[Record] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "test".to_string(), 10.5, true);
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -5.0, false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.get_records().len(), 0);
        assert_eq!(processor.count_active(), 0);
        assert_eq!(processor.calculate_average(), None);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
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
    stats: ProcessingStats,
}

#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub total_records: usize,
    pub valid_records: usize,
    pub invalid_records: usize,
    pub sum_values: f64,
    pub avg_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            stats: ProcessingStats::default(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut line_count = 0;
        for line in reader.lines() {
            let line = line?;
            line_count += 1;
            
            if line_count == 1 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                let id = parts[0].parse::<u32>().unwrap_or(0);
                let value = parts[1].parse::<f64>().unwrap_or(0.0);
                let category = parts[2].to_string();
                
                let record = DataRecord::new(id, value, &category);
                self.records.push(record);
            }
        }
        
        self.calculate_stats();
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
        self.calculate_stats();
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    fn calculate_stats(&mut self) {
        self.stats.total_records = self.records.len();
        self.stats.valid_records = self.records.iter().filter(|r| r.valid).count();
        self.stats.invalid_records = self.stats.total_records - self.stats.valid_records;
        
        let valid_values: Vec<f64> = self.records
            .iter()
            .filter(|r| r.valid)
            .map(|r| r.value)
            .collect();
        
        self.stats.sum_values = valid_values.iter().sum();
        self.stats.avg_value = if !valid_values.is_empty() {
            self.stats.sum_values / valid_values.len() as f64
        } else {
            0.0
        };
    }

    pub fn get_stats(&self) -> &ProcessingStats {
        &self.stats
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.stats = ProcessingStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(1, 10.0, "A");
        let record2 = DataRecord::new(2, 20.0, "B");
        let record3 = DataRecord::new(3, -5.0, "C");
        
        processor.add_record(record1);
        processor.add_record(record2);
        processor.add_record(record3);
        
        let stats = processor.get_stats();
        assert_eq!(stats.total_records, 3);
        assert_eq!(stats.valid_records, 2);
        assert_eq!(stats.invalid_records, 1);
        assert_eq!(stats.sum_values, 30.0);
        assert_eq!(stats.avg_value, 15.0);
        
        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 2);
        
        processor.clear();
        let stats = processor.get_stats();
        assert_eq!(stats.total_records, 0);
    }

    #[test]
    fn test_csv_loading() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,value,category")?;
        writeln!(temp_file, "1,10.5,TypeA")?;
        writeln!(temp_file, "2,20.3,TypeB")?;
        writeln!(temp_file, "3,-5.0,TypeC")?;
        
        let mut processor = DataProcessor::new();
        processor.load_from_csv(temp_file.path())?;
        
        let stats = processor.get_stats();
        assert_eq!(stats.total_records, 3);
        assert_eq!(stats.valid_records, 2);
        
        let type_a_records = processor.filter_by_category("TypeA");
        assert_eq!(type_a_records.len(), 1);
        
        Ok(())
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
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

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = Reader::from_reader(file);
        
        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_statistics(&self) -> (usize, f64, f64, f64) {
        let count = self.records.len();
        let avg = self.calculate_average();
        
        let min = self.records
            .iter()
            .map(|r| r.value)
            .fold(f64::INFINITY, f64::min);
            
        let max = self.records
            .iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);
        
        (count, avg, min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,15.7,Category1").unwrap();
        
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let filtered = processor.filter_by_category("Category1");
        assert_eq!(filtered.len(), 2);
        
        let avg = processor.calculate_average();
        assert!((avg - 15.5).abs() < 0.1);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
    }
}