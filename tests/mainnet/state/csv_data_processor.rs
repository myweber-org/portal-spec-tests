
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub category: String,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
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

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_active_records(&self) -> Vec<&Record> {
        self.records.iter().filter(|r| r.active).collect()
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for record in &self.records {
            *counts.entry(record.category.clone()).or_insert(0) += 1;
        }
        
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,category,value,active").unwrap();
        writeln!(file, "1,electronics,250.50,true").unwrap();
        writeln!(file, "2,books,45.99,false").unwrap();
        writeln!(file, "3,electronics,120.75,true").unwrap();
        writeln!(file, "4,clothing,89.99,true").unwrap();
        file
    }

    #[test]
    fn test_load_and_filter() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(csv_file.path()).unwrap();
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = processor.calculate_average().unwrap();
        assert!(avg > 0.0);
        
        let active_records = processor.get_active_records();
        assert_eq!(active_records.len(), 3);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 1);
        
        let counts = processor.count_by_category();
        assert_eq!(counts.get("electronics"), Some(&2));
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn load_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);
    
    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

fn filter_records(records: &[Record], category_filter: &str) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.category == category_filter && r.active)
        .cloned()
        .collect()
}

fn aggregate_values(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = if count > 0.0 { sum / count } else { 0.0 };
    let max = records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);
    
    (sum, avg, max)
}

fn save_results<P: AsRef<Path>>(records: &[Record], path: P) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = WriterBuilder::new().has_headers(true).from_writer(file);
    
    for record in records {
        writer.serialize(record)?;
    }
    
    writer.flush()?;
    Ok(())
}

fn process_data(input_path: &str, output_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
    let all_records = load_csv(input_path)?;
    let filtered_records = filter_records(&all_records, category);
    
    if !filtered_records.is_empty() {
        let (total, average, maximum) = aggregate_values(&filtered_records);
        println!("Processing category: {}", category);
        println!("Records found: {}", filtered_records.len());
        println!("Total value: {:.2}", total);
        println!("Average value: {:.2}", average);
        println!("Maximum value: {:.2}", maximum);
        
        save_results(&filtered_records, output_path)?;
        println!("Results saved to: {}", output_path);
    } else {
        println!("No active records found for category: {}", category);
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/processed_results.csv";
    let target_category = "electronics";
    
    process_data(input_file, output_file, target_category)
}