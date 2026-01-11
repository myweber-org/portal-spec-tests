
use csv;
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

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        println!("Loaded {} records from {}", self.records.len(), file_path);
        Ok(())
    }

    fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .collect()
    }

    fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    fn find_by_id(&self, id: u32) -> Option<&Record> {
        self.records.iter().find(|r| r.id == id)
    }

    fn export_to_json(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        serde_json::to_writer_pretty(file, &self.records)?;
        println!("Exported {} records to {}", self.records.len(), file_path);
        Ok(())
    }
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value >= 0.0
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("input.csv")?;
    
    let avg = processor.calculate_average();
    if let Some(average) = avg {
        println!("Average value: {:.2}", average);
    }
    
    let active_records = processor.filter_active();
    println!("Active records: {}", active_records.len());
    
    if let Some(record) = processor.find_by_id(42) {
        println!("Found record: {:?}", record);
    }
    
    processor.export_to_json("output.json")?;
    
    for record in &processor.records {
        if validate_record(record) {
            println!("Valid record: ID {}", record.id);
        }
    }
    
    Ok(())
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
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
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.3,category_b").unwrap();
        writeln!(temp_file, "3,15.7,category_a").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert!((stats.0 - 15.5).abs() < 0.1);
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
    }
}use csv::Reader;
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
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

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.records
            .iter()
            .map(|r| r.category.clone())
            .collect();
        
        categories.sort();
        categories.dedup();
        categories
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Alpha").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Beta").unwrap();
        writeln!(temp_file, "3,ItemC,15.7,Alpha").unwrap();
        writeln!(temp_file, "4,ItemD,25.1,Gamma").unwrap();
        temp_file
    }

    #[test]
    fn test_load_and_filter() {
        let temp_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.count_records(), 4);
        
        let alpha_items = processor.filter_by_category("Alpha");
        assert_eq!(alpha_items.len(), 2);
        
        let avg = processor.calculate_average();
        assert!((avg - 17.9).abs() < 0.01);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.name, "ItemD");
        
        let categories = processor.get_categories();
        assert_eq!(categories, vec!["Alpha", "Beta", "Gamma"]);
    }
}