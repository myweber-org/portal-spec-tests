use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug)]
struct Record {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid CSV format".into());
        }

        Ok(Record {
            id: parts[0].parse()?,
            category: parts[1].to_string(),
            value: parts[2].parse()?,
            active: parts[3].parse()?,
        })
    }
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines().skip(1) {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let record = Record::from_csv_line(&line)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.active)
            .collect()
    }

    fn aggregate_by_category(&self) -> HashMap<String, (f64, usize)> {
        let mut aggregates = HashMap::new();

        for record in &self.records {
            if !record.active {
                continue;
            }

            let entry = aggregates
                .entry(record.category.clone())
                .or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }

        aggregates
    }

    fn calculate_statistics(&self) -> (f64, f64, f64) {
        let active_values: Vec<f64> = self
            .records
            .iter()
            .filter(|r| r.active)
            .map(|r| r.value)
            .collect();

        if active_values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = active_values.iter().sum();
        let count = active_values.len() as f64;
        let mean = sum / count;

        let variance: f64 = active_values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_file("data.csv")?;

    let filtered = processor.filter_by_category("electronics");
    println!("Filtered records: {}", filtered.len());

    let aggregates = processor.aggregate_by_category();
    for (category, (total, count)) in aggregates {
        println!("Category: {}, Total: {:.2}, Count: {}", category, total, count);
    }

    let (mean, variance, std_dev) = processor.calculate_statistics();
    println!("Statistics - Mean: {:.2}, Variance: {:.2}, Std Dev: {:.2}", 
             mean, variance, std_dev);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = Record::from_csv_line("1,electronics,100.5,true").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.category, "electronics");
        assert_eq!(record.value, 100.5);
        assert!(record.active);
    }

    #[test]
    fn test_data_processing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,category,value,active").unwrap();
        writeln!(file, "1,electronics,100.5,true").unwrap();
        writeln!(file, "2,clothing,50.0,true").unwrap();
        writeln!(file, "3,electronics,75.0,false").unwrap();

        let mut processor = DataProcessor::new();
        processor.load_from_file(file.path().to_str().unwrap()).unwrap();

        let filtered = processor.filter_by_category("electronics");
        assert_eq!(filtered.len(), 1);

        let aggregates = processor.aggregate_by_category();
        assert!(aggregates.contains_key("electronics"));
        assert!(aggregates.contains_key("clothing"));
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct CsvProcessor {
    records: Vec<Record>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
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
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_statistics(&self) -> (usize, f64, f64, f64) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let min = self
            .records
            .iter()
            .map(|r| r.value)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let max = self
            .records
            .iter()
            .map(|r| r.value)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        (count, avg, min, max)
    }

    pub fn export_to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut processor = CsvProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,15.7,Category1").unwrap();
        
        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        
        let filtered = processor.filter_by_category("Category1");
        assert_eq!(filtered.len(), 2);
        
        let avg = processor.calculate_average();
        assert!((avg - 15.5).abs() < 0.1);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
        assert!((stats.1 - 15.5).abs() < 0.1);
        assert!((stats.2 - 10.5).abs() < 0.1);
        assert!((stats.3 - 20.3).abs() < 0.1);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

#[derive(Debug)]
pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = csv::Reader::from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = csv::Writer::from_writer(writer);

        for record in &self.records {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transform_fn(record.value);
        }
    }

    pub fn validate_records(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for (index, record) in self.records.iter().enumerate() {
            if record.name.trim().is_empty() {
                errors.push(format!("Record {} has empty name", index));
            }

            if record.value < 0.0 {
                errors.push(format!("Record {} has negative value: {}", index, record.value));
            }

            if record.category.trim().is_empty() {
                errors.push(format!("Record {} has empty category", index));
            }
        }

        errors
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn clear_records(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();

        let test_record = Record {
            id: 1,
            name: "Test Item".to_string(),
            value: 100.0,
            category: "Electronics".to_string(),
        };

        processor.add_record(test_record);
        assert_eq!(processor.get_record_count(), 1);

        let avg = processor.calculate_average_value();
        assert_eq!(avg, Some(100.0));

        let errors = processor.validate_records();
        assert!(errors.is_empty());

        processor.transform_values(|x| x * 1.1);
        let avg_after = processor.calculate_average_value();
        assert_eq!(avg_after, Some(110.0));
    }

    #[test]
    fn test_csv_io() {
        let mut processor = DataProcessor::new();

        let records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                value: 50.0,
                category: "Category1".to_string(),
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                value: 75.0,
                category: "Category2".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record);
        }

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        processor.save_to_csv(path).unwrap();

        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(path).unwrap();

        assert_eq!(new_processor.get_record_count(), 2);
    }
}